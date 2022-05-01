use std::{path::PathBuf, str::FromStr};

use anyhow::Result;
use bytes::Bytes;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[macro_use]
extern crate derive_builder;

pub mod cli;

/// A network debugging proxy powered by WebAssembly
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Args {
    /// Location of the configuration file
    #[clap(short, long)]
    pub config_path: Option<PathBuf>,
    #[clap(subcommand)]
    pub commands: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generates a CA
    GenerateCa {
        path: Option<PathBuf>,
        #[clap(short)]
        force: bool,
    },
    /// Initializes proxysaur
    Init { path: Option<PathBuf> },
    /// Adds a proxy to the configuration
    AddProxy { path: Option<PathBuf> },
}

fn default_address() -> String {
    "127.0.0.1".into()
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Http,
    HttpForward,
}

impl FromStr for Protocol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tcp" => Ok(Protocol::Tcp),
            "http" => Ok(Protocol::Http),
            "httpforward" => Ok(Protocol::HttpForward),
            _ => Err(anyhow::Error::msg("Invalid protocol.")),
        }
    }
}

fn default_config() -> Option<Bytes> {
    None
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
pub struct Proxy {
    #[serde(default)]
    pub pre_request_wasi_module_path: Option<PathBuf>,
    #[serde(default)]
    pub request_wasi_module_path: Option<PathBuf>,
    #[serde(default)]
    pub response_wasi_module_path: Option<PathBuf>,
    #[serde(default)]
    pub wasi_configuration_path: Option<PathBuf>,
    #[serde(skip, default = "default_config")]
    pub wasi_configuration_bytes: Option<Bytes>,
    pub port: u16,
    pub protocol: Protocol,
    pub tls: bool,
    #[serde(default = "default_address")]
    pub address: String,
    pub upstream_address: String,
    pub upstream_port: u16,
}

impl Default for Proxy {
    fn default() -> Self {
        Self::new()
    }
}

impl Proxy {
    pub fn new() -> Self {
        Self {
            pre_request_wasi_module_path: None,
            request_wasi_module_path: None,
            response_wasi_module_path: None,
            wasi_configuration_path: None,
            wasi_configuration_bytes: None,
            port: 8080,
            protocol: Protocol::Http,
            tls: false,
            address: "blah".into(),
            upstream_address: "blah".into(),
            upstream_port: 8080,
        }
    }

    pub fn address(&self) -> String {
        let mut addr = self.address.clone();
        addr.push(':');
        addr.push_str(&self.port.to_string());
        addr
    }

    pub fn upstream_address(&self) -> String {
        let mut addr = self.upstream_address.clone();
        addr.push(':');
        addr.push_str(&self.upstream_port.to_string());
        addr
    }
}

fn default_proxy() -> Vec<Proxy> {
    vec![]
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_proxy")]
    pub proxy: Vec<Proxy>,
    pub ca_path: Option<PathBuf>,
}

impl Args {
    pub fn new() -> Self {
        Args::parse()
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<Args> for Config {
    type Error = anyhow::Error;

    fn try_from(value: Args) -> Result<Self, Self::Error> {
        let path = value.config_path.map(Ok).unwrap_or_else(|| {
            std::env::current_dir().map(|mut path| {
                path.push("proxysaur.toml");
                path
            })
        })?;
        let contents = std::fs::read(path)?;
        let mut config: Config = toml::from_slice(&contents).map_err(anyhow::Error::from)?;

        for mut proxy in config.proxy.iter_mut() {
            if let Some(config_path) = proxy.wasi_configuration_path.as_ref() {
                let contents = std::fs::read(config_path)?;
                proxy.wasi_configuration_bytes = Some(Bytes::from(contents));
            }
        }

        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Write, path::PathBuf};
    use tempdir::TempDir;

    use super::{Args, Config, Protocol};

    fn tests() -> (TempDir, PathBuf) {
        let data = include_bytes!("tests/config.toml");

        let tmp_dir = TempDir::new("proxysaur").expect("should create the temp dir");
        let file_path = tmp_dir.path().join("proxysaur.toml");
        let mut tmp_file = File::create(file_path.clone()).expect("should create the file");
        tmp_file.write_all(data).expect("should write the data");

        (tmp_dir, file_path)
    }

    #[test]
    fn parse_config_arg() {
        let (_tmp_dir, file_path) = tests();
        let args = Args {
            config_path: Some(file_path),
            commands: None,
        };
        let config = Config::try_from(args).expect("should build the config object");
        assert_eq!(config.proxy.len(), 3);
        assert_eq!(&config.proxy[0].address(), "127.0.0.1:92");
        assert_eq!(&config.proxy[0].protocol, &Protocol::Tcp);
        assert_eq!(&config.proxy[1].address(), "proxysaur.us:93");
        assert_eq!(&config.proxy[2].address(), "0.0.0.0:94");
        assert_eq!(&config.proxy[0].upstream_address(), "127.0.0.1:5432");
        assert_eq!(&config.proxy[1].upstream_address(), "127.0.0.1:8000");
        assert_eq!(&config.proxy[2].upstream_address(), "127.0.0.1:8001");
    }

    #[test]
    fn parse_config_arg_no_path() {
        let (tmp_dir, _file_path) = tests();
        let args = Args {
            config_path: None,
            commands: None,
        };
        let current_dir = std::env::current_dir().expect("should get the current directory");
        std::env::set_current_dir(tmp_dir.path()).expect("should set the current directory");
        let config = Config::try_from(args);
        std::env::set_current_dir(current_dir).expect("should set directory back");
        let config = config.expect("should parse the config");
        assert_eq!(config.proxy.len(), 3);
    }
}
