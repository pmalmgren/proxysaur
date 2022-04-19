use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};

/// A network debugging proxy powered by WebAssembly
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Location of the configuration file
    #[clap(short, long)]
    pub config_path: Option<PathBuf>,
}

fn default_address() -> String {
    "127.0.0.1".into()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Proxy {
    pub wasi_module_path: PathBuf,
    pub port: u16,
    #[serde(default = "default_address")]
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub proxy: Vec<Proxy>,
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
        toml::from_slice(&contents).map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Write};
    use tempdir::TempDir;

    use super::{Args, Config};

    #[test]
    fn parse_config_arg() {
        let data = include_bytes!("test_data/config.toml");

        let tmp_dir = TempDir::new("proxysaur").expect("should create the temp dir");
        let file_path = tmp_dir.path().join("proxysaur.toml");
        let mut tmp_file = File::create(file_path.clone()).expect("should create the file");
        tmp_file.write_all(data).expect("should write the data");

        let args = Args {
            config_path: Some(file_path),
        };
        let config = Config::try_from(args).expect("should build the config object");
        assert_eq!(config.proxy.len(), 3);
    }

    #[test]
    fn parse_config_arg_no_path() {
        let args = Args { config_path: None };
        let data = include_bytes!("test_data/config.toml");

        let tmp_dir = TempDir::new("proxysaur").expect("should create the temp dir");
        std::env::set_current_dir(tmp_dir.path()).expect("should set the current directory");
        let file_path = tmp_dir.path().join("proxysaur.toml");
        let mut tmp_file = File::create(file_path).expect("should create the file");
        tmp_file.write_all(data).expect("should write the data");

        let config = Config::try_from(args).expect("should build the config object");
        assert_eq!(config.proxy.len(), 3);
    }
}
