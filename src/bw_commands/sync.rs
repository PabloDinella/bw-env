use anyhow::{Result, Context};
use std::process::Command;

/// Sync with Bitwarden server to ensure we have the latest data
pub fn sync_vault() -> Result<()> {
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