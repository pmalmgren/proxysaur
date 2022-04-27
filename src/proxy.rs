use anyhow::Result;
use futures::future::{join_all, try_join_all};
use protocols::http::proxy::{http_forward, http_proxy, HttpContext};
use protocols::tcp::tunnel;
use tokio::net::{TcpListener, TcpStream};
use wasi_runtime::WasiRuntime;

use config::{Config, Protocol, Proxy};

pub async fn run(config: Config) -> Result<()> {
    let ca_path = match config.ca_path {
        Some(ca_path) => ca_path,
        // the default CA dir uses XDG directories
        None => ca::default_ca_dir()?,
    };
    let futures = config
        .proxy
        .into_iter()
        .map(|proxy| async move { bind(proxy).await });

    let listeners = try_join_all(futures).await?;

    let http_context = HttpContext::new(ca_path).await?;
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
