use anyhow::Result;
use ca::init_project_dirs;
use config::{Args, Config, Protocol, Proxy};

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
            eprintln!("Go to the docs page to see how to trust this CA: https://proxysaur.us/ca#trusting-the-root-certificate-in-your-browser");
            println!("{}", path);
            return Ok(());
        }
        Some(config::Commands::Init { path }) => {
            let _res = config::cli::init(path)?;
            return Ok(());
        }
        Some(config::Commands::AddProxy { path }) => {
            let _res = config::cli::add_proxy(path).await?;
            return Ok(());
        }
        Some(config::Commands::Http {
            config_path,
            http_proxy_configuration_path,
            port,
        }) => {
            let project_dirs = init_project_dirs().await?;
            let default_config_path = project_dirs.config_dir().join("proxysaur.toml");
            let config_path = match config_path {
                Some(config_path) => config_path,
                None => match config::cli::init(Some(default_config_path)) {
                    Ok(path) => path,
                    Err(err) => {
                        eprintln!("Error generating or reading configuration: {err}");
                        return Err(err);
                    }
                },
            };

            let mut config = Config::try_from(config_path.as_path())?;

            let ca_path = match ca::cli::generate_ca(config.ca_path.clone(), false).await {
                Ok(ca_path) => ca_path,
                Err(err) => {
                    eprintln!("Error generating or reading certificate authority: {err}");
                    return Err(err);
                }
            };

            if config.ca_path.is_none() {
                config.ca_path = Some(ca_path);
                config.persist(&config_path).await?;
            }

            if !config
                .proxy
                .iter()
                .any(|proxy| proxy.protocol == Protocol::HttpForward)
            {
                let proxy_configuration_path = match http_proxy_configuration_path {
                    Some(path) => path,
                    None => {
                        let starter_config = include_bytes!("starter.yml");
                        let configuration_path = project_dirs.config_dir().join("config.yml");
                        tokio::fs::write(&configuration_path, starter_config).await?;
                        configuration_path
                    }
                };
                let proxy = Proxy {
                    pre_request_wasi_module_path: None,
                    request_wasi_module_path: None,
                    response_wasi_module_path: None,
                    proxy_configuration_path: Some(proxy_configuration_path),
                    wasi_configuration_bytes: None,
                    port,
                    protocol: Protocol::HttpForward,
                    tls: true,
                    address: "localhost".into(),
                    upstream_address: "".into(),
                    upstream_port: 9999,
                };

                config.add_proxy(proxy);
                config.persist(&config_path).await?;
            }
            proxy::run(config).await?;
            return Ok(());
        }
        None => {}
    };

    let config = Config::try_from(args)?;

    proxy::run(config).await
}
