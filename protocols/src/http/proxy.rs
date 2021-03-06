use std::{convert::Infallible, path::Path, sync::Arc};

use anyhow::Result;
use ca::CertificateAuthority;
use config::Proxy;
use http::{Request, Response, StatusCode, Uri, Version};
use hyper::{client::HttpConnector, server::conn::Http, service::service_fn, Body};
use hyper_alpn::AlpnConnector;
use hyper_tls::HttpsConnector;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::TlsAcceptor;
use wasi_runtime::WasiRuntime;

use crate::tcp::tunnel;

use super::{
    hostname::Hostname,
    pre_request::{process_pre_request, ProxyMode},
    request::process_request,
    response::process_response,
};

// Each protocol defines a context, and is passed in via process request
#[derive(Clone)]
pub struct HttpContext {
    client_h1: hyper::Client<HttpsConnector<HttpConnector>, hyper::Body>,
    client_h2: hyper::Client<AlpnConnector, hyper::Body>,
    #[allow(unused)]
    ca: CertificateAuthority,
}

impl HttpContext {
    pub async fn new(ca_path: &Path) -> Result<HttpContext> {
        let alpn = AlpnConnector::new();
        let client_h2 = hyper::Client::builder()
            .http2_only(true)
            .build::<_, hyper::Body>(alpn);

        let https = HttpsConnector::new();
        let client_h1 = hyper::Client::builder().build::<_, hyper::Body>(https);
        let ca = CertificateAuthority::load(ca_path).await?;

        Ok(Self {
            client_h1,
            client_h2,
            ca,
        })
    }
}
async fn negotiate_version(scheme: &str, host: &str, context: &HttpContext) -> Result<Version> {
    let uri = Uri::builder()
        .scheme(scheme)
        .authority(host)
        .path_and_query("/")
        .build()?;
    let head_req = hyper::Request::builder()
        .uri(uri)
        .method("HEAD")
        .body(Body::from(""))?;
    tracing::trace!(?head_req, "Built HEAD request for negotiation");
    match context.client_h2.request(head_req).await {
        Ok(resp) => {
            tracing::debug!(?resp, "Received HEAD response from server.");
            Ok(resp.version())
        }
        Err(error) => {
            tracing::debug!(%error, "Received error when connecting to client. Assuming HTTP/1.1");
            Ok(Version::HTTP_11)
        }
    }
}

fn error_payload(error: anyhow::Error) -> Response<Body> {
    let payload = format!("Error making request: {error}");
    let mut resp = Response::new(Body::from(payload));
    *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    resp
}

async fn http_proxy_service(
    mut req: Request<Body>,
    proxy: Proxy,
    mut wasi_runtime: WasiRuntime,
    context: HttpContext,
    version: Option<Version>,
) -> Result<Response<Body>, Infallible> {
    let scheme: String = if proxy.tls {
        "https".into()
    } else {
        "http".into()
    };
    let p_and_q = req
        .uri()
        .path_and_query()
        .map(|p_and_q| p_and_q.as_str())
        .unwrap_or("/");
    let host = proxy.upstream_address();
    let req_path = proxy.request_wasi_module_path.clone();
    let resp_path = proxy.response_wasi_module_path.clone();
    *req.uri_mut() = Uri::builder()
        .scheme(scheme.as_str())
        .authority(host.as_str())
        .path_and_query(p_and_q)
        .build()
        .unwrap();
    let request = match process_request(
        &mut wasi_runtime,
        req,
        req_path,
        scheme.as_str(),
        host.as_str(),
        proxy.clone(),
    )
    .await
    {
        Ok(request) => {
            tracing::info!(new_request = ?request, "New request.");
            request
        }
        Err(err) => {
            tracing::error!(?err, "Error getting request from WASM.");
            return Ok(error_payload(err));
        }
    };

    let version = match version {
        Some(version) => version,
        None => match negotiate_version(&scheme, &host, &context).await {
            Ok(version) => version,
            Err(err) => {
                tracing::error!(?err, "Error negotiating HTTP version.");
                return Ok(error_payload(err));
            }
        },
    };

    let method = request.method().clone();
    let uri = request.uri().clone();

    let resp = match version {
        Version::HTTP_09 | Version::HTTP_10 | Version::HTTP_11 => context
            .client_h1
            .request(request)
            .await
            .map_err(anyhow::Error::from),
        Version::HTTP_2 => context
            .client_h2
            .request(request)
            .await
            .map_err(anyhow::Error::from),
        http_version => Err(anyhow::Error::msg(format!(
            "{:?} not supported",
            http_version
        ))),
    };

    let resp = match resp {
        Ok(resp) => resp,
        Err(err) => {
            tracing::error!(?err, "Error performing HTTP request.");
            return Ok(error_payload(err));
        }
    };

    match process_response(
        &mut wasi_runtime,
        resp,
        resp_path,
        proxy,
        uri,
        version,
        method,
    )
    .await
    {
        Ok(resp) => {
            tracing::info!(new_response = ?resp, "New response.");
            Ok(resp)
        }
        Err(err) => {
            tracing::error!(?err, "Error processing response from WASM.");
            Ok(error_payload(err))
        }
    }
}

