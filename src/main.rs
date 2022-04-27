use anyhow::Result;
use config::{Args, Config};

mod proxy;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt::Subscriber::new();
    tracing::subscriber::set_global_default(subscriber).expect("should set subscriber");

    let args = Args::new();

    match args.commands {
        Some(config::Commands::GenerateCa { path, force }) => {
            let res = ca::cli::generate_ca(path, force).await?;
            let path = match res.to_str() {
                Some(path) => path.to_string(),
                None => {
                    let path = format!("{:?}", res);
                    let path = path
                        .strip_prefix('"')
                        .unwrap_or("")
                        .strip_suffix('"')
                        .unwrap_or("")
                        .to_string();
                    path
                }
            };
            eprintln!("Go to the docs page to see how to trust this CA: https://proxysaur.us/docs");
            println!("{}", path);
            return Ok(());
        }
        Some(config::Commands::Init { path }) => {
            let _res = config::cli::init(path)?;
            return Ok(());
        }
        Some(config::Commands::AddProxy { path }) => {
            let _res = config::cli::add_proxy(path)?;
            return Ok(());
        }
        None => {}
    };

    let config = Config::try_from(args)?;

    proxy::run(config).await
}
