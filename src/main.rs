use anyhow::Result;
use clap::StructOpt;
use config::{Args, Config, Proxy};
use futures::future::{join_all, try_join_all};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

mod config;

async fn respond(socket: &mut TcpStream) {
    let resp = "goodbye".as_bytes();
    match socket.write(resp).await {
        Ok(nwrote) => eprintln!("Wrote bytes: {nwrote}"),
        Err(err) => eprintln!("Error writing bytes: {err}"),
    };
}

async fn proxy_conn(mut socket: TcpStream) {
    let mut buf = vec![0; 1024];
    match socket.read(&mut buf).await {
        Ok(nread) => {
            eprintln!("Read bytes: {nread}");
            respond(&mut socket).await;
        }
        Err(err) => eprintln!("Error reading bytes: {err}"),
    }
}

async fn listen(listener: TcpListener, _proxy: Proxy) {
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            proxy_conn(socket).await;
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