pub async fn http_proxy<T: AsyncRead + AsyncWrite + std::marker::Unpin + 'static>(
    socket: T,
    proxy: Proxy,
    wasi_runtime: WasiRuntime,
    context: HttpContext,
) -> Result<()> {
    let service = service_fn(|request: Request<Body>| {
        let wasi_runtime = wasi_runtime.clone();
        let context = context.clone();
        let proxy = proxy.clone();
        async move { http_proxy_service(request, proxy, wasi_runtime, context, None).await }
    });

    if let Err(http_err) = Http::new().serve_connection(socket, service).await {
        tracing::error!(%http_err, "Error while serving HTTP connection");
    }

    Ok(())
}

pub async fn https_proxy<T: AsyncRead + AsyncWrite + std::marker::Unpin + 'static>(
    socket: T,
    proxy: Proxy,
    wasi_runtime: WasiRuntime,
    mut context: HttpContext,
    hostname: Hostname,
) -> Result<()> {
    let version = negotiate_version(&hostname.scheme, &hostname.host, &context).await?;
    let alpn_protocols: Vec<Vec<u8>> = match version {
        Version::HTTP_2 => vec!["h2".into(), "http/1.1".into()],
        _ => vec!["http/1.1".into()],
    };
    let mut config = context
        .ca
        .build_certs(&hostname.host, hostname.port)
        .await?;
    config.alpn_protocols = alpn_protocols;

    let acceptor = TlsAcceptor::from(Arc::new(config));
    let stream = acceptor.accept(socket).await?;

    let service = service_fn(|request: Request<Body>| {
        let wasi_runtime = wasi_runtime.clone();
        let context = context.clone();
        let proxy = proxy.clone();
        async move { http_proxy_service(request, proxy, wasi_runtime, context, Some(version)).await }
    });

    if let Err(http_err) = Http::new().serve_connection(stream, service).await {
        tracing::error!(%http_err, "Error while serving HTTP connection");
    }

    Ok(())
}

async fn proxy_https(
    req: Request<Body>,
    hostname: Hostname,
    proxy: Proxy,
    mut wasi_runtime: WasiRuntime,
    context: HttpContext,
) -> Result<(), Infallible> {
    tokio::spawn(async move {
        match hyper::upgrade::on(req).await {
            Ok(upgraded) => {
                let path = proxy.pre_request_wasi_module_path.clone();
                let mut proxy = proxy.clone();
                proxy.upstream_address = hostname.host.clone();
                proxy.upstream_port = hostname.port;
                proxy.tls = false;
                match process_pre_request(&mut wasi_runtime, hostname.clone(), path, proxy.clone())
                    .await
                    .unwrap_or(ProxyMode::Pass)
                {
                    ProxyMode::Intercept => {
                        let res =
                            https_proxy(upgraded, proxy, wasi_runtime, context, hostname).await;
                        tracing::info!(?res, "Finished intercepting.");
                    }
                    ProxyMode::Pass => {
                        let res = tunnel(upgraded, &hostname.authority).await;
                        tracing::info!(?res, "Finished tunneling.");
                    }
                }
            }
            Err(err) => tracing::error!(%err, "Error upgrading request."),
        };
    });
    Ok(())
}

async fn proxy_http(
    req: Request<Body>,
    hostname: Hostname,
    proxy: Proxy,
    mut wasi_runtime: WasiRuntime,
    context: HttpContext,
) -> Result<Response<Body>, Infallible> {
    let path = proxy.pre_request_wasi_module_path.clone();
    let mut proxy = proxy.clone();
    proxy.upstream_address = hostname.host.clone();
    proxy.upstream_port = hostname.port;
    proxy.tls = false;
    match process_pre_request(&mut wasi_runtime, hostname.clone(), path, proxy.clone())
        .await
        .unwrap_or(ProxyMode::Pass)
    {
        ProxyMode::Intercept => {
            let res = http_proxy_service(req, proxy, wasi_runtime, context, None).await;
            tracing::info!(?res, "Finished intercepting.");
            res
        }
        ProxyMode::Pass => {
            proxy.request_wasi_module_path = None;
            proxy.response_wasi_module_path = None;
            let res = http_proxy_service(req, proxy, wasi_runtime, context, None).await;
            tracing::info!(?res, "Finished tunneling.");
            res
        }
    }
}

