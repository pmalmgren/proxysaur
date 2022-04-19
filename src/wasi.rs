use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::Result;
use tokio::sync::RwLock;
use wasmtime::{Config, Engine, Module};

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

    pub async fn fetch_module(&mut self, path: PathBuf, engine: &Engine) -> Result<Module> {
        let (module, hit) = {
            let cache = self.module_cache.read().await;
            match cache.get(&path) {
                Some(module) => (module.clone(), true),
                None => {
                    let contents = tokio::fs::read(path.clone()).await?;
                    let module = Module::new(engine, contents)?;
                    (module, false)
                }
            }
        };

        if !hit {
            let mut cache = self.module_cache.write().await;
            cache.insert(path, module.clone());
        }

        Ok(module)
    }
}
