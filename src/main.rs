use anyhow::Result;
use config::{Args, Config};

mod proxy;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt::Subscriber::new();
    tracing::subscriber::set_global_default(subscriber).expect("should set subscriber");

    let args = Args::new();

    match args.commands {
        Some(config::Commands::GenerateCa { ref path }) => {
            let _res = ca::cli::generate_ca(path.clone()).await?;
            return Ok(());
        }
        None => {}
    };

    let config = Config::try_from(args)?;

    proxy::run(config).await
}
