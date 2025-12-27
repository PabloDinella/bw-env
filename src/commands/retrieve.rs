use anyhow::{Result, Context};
use std::process::Command;
use std::fs;
use std::path::Path;
use crate::bw_commands::sync_vault;

pub fn retrieve_env(output: &str) -> Result<()> {
    // Sync with Bitwarden server before retrieving
    sync_vault()?;
    
    // Use full path as stored
    let item_name = generate_full_path_item_name(output)?;
    
    let output_json = Command::new("bw")
        .args(["get", "item", &item_name])
        .output()
        .context("Failed to execute Bitwarden CLI")?;
    
    if !output_json.status.success() {
        anyhow::bail!("Bitwarden CLI failed to retrieve item '{}'", item_name);
    }
    
    let json: serde_json::Value = serde_json::from_slice(&output_json.stdout)
        .context("Failed to parse Bitwarden item JSON")?;
    
    let notes = json["notes"].as_str().unwrap_or("");
    fs::write(output, notes)
        .with_context(|| format!("Failed to write .env file to {}", output))?;
    
    println!(".env file retrieved from Bitwarden and saved to '{}'.", output);
    Ok(())
}

/// Generate an item name from the full path (matches store behavior)
fn generate_full_path_item_name(file_path: &str) -> Result<String> {
    let path = Path::new(file_path);
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;
    
    let relative_path = if path.is_absolute() {
        path.strip_prefix(&current_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    } else {
        file_path.to_string()
    };
    
    // Keep slashes as they are - Bitwarden supports them in item names
    Ok(relative_path)
}

