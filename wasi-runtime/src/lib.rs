use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
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
    cache_dir: PathBuf,
}

impl WasiRuntime {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        let config = Config::new();
        Ok(Self {
            engine: Engine::new(&config)?,
            module_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_dir,
        })
    }

    fn module_cache_path_for_path(&self, path: &Path) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let value = hasher.finish();
        let last_component = format!("{}.wasmtime", value);
        self.cache_dir.join(last_component)
    }

    pub async fn fetch_module(&mut self, path: &Path) -> Result<Module> {
        let module = {
            let cache = self.module_cache.read().await;
            cache.get(&path.to_path_buf()).cloned()
        };

        match module {
            Some(module) => Ok(module),
            None => {
                let module_cache_path = self.module_cache_path_for_path(path);

                if let Ok(module) = unsafe {
                    let engine = self.engine.clone();
                    let module_cache_path = module_cache_path.clone();
                    tokio::task::spawn_blocking(move || {
                        Module::deserialize_file(&engine, &module_cache_path)
                    })
                    .await?
                } {
                    return Ok(module);
                }

                let contents = match tokio::fs::read(path).await {
                    Ok(contents) => Ok(contents),
                    Err(err) => {
                        println!("Error reading file: {err}");
                        Err(err)
                    }
                }?;
                let module = Module::new(&self.engine, contents)?;

                let module_ = module.clone();
                let module_cache_path_ = module_cache_path.clone();
                tokio::task::spawn_blocking(move || {
                    if let Ok(bytes) = module_.serialize() {
                        let _res = std::fs::write(module_cache_path_, bytes);
                    }
                })
                .await?;

                {
                    let mut cache = self.module_cache.write().await;
                    cache.insert(path.to_path_buf(), module.clone());
                }
                Ok(module)
            }
        }
    }
}
