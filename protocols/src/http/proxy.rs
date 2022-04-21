use std::{convert::Infallible, path::Path};

use anyhow::Result;
use http::{Request, Response, StatusCode};
use hyper::{server::conn::Http, service::service_fn, Body};
use wasi_runtime::{Linker, Store, WasiCtx, WasiCtxBuilder, WasiRuntime};

use crate::http::{add_to_linker, ProxyHttpRequest};

struct Context {
    wasi: WasiCtx,
    proxy_request: ProxyHttpRequest,
}

pub async fn process_request(
    wasi_runtime: &mut WasiRuntime,
    req: Request<Body>,
    wasi_module_path: &Path,
) -> Result<Request<Body>> {
    let proxy_request = ProxyHttpRequest::new(req).await?;
    let module = wasi_runtime.fetch_module(wasi_module_path).await?;

    let mut linker: Linker<Context> = Linker::new(&wasi_runtime.engine);
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()?
        .build();
    let ctx = Context {
        wasi,
        proxy_request,
    };

    let mut store: Store<Context> = Store::new(&wasi_runtime.engine, ctx);
    wasi_runtime::add_to_linker(&mut linker, |s| &mut s.wasi)?;

    add_to_linker(&mut linker, |ctx| -> &mut ProxyHttpRequest {
        &mut ctx.proxy_request
    })?;

    linker.module(&mut store, "", &module)?;
    linker
        .get_default(&mut store, "")?
        .typed::<(), (), _>(&store)?
        .call(&mut store, ())?;

    let data = store.into_data();
    let new_request: Request<Body> = Request::try_from(data.proxy_request)?;
    Ok(new_request)
}

async fn http_proxy_service(
    req: Request<Body>,
    wasi_module_path: &Path,
    mut wasi_runtime: WasiRuntime,
) -> Result<Response<Body>, Infallible> {
    let _request = match process_request(&mut wasi_runtime, req, wasi_module_path).await {
        Ok(request) => {
            tracing::info!(new_request = ?request, "New request.");
            request
        }
        Err(err) => {
            let payload = format!("Error making request: {err}");
            tracing::error!(?err, "Error making new request.");
            let mut resp = Response::new(Body::from(payload));
            *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(resp);
        }
    };

    Ok(Response::new(Body::from("hello")))
}

pub async fn http_proxy(
    socket: tokio::net::TcpStream,
    wasi_module_path: &Path,
    wasi_runtime: WasiRuntime,
) -> Result<()> {
    let service = service_fn(|request: Request<Body>| {
        let path = wasi_module_path.to_path_buf();
        let wasi_runtime = wasi_runtime.clone();
        async move { http_proxy_service(request, &path, wasi_runtime).await }
    });

    if let Err(http_err) = Http::new().serve_connection(socket, service).await {
        tracing::error!(%http_err, "Error while serving HTTP connection");
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::{process_request, WasiRuntime};
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
        let new_request = process_request(&mut wasi_runtime, request, &wasi_path)
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
}
