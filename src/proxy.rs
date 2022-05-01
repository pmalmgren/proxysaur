use anyhow::Result;
use futures::future::{join_all, try_join_all};
use protocols::http::proxy::{http_forward, http_proxy, HttpContext};
use protocols::tcp::tunnel;
use tokio::net::{TcpListener, TcpStream};
use wasi_runtime::WasiRuntime;

use config::{Config, Protocol, Proxy};

async fn add_default_http_proxy(proxy: &mut Proxy) -> Result<()> {
    let use_default = proxy.pre_request_wasi_module_path.is_none()
        && proxy.request_wasi_module_path.is_none()
        && proxy.response_wasi_module_path.is_none()
        && proxy.proxy_configuration_path.is_some();
    if !use_default {
        return Ok(());
    }

    let project_dirs = directories::ProjectDirs::from("com", "proxysaur", "proxysaur")
        .ok_or_else(|| anyhow::Error::msg("Could not build project dirs"))?;
    let cache_dir = project_dirs.cache_dir();

    let request_wasm_bytes =
        include_bytes!("../http-forward-proxy/target/wasm32-wasi/release/request.wasm");
    let pre_request_wasm_bytes =
        include_bytes!("../http-forward-proxy/target/wasm32-wasi/release/pre-request.wasm");
    let response_wasm_bytes =
        include_bytes!("../http-forward-proxy/target/wasm32-wasi/release/response.wasm");

    let pre_request_path = cache_dir.join("pre_request.wasm");
    let request_path = cache_dir.join("request.wasm");
    let response_path = cache_dir.join("response.wasm");

    tokio::fs::write(&pre_request_path, pre_request_wasm_bytes).await?;
    tokio::fs::write(&request_path, request_wasm_bytes).await?;
    tokio::fs::write(&response_path, response_wasm_bytes).await?;

    proxy.request_wasi_module_path = Some(request_path);
    proxy.pre_request_wasi_module_path = Some(pre_request_path);
    proxy.response_wasi_module_path = Some(response_path);

    Ok(())
}

async fn add_defaults(config: &mut Config) -> Result<()> {
    for proxy in config.proxy.iter_mut() {
        if proxy.protocol == Protocol::HttpForward {
            add_default_http_proxy(proxy).await?;
        }
    }
    Ok(())
}

pub async fn run(mut config: Config) -> Result<()> {
    let ca_path = match config.ca_path {
        Some(ref ca_path) => ca_path.to_path_buf(),
        // the default CA dir uses XDG directories
        None => ca::default_ca_dir()?,
    };
    add_defaults(&mut config).await?;
    println!("New config = {:?}", config);
    let futures = config
        .proxy
        .into_iter()
        .map(|proxy| async move { bind(proxy).await });

    let listeners = try_join_all(futures).await?;

    let http_context = HttpContext::new(ca_path.as_path()).await?;
    let wasi_runtime = WasiRuntime::new()?;

    let _handle = join_all(
        listeners
            .into_iter()
            .map(|(listener, proxy)| (listener, proxy, wasi_runtime.clone(), http_context.clone()))
            .map(|(listener, proxy, wasi_runtime, context)| async move {
                listen(listener, proxy, wasi_runtime, context).await
            }),
    )
    .await;

    Ok(())
}

async fn proxy_conn(
    mut socket: TcpStream,
    proxy: Proxy,
    wasi_runtime: WasiRuntime,
    context: HttpContext,
) -> Result<()> {
    match proxy.protocol {
        Protocol::Tcp => tunnel(&mut socket, &proxy.upstream_address()).await,
        Protocol::HttpForward => http_forward(socket, proxy, wasi_runtime, context).await,
        Protocol::Http => http_proxy(socket, proxy, wasi_runtime, context).await,
    }
}

async fn listen(
    listener: TcpListener,
    proxy: Proxy,
    wasi_runtime: WasiRuntime,
    context: HttpContext,
) {
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let proxy = proxy.clone();
        let wasi_runtime = wasi_runtime.clone();
        let context = context.clone();
        tokio::spawn(async move {
            if let Err(err) = proxy_conn(socket, proxy, wasi_runtime, context).await {
                tracing::error!(?err, "Error proxying the connection");
            }
        });
    }
}

async fn bind(proxy: Proxy) -> Result<(TcpListener, Proxy)> {
    TcpListener::bind(&proxy.address())
        .await
        .map(|listener| (listener, proxy))
        .map_err(anyhow::Error::from)
}
