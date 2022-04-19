use anyhow::Result;
use clap::StructOpt;
use config::{Args, Config};

mod config;
mod proxy;
mod wasi;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt::Subscriber::new();
    tracing::subscriber::set_global_default(subscriber).expect("should set subscriber");

    let args = Args::parse();
    let config = Config::try_from(args)?;

    proxy::run(config).await
}
