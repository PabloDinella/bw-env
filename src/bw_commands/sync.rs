use anyhow::Result;
use crate::auth::run_bw_command;
use crate::auth::ensure_bw_cli_available;

/// Sync with Bitwarden server to ensure we have the latest data
pub fn sync_vault() -> Result<()> {
    ensure_bw_cli_available()?;

    println!("Syncing with Bitwarden server...");
    
    let sync_output = run_bw_command(&["sync"])?;
    
    if !sync_output.status.success() {
        anyhow::bail!("Failed to sync with Bitwarden server");
    }
    
    println!("Sync completed successfully.");
    Ok(())
}