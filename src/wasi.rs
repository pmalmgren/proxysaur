use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use hyper::{Body, Request};
use protocol_interfaces::http::ProxyHttpRequest;
use tokio::sync::RwLock;
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

use crate::config::Proxy;

#[derive(Clone)]
pub struct WasiRuntime {
    engine: Engine,
    module_cache: Arc<RwLock<HashMap<PathBuf, Module>>>,
}

struct Context {
    wasi: WasiCtx,
    proxy_request: ProxyHttpRequest,
}

impl WasiRuntime {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        let config = config.async_support(true).epoch_interruption(true);
        Ok(Self {
            engine: Engine::new(config)?,
            module_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn fetch_module(&mut self, path: &Path) -> Result<Module> {
        let module = {
            let cache = self.module_cache.read().await;
            cache.get(&path.to_path_buf()).cloned()
        };

        match module {
            Some(module) => Ok(module),
            None => {
                let contents = tokio::fs::read(path).await?;
                let module = Module::new(&self.engine, contents)?;
                let mut cache = self.module_cache.write().await;
                cache.insert(path.to_path_buf(), module.clone());
                Ok(module)
            }
        }
    }

    pub async fn process_request(
        &mut self,
        req: Request<Body>,
        proxy: Proxy,
    ) -> Result<Request<Body>> {
        let proxy_request = ProxyHttpRequest::new(req).await?;
        let module = self.fetch_module(&proxy.wasi_module_path).await?;

        let mut linker: Linker<Context> = Linker::new(&self.engine);
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()?
            .build();
        let ctx = Context {
            wasi,
            proxy_request,
        };

        let mut store: Store<Context> = Store::new(&self.engine, ctx);
        wasmtime_wasi::add_to_linker(&mut linker, |s| &mut s.wasi)?;

        protocol_interfaces::http::add_to_linker(&mut linker, |ctx| -> &mut ProxyHttpRequest {
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
}
