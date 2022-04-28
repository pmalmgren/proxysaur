use std::{convert::Infallible, path::PathBuf, sync::Arc};

use anyhow::Result;
use ca::CertificateAuthority;
use config::Proxy;
use http::{Request, Response, StatusCode, Uri, Version};
use hyper::{client::HttpConnector, server::conn::Http, service::service_fn, Body};
use hyper_alpn::AlpnConnector;
use hyper_tls::HttpsConnector;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::TlsAcceptor;
use wasi_runtime::{Linker, Store, WasiCtx, WasiCtxBuilder, WasiRuntime};

use crate::{
    http::{
        pre_request_add_to_linker, request_add_to_linker, response_add_to_linker,
        ProxyHttpPreRequest, ProxyHttpRequest,
    },
    tcp::tunnel,
};

use super::{hostname::Hostname, ProxyHttpResponse, ProxyMode};

// Each protocol defines a context, and is passed in via process request
#[derive(Clone)]
pub struct HttpContext {
    client_h1: hyper::Client<HttpsConnector<HttpConnector>, hyper::Body>,
    client_h2: hyper::Client<AlpnConnector, hyper::Body>,
    #[allow(unused)]
    ca: CertificateAuthority,
}

impl HttpContext {
    pub async fn new(ca_path: PathBuf) -> Result<HttpContext> {
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

struct RequestContext {
    wasi: WasiCtx,
    proxy_request: ProxyHttpRequest,
}

struct PreRequestContext {
    wasi: WasiCtx,
    proxy_request: ProxyHttpPreRequest,
}

struct ResponseContext {
    wasi: WasiCtx,
    proxy_response: ProxyHttpResponse,
}

async fn process_pre_request(
    wasi_runtime: &mut WasiRuntime,
    hostname: Hostname,
    wasi_module_path: Option<PathBuf>,
) -> Result<ProxyMode> {
    let wasi_module_path = match wasi_module_path {
        Some(wasi_module_path) => wasi_module_path,
        None => {
            return Ok(ProxyMode::Pass);
        }
    };

    tracing::trace!("Building request.");
    let proxy_request = ProxyHttpPreRequest::new(hostname);
    tracing::trace!(?proxy_request, "Built request.");
    let module = wasi_runtime
        .fetch_module(wasi_module_path.as_path())
        .await?;

    let mut linker: Linker<PreRequestContext> = Linker::new(&wasi_runtime.engine);
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()?
        .build();
    tracing::trace!("Built WASI context.");
    let ctx = PreRequestContext {
        wasi,
        proxy_request,
    };

    let mut store: Store<PreRequestContext> = Store::new(&wasi_runtime.engine, ctx);
    wasi_runtime::add_to_linker(&mut linker, |s| &mut s.wasi)?;
    tracing::trace!("Linked module.");

    pre_request_add_to_linker(&mut linker, |ctx| -> &mut ProxyHttpPreRequest {
        &mut ctx.proxy_request
    })?;
    tracing::trace!("Linked module with WIT.");

    linker.module(&mut store, "", &module)?;
    tracing::trace!("Added module to linker.");
    linker
        .get_default(&mut store, "")?
        .typed::<(), (), _>(&store)?
        .call(&mut store, ())?;
    tracing::trace!("Called WASI module.");

    let data = store.into_data();
    tracing::trace!("Fetched request context from store.");
    let proxy_mode = data.proxy_request.mode;
    Ok(proxy_mode)
}

async fn process_request(
    wasi_runtime: &mut WasiRuntime,
    req: Request<Body>,
    wasi_module_path: Option<PathBuf>,
    scheme: &str,
    host: &str,
) -> Result<Request<Body>> {
    let wasi_module_path = match wasi_module_path {
        Some(wasi_module_path) => wasi_module_path,
        None => {
            return Ok(req);
        }
    };

    tracing::trace!("Building request.");
    let proxy_request = ProxyHttpRequest::new(req, scheme, host).await?;
    tracing::trace!(?proxy_request, "Built request.");
    let module = wasi_runtime
        .fetch_module(wasi_module_path.as_path())
        .await?;

    let mut linker: Linker<RequestContext> = Linker::new(&wasi_runtime.engine);
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()?
        .build();
    tracing::trace!("Built WASI context.");
    let ctx = RequestContext {
        wasi,
        proxy_request,
    };

    let mut store: Store<RequestContext> = Store::new(&wasi_runtime.engine, ctx);
    wasi_runtime::add_to_linker(&mut linker, |s| &mut s.wasi)?;
    tracing::trace!("Linked module.");

    request_add_to_linker(&mut linker, |ctx| -> &mut ProxyHttpRequest {
        &mut ctx.proxy_request
    })?;
    tracing::trace!("Linked module with WIT.");

    linker.module(&mut store, "", &module)?;
    tracing::trace!("Added module to linker.");
    linker
        .get_default(&mut store, "")?
        .typed::<(), (), _>(&store)?
        .call(&mut store, ())?;
    tracing::trace!("Called WASI module.");

    let data = store.into_data();
    tracing::trace!("Fetched request context from store.");
    let new_request: Request<Body> = Request::try_from(data.proxy_request)?;
    tracing::trace!(?new_request, "Built new request.");
    Ok(new_request)
}

async fn process_response(
    wasi_runtime: &mut WasiRuntime,
    resp: Response<Body>,
    wasi_module_path: Option<PathBuf>,
) -> Result<Response<Body>> {
    let wasi_module_path = match wasi_module_path {
        Some(path) => path,
        None => {
            return Ok(resp);
        }
    };
    let proxy_response = ProxyHttpResponse::new(resp).await?;
    let module = wasi_runtime
        .fetch_module(wasi_module_path.as_path())
        .await?;

    let mut linker: Linker<ResponseContext> = Linker::new(&wasi_runtime.engine);
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()?
        .build();
    let ctx = ResponseContext {
        wasi,
        proxy_response,
    };

    let mut store: Store<ResponseContext> = Store::new(&wasi_runtime.engine, ctx);
    wasi_runtime::add_to_linker(&mut linker, |s| &mut s.wasi)?;

    response_add_to_linker(&mut linker, |ctx| -> &mut ProxyHttpResponse {
        &mut ctx.proxy_response
    })?;

    linker.module(&mut store, "", &module)?;
    linker
        .get_default(&mut store, "")?
        .typed::<(), (), _>(&store)?
        .call(&mut store, ())?;

    let data = store.into_data();
    let new_response: Response<Body> = Response::try_from(data.proxy_response)?;
    Ok(new_response)
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

    match process_response(&mut wasi_runtime, resp, resp_path).await {
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
                match process_pre_request(&mut wasi_runtime, hostname.clone(), path)
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
    match process_pre_request(&mut wasi_runtime, hostname.clone(), path)
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
    use crate::http::proxy::process_response;

    use super::{process_request, WasiRuntime};
    use http::Response;
    use hyper::{Body, Request};

    #[tokio::test]
    async fn processes_request() {
        let request = Request::builder()
            .method("get")
            .body(Body::from("hello"))
            .expect("should build the request");

        let mut wasi_path = std::env::current_dir().expect("should get the current directory");
        wasi_path.push("src/http/tests/http-request/target/wasm32-wasi/debug/http-request.wasm");
        let mut wasi_runtime = WasiRuntime::new().expect("should build the runtime");
        let new_request = process_request(
            &mut wasi_runtime,
            request,
            Some(wasi_path),
            "http",
            "localhost",
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
        wasi_path.push("src/http/tests/http-response/target/wasm32-wasi/debug/http-response.wasm");
        let mut wasi_runtime = WasiRuntime::new().expect("should build the runtime");
        let new_response = process_response(&mut wasi_runtime, response, Some(wasi_path))
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
