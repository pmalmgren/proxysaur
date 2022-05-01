use std::{io::Write, path::PathBuf, str::FromStr};

use anyhow::Result;

use crate::{Config, Protocol, Proxy, ProxyBuilder};

/// Responsible for creating a proxysaur.toml file
pub fn init(path: Option<PathBuf>) -> Result<()> {
    let path = match path {
        Some(path) => path,
        None => {
            let mut path = std::env::current_dir()?;
            path.push("proxysaur.toml");
            path
        }
    };

    match std::fs::metadata(&path) {
        Ok(_metadata) => eprintln!("File exists at {:?}.", path),
        Err(_) => std::fs::write(path, "")?,
    };

    Ok(())
}

fn try_input<T: FromStr>(prompt: &str) -> T {
    loop {
        let mut buffer = String::new();
        print!("{}", prompt);
        let _res = std::io::stdout().flush();
        if let Err(_err) = std::io::stdin().read_line(&mut buffer) {
            println!("Error reading input.");
            continue;
        }
        let input = buffer
            .strip_suffix("\r\n")
            .or_else(|| buffer.strip_suffix('\n'))
            .unwrap_or(&buffer);
        match T::from_str(input) {
            Ok(val) => {
                return val;
            }
            Err(_err) => println!("Invalid input: {}", input),
        }
    }
}

fn get_proxy() -> Result<Proxy> {
    let mut builder = ProxyBuilder::create_empty();

    let address: String = try_input("Enter host: ");
    let port: u16 = try_input("Enter port: ");
    let protocol: Protocol = try_input("Enter protocol [http|httpforward|tcp]: ");
    let tls: bool = try_input("Use tls [true/false]: ");

    let upstream_address: String = match protocol {
        Protocol::Tcp | Protocol::Http => try_input("Upstream address: "),
        Protocol::HttpForward => "".to_string(),
    };
    let upstream_port: u16 = match protocol {
        Protocol::Tcp | Protocol::Http => try_input("Upstream port: "),
        Protocol::HttpForward => 9999,
    };

    let use_custom_wasi: bool = try_input("Use custom wasi [true/false]? ");
    let pre_request_wasi_module_path = if use_custom_wasi {
        let output: String = try_input("Enter pre-request WASI path: ");
        if output.is_empty() {
            None
        } else {
            Some(PathBuf::from(output))
        }
    } else {
        None
    };
    let request_wasi_module_path = if use_custom_wasi {
        let output: String = try_input("Enter request WASI path: ");
        if output.is_empty() {
            None
        } else {
            Some(PathBuf::from(output))
        }
    } else {
        None
    };
    let response_wasi_module_path = if use_custom_wasi {
        let output: String = try_input("Enter response WASI path: ");
        if output.is_empty() {
            None
        } else {
            Some(PathBuf::from(output))
        }
    } else {
        None
    };
    let wasi_configuration_path = if use_custom_wasi {
        let output: String = try_input("Enter custom WASI configuration path: ");
        if output.is_empty() {
            None
        } else {
            Some(PathBuf::from(output))
        }
    } else {
        None
    };

    builder
        .address(address)
        .port(port)
        .protocol(protocol)
        .tls(tls)
        .upstream_address(upstream_address)
        .upstream_port(upstream_port)
        .pre_request_wasi_module_path(pre_request_wasi_module_path)
        .request_wasi_module_path(request_wasi_module_path)
        .response_wasi_module_path(response_wasi_module_path)
        .wasi_configuration_path(wasi_configuration_path)
        .build()
        .map_err(anyhow::Error::from)
}

pub fn add_proxy(conf_path: Option<PathBuf>) -> Result<()> {
    let conf_path = match conf_path {
        Some(conf_path) => conf_path,
        None => {
            let mut conf_path = std::env::current_dir()?;
            conf_path.push("proxysaur.toml");
            conf_path
        }
    };
    let config_contents = std::fs::read(&conf_path)?;
    let mut config: Config = toml::from_slice(&config_contents)?;

    let proxy = get_proxy()?;
    config.proxy.push(proxy);

    let new_config_contents = toml::to_string_pretty(&config)?;
    std::fs::write(conf_path, new_config_contents)?;

    Ok(())
}
