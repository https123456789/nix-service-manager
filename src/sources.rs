use crate::config::{Config, ConfigService};
use anyhow::{anyhow, Result};
use git2::build::RepoBuilder;
use git2::{
    AutotagOption, Cred, FetchOptions, RemoteCallbacks, RemoteUpdateFlags, Repository,
    StatusOptions,
};
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn ensure_git_source(config: &Config, name: &str) -> Result<()> {
    let dir = PathBuf::from(&config.root).join(name);

    if Repository::open(&dir).is_err() {
        std::fs::create_dir_all(&dir)?;
        let uri = config
            .services
            .get(name)
            .ok_or(anyhow!("Failed to get service {name} from config"))?
            .git_uri
            .as_ref()
            .unwrap()
            .to_string();

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key(
                username_from_url.unwrap(),
                None,
                Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
                None,
            )
        });

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_options);
        builder.clone(&uri, &dir)?;
    }

    Ok(())
}

pub fn check_git_source_update(
    name: &str,
    service: &ConfigService,
    sources_root: &PathBuf,
) -> Result<bool> {
    let source_path = sources_root.join(name);
    let repo = Repository::open(source_path)?;
    let mut cb = RemoteCallbacks::new();
    let mut remote = repo.find_remote("origin")?;

    // Fetch
    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(cb);
    remote.download(&[] as &[&str], Some(&mut fetch_options))?;

    let received = {
        let stats = remote.stats();

        if stats.received_bytes() > 0 {
            if stats.local_objects() > 0 {
                eprintln!(
                    "[{}] | Received {}/{} objects in {} bytes (used {} local \
                     objects)",
                    name,
                    stats.indexed_objects(),
                    stats.total_objects(),
                    stats.received_bytes(),
                    stats.local_objects()
                );
            } else {
                eprintln!(
                    "[{}] | Received {}/{} objects in {} bytes",
                    name,
                    stats.indexed_objects(),
                    stats.total_objects(),
                    stats.received_bytes()
                );
            }
        }

        stats.received_bytes()
    };

    remote.disconnect()?;

    // We intentionaly don't actually do the update

    Ok(received > 0)
}
