use std::{convert::Infallible, path::Path};

use anyhow::Result;
use http::{Request, Response, StatusCode, Uri, Version};
use hyper::{client::HttpConnector, server::conn::Http, service::service_fn, Body};
use hyper_alpn::AlpnConnector;
use hyper_tls::HttpsConnector;
use wasi_runtime::{Linker, Store, WasiCtx, WasiCtxBuilder, WasiRuntime};

use crate::http::{request_add_to_linker, response_add_to_linker, ProxyHttpRequest};

use super::ProxyHttpResponse;

// Each protocol defines a context, and is passed in via process request
#[derive(Clone)]
pub struct HttpContext {
    client_h1: hyper::Client<HttpsConnector<HttpConnector>, hyper::Body>,
    client_h2: hyper::Client<AlpnConnector, hyper::Body>,
}

impl HttpContext {
    pub fn new() -> HttpContext {
        let alpn = AlpnConnector::new();
        let client_h2 = hyper::Client::builder()
            .http2_only(true)
            .build::<_, hyper::Body>(alpn);

        let https = HttpsConnector::new();
        let client_h1 = hyper::Client::builder().build::<_, hyper::Body>(https);

        Self {
            client_h1,
            client_h2,
        }
    }
}

impl Default for HttpContext {
    fn default() -> Self {
        Self::new()
    }
}

struct RequestContext {
    wasi: WasiCtx,
    proxy_request: ProxyHttpRequest,
}

struct ResponseContext {
    wasi: WasiCtx,
    proxy_response: ProxyHttpResponse,
}

async fn process_request(
    wasi_runtime: &mut WasiRuntime,
    req: Request<Body>,
    wasi_module_path: &Path,
    scheme: &str,
    host: &str,
) -> Result<Request<Body>> {
    tracing::info!("Building request.");
    let proxy_request = ProxyHttpRequest::new(req, scheme, host).await?;
    tracing::info!(?proxy_request, "Built request.");
    let module = wasi_runtime.fetch_module(wasi_module_path).await?;

    let mut linker: Linker<RequestContext> = Linker::new(&wasi_runtime.engine);
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()?
        .build();
    tracing::info!("Built WASI context.");
    let ctx = RequestContext {
        wasi,
        proxy_request,
    };

    let mut store: Store<RequestContext> = Store::new(&wasi_runtime.engine, ctx);
    wasi_runtime::add_to_linker(&mut linker, |s| &mut s.wasi)?;
    tracing::info!("Linked module.");

    request_add_to_linker(&mut linker, |ctx| -> &mut ProxyHttpRequest {
        &mut ctx.proxy_request
    })?;
    tracing::info!("Linked module with WIT.");

    linker.module(&mut store, "", &module)?;
    tracing::info!("Added module to linker.");
    linker
        .get_default(&mut store, "")?
        .typed::<(), (), _>(&store)?
        .call(&mut store, ())?;
    tracing::info!("Called WASI module.");

    let data = store.into_data();
    tracing::info!("Fetched request context from store.");
    let new_request: Request<Body> = Request::try_from(data.proxy_request)?;
    tracing::info!(?new_request, "Built new request.");
    Ok(new_request)
}

async fn process_response(
    wasi_runtime: &mut WasiRuntime,
    resp: Response<Body>,
    wasi_module_path: &Path,
) -> Result<Response<Body>> {
    let proxy_response = ProxyHttpResponse::new(resp).await?;
    let module = wasi_runtime.fetch_module(wasi_module_path).await?;

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

async fn negotiate_version(scheme: String, host: String, context: &HttpContext) -> Result<Version> {
    let uri = Uri::builder()
        .scheme(scheme.as_str())
        .authority(host)
        .path_and_query("/")
        .build()?;
    let head_req = hyper::Request::builder()
        .uri(uri)
        .method("HEAD")
        .body(Body::from(""))?;
    tracing::info!(?head_req, "Built HEAD request for negotiation");
    match context.client_h2.request(head_req).await {
        Ok(resp) => {
            tracing::debug!(?resp, "Received HEAD response from server.");
            Ok(resp.version())
        }
        Err(error) => {
            tracing::warn!(%error, "Received error when connecting to client. Assuming HTTP/1.1");
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
    req: Request<Body>,
    req_wasi_module_path: &Path,
    resp_wasi_module_path: &Path,
    scheme: String,
    host: String,
    mut wasi_runtime: WasiRuntime,
    context: HttpContext,
) -> Result<Response<Body>, Infallible> {
    let request =
        match process_request(&mut wasi_runtime, req, req_wasi_module_path, &scheme, &host).await {
            Ok(request) => {
                tracing::info!(new_request = ?request, "New request.");
                request
            }
            Err(err) => {
                tracing::error!(?err, "Error getting request from WASM.");
                return Ok(error_payload(err));
            }
        };
    let version = match negotiate_version(scheme, host, &context).await {
        Ok(version) => version,
        Err(err) => {
            tracing::error!(?err, "Error negotiating HTTP version.");
            return Ok(error_payload(err));
        }
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

    match process_response(&mut wasi_runtime, resp, resp_wasi_module_path).await {
        Ok(resp) => Ok(resp),
        Err(err) => {
            tracing::error!(?err, "Error processing respones from WASM.");
            Ok(error_payload(err))
        }
    }
}

pub async fn http_proxy(
    socket: tokio::net::TcpStream,
    request_wasi_module_path: &Path,
    response_wasi_module_path: &Path,
    scheme: String,
    host: String,
    wasi_runtime: WasiRuntime,
    context: HttpContext,
) -> Result<()> {
    let service = service_fn(|request: Request<Body>| {
        let req_path = request_wasi_module_path.to_path_buf();
        let resp_path = response_wasi_module_path.to_path_buf();
        let wasi_runtime = wasi_runtime.clone();
        let context = context.clone();
        let scheme = scheme.clone();
        let host = host.clone();
        async move {
            http_proxy_service(
                request,
                &req_path,
                &resp_path,
                scheme,
                host,
                wasi_runtime,
                context,
            )
            .await
        }
    });

    if let Err(http_err) = Http::new().serve_connection(socket, service).await {
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
        let new_request =
            process_request(&mut wasi_runtime, request, &wasi_path, "http", "localhost")
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
        let new_response = process_response(&mut wasi_runtime, response, &wasi_path)
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
