use anyhow::{Result, Context};
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum TemplateType {
    Item,
    Folder,
}

impl TemplateType {
    fn as_str(&self) -> &'static str {
        match self {
            TemplateType::Item => "item",
            TemplateType::Folder => "folder",
        }
    }
}

/// Get a template for Bitwarden items or folders
pub fn get_template(template_type: TemplateType) -> Result<serde_json::Value> {
    let template_name = template_type.as_str();
    
    let template_output = Command::new("bw")
        .args(["get", "template", template_name])
        .output()
        .with_context(|| format!("Failed to get Bitwarden {} template", template_name))?;
    
    if !template_output.status.success() {
        anyhow::bail!("Failed to get Bitwarden {} template", template_name);
    }
    
    // Check if we got valid JSON output
    let stdout_str = String::from_utf8(template_output.stdout)
        .context("Failed to parse template output as UTF-8")?;
    
    if stdout_str.trim().is_empty() {
        anyhow::bail!("Empty response from bw get template {} - authentication may have failed", template_name);
    }
    
    // Parse the template
    let template: serde_json::Value = serde_json::from_str(&stdout_str)
        .context("Failed to parse template JSON")?;
    
    Ok(template)
}