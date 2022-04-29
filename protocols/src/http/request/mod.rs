use std::path::PathBuf;

use anyhow::Result;
use config::Proxy;
use http::Uri;
use hyper::{Body, Request};
use proxysaur_wit_bindings::http::request;
use proxysaur_wit_bindings::config::config::add_to_linker;
use wasi_runtime::{Linker, Store, WasiCtx, WasiCtxBuilder, WasiRuntime};

use crate::http::convert_version;

use super::{ProxyHttpError, config::ProxyConfig};

#[derive(Debug)]
pub struct ProxyHttpRequest {
    request: request::HttpRequest,
}

impl TryFrom<ProxyHttpRequest> for Request<Body> {
    type Error = ProxyHttpError;

    fn try_from(req: ProxyHttpRequest) -> Result<Self, Self::Error> {
        let request = req.request;
        tracing::info!(?request, "Building the request.");
        let body = Body::from(request.body);
        let uri = Uri::builder()
            .authority(request.authority)
            .scheme(request.scheme.as_str())
            .path_and_query(request.path)
            .build()?;
        tracing::info!(?uri, "Built URI.");
        let request = Request::builder()
            .method(request.method.as_str())
            .version(convert_version(&request.version)?)
            .uri(uri)
            .body(body)
            .map_err(ProxyHttpError::from)?;
        tracing::info!(?request, "Built request.");

        Ok(request)
    }
}

impl ProxyHttpRequest {
    pub async fn new(
        req: http::Request<Body>,
        scheme: &str,
        authority: &str,
    ) -> Result<Self, ProxyHttpError> {
        let (parts, body) = req.into_parts();
        let uri = parts.uri;
        let host = uri.host().map(String::from).unwrap_or_else(String::new);
        let authority = uri
            .authority()
            .map(|auth| auth.to_string())
            .unwrap_or_else(|| String::from(authority));
        let path = uri.path().to_string();
        let scheme = uri
            .scheme()
            .map(|scheme| scheme.to_string())
            .unwrap_or_else(|| String::from(scheme));
        let method = parts.method.as_str().to_string();
        let version = format!("{:?}", parts.version);
        let headers = parts
            .headers
            .iter()
            .flat_map(|(name, value)| match value.to_str() {
                Ok(value) => Some((name.to_string(), value.to_string())),
                Err(_) => None,
            })
            .collect();
        let body = hyper::body::to_bytes(body).await?.to_vec();
        let request = request::HttpRequest {
            path,
            authority,
            scheme,
            version,
            headers,
            method,
            host,
            body,
        };
        Ok(Self { request })
    }
}

impl request::Request for ProxyHttpRequest {
    fn http_request_get(&mut self) -> Result<request::HttpRequest, request::Error> {
        Ok(self.request.clone())
    }

    fn http_request_set_method(&mut self, method: &str) -> Result<(), request::Error> {
        self.request.method = method.into();
        Ok(())
    }

    fn http_request_set_header(&mut self, header: &str, value: &str) -> Result<(), request::Error> {
        match self
            .request
            .headers
            .iter()
            .enumerate()
            .find(|(_idx, (name, _value))| name == header)
        {
            Some((idx, _)) => {
                self.request.headers[idx].1 = value.to_string();
            }
            None => self
                .request
                .headers
                .push((header.to_string(), value.to_string())),
        };
        Ok(())
    }

    fn http_request_set_uri(&mut self, uri: &str) -> Result<(), request::Error> {
        let uri = Uri::try_from(uri).map_err(|err| format!("Invalid uri: {err}"))?;
        self.request.host = uri.host().map(String::from).unwrap_or_else(String::new);
        self.request.authority = uri
            .authority()
            .map(|auth| auth.to_string())
            .unwrap_or_else(|| String::from(""));
        self.request.path = uri.path().to_string();
        self.request.scheme = uri
            .scheme()
            .map(|scheme| scheme.to_string())
            .unwrap_or_else(|| String::from(""));
        Ok(())
    }

    fn http_request_set_version(&mut self, version: &str) -> Result<(), request::Error> {
        match version {
            "HTTP/0.9" | "HTTP/1.0" | "HTTP/1.1" | "HTTP/2.0" | "HTTP/3.0" => Ok(()),
            _ => Err(format!("Invalid version: {version}")),
        }?;

        self.request.version = version.to_string();

        Ok(())
    }

    fn http_request_rm_header(&mut self, header: &str) -> Result<(), request::Error> {
        if let Some((idx, _)) = self
            .request
            .headers
            .iter()
            .enumerate()
            .find(|(_idx, (name, _value))| name == header)
        {
            self.request.headers.remove(idx);
        }
        Ok(())
    }

    fn http_request_set_body(
        &mut self,
        body: request::BodyParam<'_>,
    ) -> Result<(), request::Error> {
        self.request.body = body.to_vec();
        Ok(())
    }
}

struct RequestContext {
    wasi: WasiCtx,
    proxy_request: ProxyHttpRequest,
    proxy_config: ProxyConfig,
}

pub async fn process_request(
    wasi_runtime: &mut WasiRuntime,
    req: Request<Body>,
    wasi_module_path: Option<PathBuf>,
    scheme: &str,
    host: &str,
    proxy: Proxy,
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
        proxy_config: ProxyConfig { proxy, error: "".into() },
    };

    let mut store: Store<RequestContext> = Store::new(&wasi_runtime.engine, ctx);
    wasi_runtime::add_to_linker(&mut linker, |s| &mut s.wasi)?;
    tracing::trace!("Linked module.");

    request::add_to_linker(&mut linker, |ctx| -> &mut ProxyHttpRequest {
        &mut ctx.proxy_request
    })?;
    add_to_linker(&mut linker, |ctx| -> &mut ProxyConfig {
        &mut ctx.proxy_config
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
