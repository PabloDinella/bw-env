use anyhow::{Result, Context};
use std::process::Command;
use std::fs;
use crate::bw_commands::sync_vault;

pub fn retrieve_env(item_name: &str, output: &str) -> Result<()> {
    // Sync with Bitwarden server before retrieving
    sync_vault()?;
    
    let output_json = Command::new("bw")
        .args(["get", "item", item_name])
        .output()
        .context("Failed to execute Bitwarden CLI")?;
    
    if !output_json.status.success() {
        anyhow::bail!("Bitwarden CLI failed to retrieve item");
    }
    
    let json: serde_json::Value = serde_json::from_slice(&output_json.stdout)
        .context("Failed to parse Bitwarden item JSON")?;
    
    let notes = json["notes"].as_str().unwrap_or("");
    fs::write(output, notes)
        .with_context(|| format!("Failed to write .env file to {}", output))?;
    
    println!(".env file retrieved from Bitwarden and saved to '{}'.", output);
    Ok(())
}