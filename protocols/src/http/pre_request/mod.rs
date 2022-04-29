use std::path::PathBuf;

use anyhow::Result;
use config::Proxy;
use proxysaur_wit_bindings::http::pre_request;
use proxysaur_wit_bindings::config::config::add_to_linker;
pub use proxysaur_wit_bindings::http::pre_request::ProxyMode;
use wasi_runtime::{Linker, Store, WasiCtx, WasiCtxBuilder, WasiRuntime};

use super::{hostname::Hostname, config::ProxyConfig};

#[derive(Debug)]
pub struct ProxyHttpPreRequest {
    request: pre_request::HttpPreRequest,
    mode: pre_request::ProxyMode,
}

impl pre_request::PreRequest for ProxyHttpPreRequest {
    fn http_request_get(&mut self) -> pre_request::HttpPreRequest {
        self.request.clone()
    }

    fn http_set_proxy_mode(&mut self, mode: pre_request::ProxyMode) {
        self.mode = mode;
    }
}

impl ProxyHttpPreRequest {
    pub fn new(hostname: Hostname) -> Self {
        let request = pre_request::HttpPreRequest {
            path: "/".into(),
            authority: hostname.authority,
            host: hostname.host,
            scheme: hostname.scheme,
        };
        Self {
            request,
            mode: pre_request::ProxyMode::Pass,
        }
    }
}

struct PreRequestContext {
    wasi: WasiCtx,
    proxy_request: ProxyHttpPreRequest,
    proxy_config: ProxyConfig
}

pub async fn process_pre_request(
    wasi_runtime: &mut WasiRuntime,
    hostname: Hostname,
    wasi_module_path: Option<PathBuf>,
    proxy: Proxy
) -> Result<pre_request::ProxyMode> {
    let wasi_module_path = match wasi_module_path {
        Some(wasi_module_path) => wasi_module_path,
        None => {
            return Ok(pre_request::ProxyMode::Pass);
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
        proxy_config: ProxyConfig { proxy, error: "".into() }
    };

    let mut store: Store<PreRequestContext> = Store::new(&wasi_runtime.engine, ctx);
    wasi_runtime::add_to_linker(&mut linker, |s| &mut s.wasi)?;
    tracing::trace!("Linked module.");

    pre_request::add_to_linker(&mut linker, |ctx| -> &mut ProxyHttpPreRequest {
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
    let proxy_mode = data.proxy_request.mode;
    Ok(proxy_mode)
}
