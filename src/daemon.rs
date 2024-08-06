use anyhow::Result;
use lockfile::Lockfile;

pub const LOCKFILE_PATH: &'static str = "/tmp/nix-service-manager.pid";

pub fn daemon_main() -> Result<()> {
    let lockfile = Lockfile::create(LOCKFILE_PATH)?;

    std::fs::write(LOCKFILE_PATH, format!("{}", std::process::id()))?;

    std::thread::sleep(std::time::Duration::from_secs(15));

    eprintln!("Message");

    Ok(())
}