async fn http_forward_proxy_service(
    req: Request<Body>,
    proxy: Proxy,
    wasi_runtime: WasiRuntime,
    context: HttpContext,
) -> Result<Response<Body>, Infallible> {
    tracing::info!(?req, "Received request");
    let hostname = match Hostname::try_from(&req) {
        Ok(hostname) => hostname,
        Err(err) => {
            tracing::warn!(?err, "Error processing hostname from connect request.");
            return Ok(Response::new(hyper::Body::empty()));
        }
    };

    if req.method() == hyper::Method::CONNECT {
        let res = proxy_https(req, hostname, proxy, wasi_runtime, context).await;
        tracing::info!(?res, "HTTPS proxy result.");
        Ok(Response::new(hyper::Body::empty()))
    } else {
        let res = proxy_http(req, hostname, proxy, wasi_runtime, context).await;
        tracing::info!(?res, "HTTP proxy result.");
        res
    }
}

pub async fn http_forward(
    socket: tokio::net::TcpStream,
    proxy: Proxy,
    wasi_runtime: WasiRuntime,
    context: HttpContext,
) -> Result<()> {
    let service = service_fn(|request: Request<Body>| {
        let wasi_runtime = wasi_runtime.clone();
        let context = context.clone();
        let proxy = proxy.clone();
        async move { http_forward_proxy_service(request, proxy, wasi_runtime, context).await }
    });

    if let Err(http_err) = Http::new()
        .serve_connection(socket, service)
        .with_upgrades()
        .await
    {
        tracing::error!(%http_err, "Error while serving HTTP connection");
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::http::proxy::process_response;

    use super::{process_request, WasiRuntime};
    use config::Proxy;
    use http::{Response, Uri};
    use hyper::{Body, Request};

    #[tokio::test]
    async fn processes_request() {
        let request = Request::builder()
            .method("get")
            .body(Body::from("hello"))
            .expect("should build the request");

        let mut wasi_path = std::env::current_dir().expect("should get the current directory");
        wasi_path
            .push("../wit-bindings/tests/http-request/target/wasm32-wasi/debug/http-request.wasm");
        let mut wasi_runtime =
            WasiRuntime::new(PathBuf::from("/")).expect("should build the runtime");
        let new_request = process_request(
            &mut wasi_runtime,
            request,
            Some(wasi_path),
            "http",
            "localhost",
            Proxy::new(),
        )
        .await
        .expect("should process the request");

        let (parts, body) = new_request.into_parts();
        assert_eq!(parts.method, "post");
        let body = hyper::body::to_bytes(body)
            .await
            .expect("should read the body");
        let body_str = std::str::from_utf8(&body).expect("should convert to text");
        assert_eq!(body_str, "haha!");
    }

    #[tokio::test]
    async fn processes_response() {
        let response = Response::builder()
            .status(200)
            .body(Body::from("hello"))
            .expect("should build the response");

        let mut wasi_path = std::env::current_dir().expect("should get the current directory");
        wasi_path.push(
            "../wit-bindings/tests/http-response/target/wasm32-wasi/debug/http-response.wasm",
        );
        let mut wasi_runtime =
            WasiRuntime::new(PathBuf::from("/")).expect("should build the runtime");
        let proxy = Proxy::new();
        let uri = Uri::builder()
            .scheme("https")
            .authority("jaksf.com")
            .path_and_query("/")
            .build()
            .expect("");
        let req = hyper::Request::builder()
            .uri(uri)
            .method("HEAD")
            .body(Body::from(""))
            .expect("");
        let uri = req.uri().clone();
        let version = req.version();
        let method = req.method().clone();

        let new_response: Response<Body> = process_response(
            &mut wasi_runtime,
            response,
            Some(wasi_path),
            proxy,
            uri,
            version,
            method,
        )
        .await
        .expect("should process the response");

        let (parts, body) = new_response.into_parts();
        assert_eq!(parts.status, 500);
        let body = hyper::body::to_bytes(body)
            .await
            .expect("should read the body");
        let body_str = std::str::from_utf8(&body).expect("should convert to text");
        assert_eq!(body_str, "broken!");
    }
}
