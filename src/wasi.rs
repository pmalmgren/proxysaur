use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use tokio::sync::RwLock;
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

#[derive(Clone)]
pub struct WasiRuntime {
    engine: Engine,
    module_cache: Arc<RwLock<HashMap<PathBuf, Module>>>,
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

    pub async fn process_request(&mut self, buf: Vec<u8>, path: &Path) -> Result<Vec<u8>> {
        let mut linker: Linker<WasiCtx> = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()?
            .build();
        let mut store = Store::new(&self.engine, wasi);
        let module = self.fetch_module(path).await?;
        linker.module(&mut store, "", &module)?;
        linker
            .get_default(&mut store, "")?
            .typed::<(), (), _>(&store)?
            .call(&mut store, ())?;

        Ok(buf)
    }
}
