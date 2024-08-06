use anyhow::Result;
use clap::Parser;

mod args;
mod config;
mod daemon;

use args::{Args, Commands};
use daemon::{start_daemon, stop_daemon};

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Daemon { start, stop } => {
            if start {
                start_daemon()?;
            }

            if stop {
                stop_daemon()?;
            }
        }
        Commands::Service { .. } => todo!(),
    }

    Ok(())
}
