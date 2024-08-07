use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use clap::Parser;

mod args;
mod config;
mod daemon;
mod sources;

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

        Commands::Daemon { start, stop, wait } => {
            if start {
                start_daemon(args)?;
            }

            if stop {
                stop_daemon()?;
            }

            if wait.is_some() && wait.unwrap() {
                // Give the daemon a change to get the lockfile
                std::thread::sleep(Duration::from_secs(1));

                while PathBuf::from(daemon::LOCKFILE_PATH).exists() {
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        }
        Commands::Service { .. } => todo!(),
    }

    Ok(())
}
