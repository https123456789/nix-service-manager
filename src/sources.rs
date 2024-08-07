use std::path::PathBuf;
use crate::config::Config;
use anyhow::{anyhow, Result};
use git2::Repository;

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
        Repository::clone(&uri, dir)?;
    }

    Ok(())
}
