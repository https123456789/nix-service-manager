use anyhow::Result;
use clap::Parser;

mod args;
mod config;
mod daemon;

use args::{Args, Commands};
use config::Config;
use daemon::{start_daemon, stop_daemon};

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::TestConfig {} => {
            let config = match args.config {
                Some(config_path) => Config::load_from(config_path)?,
                None => Config::default(),
            };
            dbg!(config);
        }

        Commands::Daemon { start, stop } => {
            if start {
                start_daemon(args)?;
            }

            if stop {
                stop_daemon()?;
            }
        }
        Commands::Service { .. } => todo!(),
    }

    Ok(())
}
