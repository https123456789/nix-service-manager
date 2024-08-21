use anyhow::{anyhow, bail, Result};
use fork::{fork, Fork};
use git2::Repository;
use lockfile::Lockfile;
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use std::{
    path::{Path, PathBuf},
    process::Child,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

use crate::{
    args::{Args, Commands},
    config::{self, ConfigService},
    sources::check_git_source_update,
};
use crate::{config::Config, sources::ensure_git_source};

pub const LOCKFILE_PATH: &str = "/tmp/nix-service-manager.pid";

// Starting the daemon process requires:
// - Ensuring that another daemon isn't running
// - Forking the current process to create the daemon process
// - Beginning execution of the daemon
// - Exiting the parent process
pub fn start_daemon(args: Args) -> Result<()> {
    let lockfile = Lockfile::create(LOCKFILE_PATH);
    if let Err(lockfile::Error::LockTaken) = lockfile {
        bail!("Daemon is already running!");
    }

    // Explicitly drop the lockfile so it won't exist when we start the daemon
    drop(lockfile);

    let no_forking: bool = match args.command {
        Commands::Daemon { no_fork, .. } => no_fork.unwrap_or_default(),
        _ => false,
    };

    if no_forking {
        eprintln!("Forking is disabled, jumping into daemon main...");
        daemon_main(&args)?;
        return Ok(());
    }

    eprintln!("Forking...");
    match fork() {
        Ok(Fork::Parent(child)) => {
            eprintln!("Sucessfully forked. Child pid is {child}");
        }
        Ok(Fork::Child) => {
            eprintln!("Forked child is here");
            daemon_main(&args)?;
        }
        Err(e) => bail!(anyhow!("Failed to fork").context(e)),
    }
    Ok(())
}

pub fn stop_daemon() -> Result<()> {
    if !PathBuf::from(LOCKFILE_PATH).exists() {
        eprintln!("Daemon is not running!");
        return Ok(());
    }

    let raw = std::fs::read_to_string(LOCKFILE_PATH)?;
    let pid = raw.parse::<i32>()?;

    signal::kill(Pid::from_raw(pid), Signal::SIGTERM)?;

    Ok(())
}

pub fn daemon_main(args: &Args) -> Result<()> {
    // Setup signal handlers
    let terminate = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&terminate))?;
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&terminate))?;

    // We need to bind the lockfile to a variable so it doesn't get dropped
    let _lockfile = Lockfile::create(LOCKFILE_PATH)?;
    std::fs::write(LOCKFILE_PATH, format!("{}", std::process::id()))?;

    // Fetch the configuration
    let config = match &args.config {
        Some(config_path) => Config::load_from(config_path.to_owned())?,
        None => Config::default(),
    };
    if config::CONFIG.set(config).is_err() {
        bail!("Failed to set global config");
    }

    let sources_root = &config::CONFIG
        .get()
        .expect("Global config should be initialized")
        .root;
    let debug_allowed = match &config::CONFIG
        .get()
        .expect("Global config should be initialized")
        .debug
    {
        Some(value) => value.to_owned(),
        None => false,
    };

    // Start the services
    let mut children = start_services(sources_root, debug_allowed)?;

    eprintln!("All services have been started");

    let mut start = Instant::now();

    while !terminate.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_secs(1));

        if start.elapsed().as_secs() > 59 {
            let services = &config::CONFIG
                .get()
                .expect("Global config should be initialized")
                .services;

            for (i, (name, service)) in services.iter().enumerate() {
                if service.git_uri.is_none() {
                    continue;
                }

                let check = match check_git_source_update(name, service, sources_root) {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("Error: {e:?}");
                        continue;
                    }
                };

                if check {
                    eprintln!("Updating git source for {}", name);
                    let new_child = match update_git_service(
                        name,
                        service,
                        sources_root,
                        &mut children[i],
                    ) {
                        Ok(child) => child,
                        Err(e) => {
                            eprintln!("Git source update failed: {e:?}");
                            children.remove(i);
                            continue
                        },
                    };
                    children[i] = new_child;
                }
            }

            start = Instant::now();
        }
    }

    stop_services(children)?;

    eprintln!("Daemon finished");

    Ok(())
}

fn start_services(sources_root: &PathBuf, debug_allowed: bool) -> Result<Vec<Child>> {
    let mut children = vec![];
    for (name, conf) in config::CONFIG
        .get()
        .expect("Global config should be initialized")
        .services
        .iter()
    {
        if !conf.enabled {
            continue;
        }

        let cdir = match (&conf.base_dir, &conf.git_uri) {
            (Some(dir), None) => dir,
            (None, Some(_)) => &PathBuf::from(sources_root).join(name),
            (Some(_), Some(_)) => {
                bail!("Invalid configuration ({name}): base_dir and git_uri are exclusive!")
            }
            (None, None) => {
                bail!("Invalid configuration ({name}): either base_dir or git_uri must be set!")
            }
        };

        if debug_allowed {
            eprintln!("Cdir for service {name}: {cdir:?}");
        }

        if conf.base_dir.is_none() {
            ensure_git_source(
                config::CONFIG
                    .get()
                    .expect("Global config should be initialized"),
                name,
            )?;
        }

        eprintln!("Starting service: {}", name);
        let child = std::process::Command::new("sh")
            .current_dir(cdir)
            .arg("-c")
            .arg(&conf.run_command)
            .spawn()?;
        children.push(child);
    }

    Ok(children)
}

fn stop_services(children: Vec<Child>) -> Result<()> {
    for mut child in children {
        child.kill()?;
    }

    Ok(())
}

/// Update the source for a git service
///
/// Rather than worry about git merge problems, this will just delete the directory and clone again
fn update_git_service(
    name: &str,
    service: &ConfigService,
    sources_root: &Path,
    service_proc: &mut Child,
) -> Result<Child> {
    let source_path = sources_root.join(format!("{}-update-tmp", name));

    eprintln!("[{}] | Cloning git repo to temp dir", name);

    Repository::clone(service.git_uri.as_ref().unwrap(), &source_path)?;
    
    eprintln!("[{}] | Done with clone; stopping service", name);

    service_proc.kill()?;

    std::fs::remove_dir_all(sources_root.join(name))?;
    std::fs::rename(source_path, sources_root.join(name))?;
    
    eprintln!("[{}] | New source is all ready; restarting service", name);

    let child = std::process::Command::new("sh")
        .current_dir(sources_root.join(name))
        .arg("-c")
        .arg(&service.run_command)
        .spawn()?;

    Ok(child)
}
