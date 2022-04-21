use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use hyper::{Body, Request};
use protocols::http::ProxyHttpRequest;
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
        let config = Config::new();
        Ok(Self {
            engine: Engine::new(&config)?,
            module_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn fetch_module(&mut self, path: &Path) -> Result<Module> {
        println!("path = {:?}", path);
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

        protocols::http::add_to_linker(&mut linker, |ctx| -> &mut ProxyHttpRequest {
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

#[cfg(test)]
mod http_test {
    use hyper::{Body, Request};

    use crate::config::{Protocol, Proxy};

    use super::WasiRuntime;

    #[tokio::test]
    async fn processes_request() {
        let request = Request::builder()
            .method("get")
            .body(Body::from("hello"))
            .expect("should build the request");

        let mut wasi_path = std::env::current_dir().expect("should get the current directory");
        wasi_path.push("src/tests/http-request/target/wasm32-wasi/debug/http-request.wasm");
        let mut wasi_runtime = WasiRuntime::new().expect("should build the runtime");
        let proxy = Proxy::new_for_test(
            wasi_path,
            8080,
            Protocol::Http,
            "google.com".into(),
            "google.com".into(),
            8080,
        );
        let new_request = wasi_runtime
            .process_request(request, proxy)
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
