use anyhow::Result;
use futures::future::{join_all, try_join_all};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::config::{Config, Proxy};

pub async fn run(config: Config) -> Result<()> {
    let futures = config
        .proxy
        .into_iter()
        .map(|proxy| async move { bind(proxy).await });

    let listeners = try_join_all(futures).await?;

    let _handle = join_all(
        listeners
            .into_iter()
            .map(|(listener, proxy)| async move { listen(listener, proxy).await }),
    )
    .await;

    Ok(())
}

async fn tunnel(socket: &mut TcpStream, upstream: &mut TcpStream) -> Result<()> {
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
                match server_wh.write_all(&buf[0..bytes_read]).await {
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

async fn proxy_conn(mut socket: TcpStream, proxy: Proxy) -> Result<()> {
    let mut upstream = TcpStream::connect(&proxy.upstream_address()).await?;
    tunnel(&mut socket, &mut upstream).await
}

async fn listen(listener: TcpListener, proxy: Proxy) {
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let proxy = proxy.clone();
        tokio::spawn(async move {
            if let Err(err) = proxy_conn(socket, proxy).await {
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
