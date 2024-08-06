use anyhow::{anyhow, bail, Result};
use fork::{fork, Fork};
use lockfile::Lockfile;
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

pub const LOCKFILE_PATH: &str = "/tmp/nix-service-manager.pid";

// Starting the daemon process requires:
// - Ensuring that another daemon isn't running
// - Forking the current process to create the daemon process
// - Beginning execution of the daemon
// - Exiting the parent process
pub fn start_daemon() -> Result<()> {
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
        }
        Ok(Fork::Child) => {
            eprintln!("Forked child is here");
            daemon_main()?;
        }
        Err(e) => bail!(anyhow!("Failed to fork").context(e)),
    }
    Ok(())
}

pub fn stop_daemon() -> Result<()> {
    let raw = std::fs::read_to_string(LOCKFILE_PATH)?;
    let pid = raw.parse::<i32>()?;

    signal::kill(Pid::from_raw(pid), Signal::SIGTERM)?;

    Ok(())
}

pub fn daemon_main() -> Result<()> {
    // Setup signal handlers
    let terminate = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&terminate))?;
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&terminate))?;

    // We need to bind it to a variable so it doesn't get dropped
    let _lockfile = Lockfile::create(LOCKFILE_PATH)?;
    std::fs::write(LOCKFILE_PATH, format!("{}", std::process::id()))?;

    while !terminate.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_secs(1));
        eprintln!("Message");
    }

    eprintln!("Daemon finished");

    Ok(())
}
