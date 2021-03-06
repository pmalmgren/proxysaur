use std::path::PathBuf;

use anyhow::Result;
use config::Proxy;
use http::{Method, Uri, Version};
use hyper::{Body, Response};
use proxysaur_wit_bindings::config::config::add_to_linker;
use proxysaur_wit_bindings::http::response;
use wasi_runtime::{Linker, Store, WasiCtx, WasiCtxBuilder, WasiRuntime};

use super::{config::ProxyConfig, ProxyHttpError};

pub struct ProxyHttpResponse {
    response: response::HttpResponse,
}

impl TryFrom<ProxyHttpResponse> for Response<Body> {
    type Error = ProxyHttpError;

    fn try_from(value: ProxyHttpResponse) -> Result<Self, Self::Error> {
        let resp = Response::builder()
            .status(value.response.status)
            .body(Body::from(value.response.body))?;
        Ok(resp)
    }
}

impl ProxyHttpResponse {
    pub async fn new(
        response: Response<Body>,
        uri: Uri,
        version: Version,
        method: Method,
    ) -> Result<Self, ProxyHttpError> {
        let (parts, body) = response.into_parts();
        let headers = parts
            .headers
            .iter()
            .flat_map(|(name, value)| match value.to_str() {
                Ok(value) => Some((name.to_string(), value.to_string())),
                Err(_) => None,
            })
            .collect();
        let body = hyper::body::to_bytes(body).await?.to_vec();
        let response = response::HttpResponse {
            headers,
            status: parts.status.as_u16(),
            body,
            request_path: uri
                .path_and_query()
                .map(|pq| pq.to_string())
                .unwrap_or_else(|| "".to_string()),
            request_authority: uri
                .authority()
                .map(|authority| authority.as_str())
                .unwrap_or_else(|| "")
                .to_string(),
            request_host: uri.host().unwrap_or("https").to_string(),
            request_scheme: uri.scheme_str().unwrap_or("https").to_string(),
            request_version: format!("{:?}", version),
            request_headers: vec![],
            request_method: method.to_string(),
        };

        Ok(Self { response })
    }
}

impl response::Response for ProxyHttpResponse {
    fn http_response_get(&mut self) -> Result<response::HttpResponse, response::Error> {
        Ok(self.response.clone())
    }

    fn http_response_set_status(&mut self, status: u16) -> Result<(), response::Error> {
        self.response.status = status;
        Ok(())
    }

    fn http_response_set_body(
        &mut self,
        body: response::BodyParam<'_>,
    ) -> Result<(), response::Error> {
        self.response.body = body.to_vec();
        Ok(())
    }

    fn http_response_set_headers(
        &mut self,
        headers: response::HttpHeadersParam<'_>,
    ) -> Result<(), response::Error> {
        let headers: Vec<(String, String)> = headers
            .iter()
            .map(|(h, v)| (h.to_string(), v.to_string()))
            .collect();
        self.response.headers = headers;
        Ok(())
    }
}
struct ResponseContext {
    wasi: WasiCtx,
    proxy_response: ProxyHttpResponse,
    config: ProxyConfig,
}

pub async fn process_response(
    wasi_runtime: &mut WasiRuntime,
    resp: Response<Body>,
    wasi_module_path: Option<PathBuf>,
    proxy: Proxy,
    uri: Uri,
    version: Version,
    method: Method,
) -> Result<Response<Body>> {
    let wasi_module_path = match wasi_module_path {
        Some(path) => path,
        None => {
            return Ok(resp);
        }
    };
    let proxy_response = ProxyHttpResponse::new(resp, uri, version, method).await?;
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
        config: ProxyConfig {
            proxy,
            error: "".into(),
        },
    };

    let mut store: Store<ResponseContext> = Store::new(&wasi_runtime.engine, ctx);
    wasi_runtime::add_to_linker(&mut linker, |s| &mut s.wasi)?;

    response::add_to_linker(&mut linker, |ctx| -> &mut ProxyHttpResponse {
        &mut ctx.proxy_response
    })?;

    add_to_linker(&mut linker, |ctx| -> &mut ProxyConfig { &mut ctx.config })?;

    linker.module(&mut store, "", &module)?;
    linker
        .get_default(&mut store, "")?
        .typed::<(), (), _>(&store)?
        .call(&mut store, ())?;

    let data = store.into_data();
    let new_response: Response<Body> = Response::try_from(data.proxy_response)?;
    Ok(new_response)
}
