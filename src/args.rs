use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "nix-service-manager")]
#[command(bin_name = "nix-service-manager")]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Test the configuration")]
    TestConfig {},

    #[command(about = "Manage a service")]
    Service { service_name: String },

    #[command(about = "Manage the daemon")]
    Daemon {
        #[arg(long, group = "actions")]
        start: bool,

        #[arg(long, group = "actions")]
        stop: bool,

        #[arg(long)]
        no_fork: Option<bool>,

        #[arg(long)]
        wait: Option<bool>,
    },
}
