use anyhow::Result;
use clap::StructOpt;
use config::{Args, Config};

mod config;

fn main() -> Result<()> {
    let args = Args::parse();
    let _config = Config::try_from(args)?;

    Ok(())
}
