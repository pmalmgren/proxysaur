use anyhow::Result;
use config::Config;

mod proxy;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt::Subscriber::new();
    tracing::subscriber::set_global_default(subscriber).expect("should set subscriber");

    let config = Config::try_parse()?;

    proxy::run(config).await
}
