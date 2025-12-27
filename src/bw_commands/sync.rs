use anyhow::{Context, Result};
use std::process::Command;
use crate::auth::ensure_bw_cli_available;

/// Sync with Bitwarden server to ensure we have the latest data
pub fn sync_vault() -> Result<()> {
    ensure_bw_cli_available()?;

    println!("Syncing with Bitwarden server...");
    
    let sync_status = Command::new("bw")
        .arg("sync")
        .status()
        .context("Failed to run bw sync")?;
    
    if !sync_status.success() {
        anyhow::bail!("Failed to sync with Bitwarden server");
    }
    
    println!("Sync completed successfully.");
    Ok(())
}