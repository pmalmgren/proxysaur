use anyhow::Result;
use clap::StructOpt;
use config::{Args, Config};
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

async fn process(mut socket: TcpStream) {
    let mut buf = vec![0; 1024];
    match socket.read(&mut buf).await {
        Ok(nread) => {
            eprintln!("Read bytes: {nread}");
            respond(&mut socket).await;
        }
        Err(err) => eprintln!("Error reading bytes: {err}"),
    }
}

async fn listen(listener: TcpListener) {
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::try_from(args)?;

    let futures = config
        .proxy
        .into_iter()
        .map(|proxy| async move { TcpListener::bind(proxy.address()).await });

    let listeners = try_join_all(futures).await?;

    let _handle =
        join_all(listeners.into_iter().map(|listener| async move {
            tokio::spawn(async move { listen(listener).await }).await
        }))
        .await;

    Ok(())
}
