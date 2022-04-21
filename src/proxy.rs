use anyhow::Result;
use futures::future::{join_all, try_join_all};
use protocols::http::proxy::http_proxy;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use wasi_runtime::WasiRuntime;

use crate::config::{Config, Protocol, Proxy};

pub async fn run(config: Config) -> Result<()> {
    let futures = config
        .proxy
        .into_iter()
        .map(|proxy| async move { bind(proxy).await });

    let listeners = try_join_all(futures).await?;

    let wasi_runtime = WasiRuntime::new()?;

    let _handle = join_all(
        listeners
            .into_iter()
            .map(|(listener, proxy)| (listener, proxy, wasi_runtime.clone()))
            .map(|(listener, proxy, wasi_runtime)| async move {
                listen(listener, proxy, wasi_runtime).await
            }),
    )
    .await;

    Ok(())
}

async fn tunnel(
    socket: &mut TcpStream,
    upstream: &mut TcpStream,
    _proxy: Proxy,
    mut _wasi_runtime: WasiRuntime,
) -> Result<()> {
    let (mut server_rh, mut server_wh) = upstream.split();
    let (mut client_rh, mut client_wh) = tokio::io::split(socket);

    tokio::join! {
        async {
            loop {
                let mut buf: Vec<u8> = vec![0; 2056];
                let bytes_read = match server_rh.read(&mut buf).await {
                    Ok(n_bytes) => n_bytes,
                    Err(error) => {
                        tracing::error!(%error, "Error reading bytes from server");
                        break;
                    },
                };
                if bytes_read == 0 {
                    tracing::debug!("Detected EOF from server.");
                    break;
                }
                match client_wh.write_all(&buf[0..bytes_read]).await {
                    Ok(_) => {},
                    Err(error) => {
                        tracing::error!(%error, "Error writing bytes to client.");
                        break;
                    }
                };
            }
        },
        async {
            loop {
                let mut buf: Vec<u8> = vec![0; 2056];
                let bytes_read = match client_rh.read(&mut buf).await {
                    Ok(n_bytes) => n_bytes,
                    Err(error) => {
                        tracing::error!(%error, "Error reading bytes from client.");
                        break;
                    },
                };
                if bytes_read == 0 {
                    tracing::debug!("Detected EOF from client.");
                    break;
                }
                match server_wh.write_all(&buf).await {
                    Ok(_) => {},
                    Err(error) => {
                        tracing::error!(%error, "Error writing bytes to server.");
                        break;
                    }
                };
            }
        }
    };

    Ok(())
}

async fn proxy_conn(mut socket: TcpStream, proxy: Proxy, wasi_runtime: WasiRuntime) -> Result<()> {
    match proxy.protocol {
        Protocol::Tcp => {
            let mut upstream = TcpStream::connect(&proxy.upstream_address()).await?;
            tunnel(&mut socket, &mut upstream, proxy, wasi_runtime).await
        }
        Protocol::Http => http_proxy(socket, &proxy.wasi_module_path, wasi_runtime).await,
    }
}

async fn listen(listener: TcpListener, proxy: Proxy, wasi_runtime: WasiRuntime) {
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let proxy = proxy.clone();
        let wasi_runtime = wasi_runtime.clone();
        tokio::spawn(async move {
            if let Err(err) = proxy_conn(socket, proxy, wasi_runtime).await {
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
