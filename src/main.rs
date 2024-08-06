use clap::Parser;
use lockfile::Lockfile;
use fork::{fork, Fork};
use anyhow::{anyhow, bail, Result};

mod daemon;
mod args;

use args::{Args, Commands};
use daemon::{LOCKFILE_PATH, daemon_main};

fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Daemon { start, stop } => {
            if start {
                start_daemon()?;
            }
        },
        Commands::Service { .. } => todo!(),
    }
    Ok(())
}

// Starting the daemon process requires:
// - Ensuring that another daemon isn't running
// - Forking the current process to create the daemon process
// - Beginning execution of the daemon
// - Exiting the parent process
fn start_daemon() -> Result<()> {
    let lockfile = Lockfile::create(LOCKFILE_PATH);
    if let Err(lockfile::Error::LockTaken) = lockfile {
        bail!("Daemon is already running!");
    }
    
    // Explicitly drop the lockfile so it won't exist when we start the daemon
    drop(lockfile);

    eprintln!("Forking...");
    match fork() {
        Ok(Fork::Parent(child)) => {
            eprintln!("Sucessfully forked. Child pid is {child}");
        },
        Ok(Fork::Child) => {
            eprintln!("Forked child is here");
            daemon_main()?;
        },
        Err(e) => bail!(anyhow!("Failed to fork").context(e))
    }
    Ok(())
}
