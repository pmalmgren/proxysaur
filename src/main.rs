use anyhow::Result;
use clap::StructOpt;
use config::{Args, Config, Proxy};
use futures::future::{join_all, try_join_all};
use tokio::net::{TcpListener, TcpStream};

mod config;

async fn tunnel(socket: &mut TcpStream, upstream: &mut TcpStream) -> Result<()> {
    tokio::io::copy_bidirectional(upstream, socket)
        .await
        .map(|_| ())
        .map_err(anyhow::Error::from)
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
                eprintln!("Error proxying the connection: {err}");
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::try_from(args)?;

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
