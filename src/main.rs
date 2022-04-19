use clap::StructOpt;
use config::{Args, Config};
use anyhow::Result;

mod config;

fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::try_from(args)?;

    Ok(())
}
