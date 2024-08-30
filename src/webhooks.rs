use std::sync::{atomic::AtomicBool, Arc};

use anyhow::Result;

pub fn webhook_server_main(terminate: Arc<AtomicBool>) -> Result<()> {
    Ok(())
}
