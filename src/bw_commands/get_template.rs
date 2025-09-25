use anyhow::{Result, Context};
use std::process::Command;

/// Get the template for a Bitwarden item
pub fn get_item_template() -> Result<serde_json::Value> {
    let template_output = Command::new("bw")
        .args(["get", "template", "item"])
        .output()
        .context("Failed to get Bitwarden item template")?;
    
    if !template_output.status.success() {
        anyhow::bail!("Failed to get Bitwarden item template");
    }
    
    // Check if we got valid JSON output
    let stdout_str = String::from_utf8(template_output.stdout)
        .context("Failed to parse template output as UTF-8")?;
    
    if stdout_str.trim().is_empty() {
        anyhow::bail!("Empty response from bw get template item - authentication may have failed");
    }
    
    // Parse the template
    let template: serde_json::Value = serde_json::from_str(&stdout_str)
        .context("Failed to parse template JSON")?;
    
    Ok(template)
}

/// Get the template for a Bitwarden folder
pub fn get_folder_template() -> Result<serde_json::Value> {
    let template_output = Command::new("bw")
        .args(["get", "template", "folder"])
        .output()
        .context("Failed to get Bitwarden folder template")?;
    
    if !template_output.status.success() {
        anyhow::bail!("Failed to get Bitwarden folder template");
    }
    
    let stdout_str = String::from_utf8(template_output.stdout)
        .context("Failed to parse folder template output as UTF-8")?;
    
    if stdout_str.trim().is_empty() {
        anyhow::bail!("Empty response from bw get template folder - authentication may have failed");
    }
    
    let template: serde_json::Value = serde_json::from_str(&stdout_str)
        .context("Failed to parse folder template JSON")?;
    
    Ok(template)
}