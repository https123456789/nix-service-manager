use anyhow::Result;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf, process::Command, sync::OnceLock};

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub debug: Option<bool>,
    pub root: PathBuf,
    pub services: HashMap<String, ConfigService>,
}

#[derive(Debug, Default, Deserialize, Hash, PartialEq)]
pub struct ConfigService {
    pub base_dir: Option<PathBuf>,
    pub git_uri: Option<String>,
    pub enabled: bool,
    pub run_command: String,
}

impl Config {
    pub fn load_from(file: PathBuf) -> Result<Self> {
        let output = Command::new("nix")
            .args(["eval", "--json", "--file"])
            .arg(&file)
            .output()?;
        Ok(serde_json::from_str(
            String::from_utf8(output.stdout)?.as_str(),
        )?)
    }
}
