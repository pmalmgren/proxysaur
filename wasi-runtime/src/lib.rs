use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::RwLock;

use anyhow::Result;
pub use wasmtime::{Config, Engine, Linker, Module, Store};
pub use wasmtime_wasi::add_to_linker;
pub use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

#[derive(Clone)]
pub struct WasiRuntime {
    pub engine: Engine,
    module_cache: Arc<RwLock<HashMap<PathBuf, Module>>>,
}

impl WasiRuntime {
    pub fn new() -> Result<Self> {
        let config = Config::new();
        Ok(Self {
            engine: Engine::new(&config)?,
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
                let contents = match tokio::fs::read(path).await {
                    Ok(contents) => Ok(contents),
                    Err(err) => {
                        println!("Error reading file: {err}");
                        Err(err)
                    }
                }?;
                let module = Module::new(&self.engine, contents)?;
                let mut cache = self.module_cache.write().await;
                cache.insert(path.to_path_buf(), module.clone());
                Ok(module)
            }
        }
    }
}
