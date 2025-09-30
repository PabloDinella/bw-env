use anyhow::{Result, Context};
use std::process::Command;
use std::io::Write;
use crate::bw_commands::get_template::{get_template, TemplateType};

/// Create a folder in Bitwarden with the given name
pub fn create_folder(name: &str) -> Result<String> {
    println!("Creating folder '{}'...", name);
    
    // Get the folder template
    let mut template = get_template(TemplateType::Folder)?;
    
    // Set the folder name
    template["name"] = serde_json::Value::String(name.to_string());
    
    let json_str = serde_json::to_string(&template)
        .context("Failed to serialize folder template")?;
    
    // Encode the JSON
    let mut encode_child = Command::new("bw")
        .arg("encode")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn bw encode for folder")?;
    
    {
        let stdin = encode_child.stdin.as_mut().unwrap();
        stdin.write_all(json_str.as_bytes())
            .context("Failed to write to bw encode stdin for folder")?;
    }
    
    let encode_result = encode_child.wait_with_output()
        .context("Failed to wait for bw encode for folder")?;
    
    if !encode_result.status.success() {
        anyhow::bail!("Failed to encode folder");
    }
    
    let encoded_data = String::from_utf8(encode_result.stdout)
        .context("Failed to parse encoded folder data")?;
    
    // Create the folder
    let create_output = Command::new("bw")
        .args(["create", "folder", encoded_data.trim()])
        .output()
        .context("Failed to create Bitwarden folder")?;
    
    if !create_output.status.success() {
        anyhow::bail!("Bitwarden CLI failed to create folder");
    }
    
    // Parse the created folder response to get the ID
    let created_folder: serde_json::Value = serde_json::from_slice(&create_output.stdout)
        .context("Failed to parse created folder JSON")?;
    
    if let Some(id) = created_folder["id"].as_str() {
        println!("Created folder '{}' successfully with ID: {}", name, id);
        Ok(id.to_string())
    } else {
        anyhow::bail!("Failed to get folder ID from created folder response");
    }
}

/// List all folders in Bitwarden and return them as a Vec of JSON values
pub fn list_folders() -> Result<Vec<serde_json::Value>> {
    let list_output = Command::new("bw")
        .args(["list", "folders"])
        .output()
        .context("Failed to list Bitwarden folders")?;
    
    if !list_output.status.success() {
        anyhow::bail!("Failed to list Bitwarden folders");
    }
    
    let folders: Vec<serde_json::Value> = serde_json::from_slice(&list_output.stdout)
        .context("Failed to parse folders JSON")?;
    
    Ok(folders)
}

/// Find a folder by name and return its ID if it exists
pub fn find_folder_by_name(name: &str) -> Result<Option<String>> {
    let folders = list_folders()?;
    
    for folder in folders {
        if let Some(folder_name) = folder["name"].as_str() {
            if folder_name == name {
                if let Some(id) = folder["id"].as_str() {
                    return Ok(Some(id.to_string()));
                }
            }
        }
    }
    
    Ok(None)
}

/// Create a folder if it doesn't exist, otherwise return the existing folder ID
pub fn ensure_folder_exists(name: &str) -> Result<String> {
    // First, try to find existing folder
    if let Some(existing_id) = find_folder_by_name(name)? {
        println!("Found existing folder '{}'.", name);
        return Ok(existing_id);
    }
    
    // Folder doesn't exist, create it
    create_folder(name)
}