use anyhow::{Result, Context};
use std::process::Command;
use std::collections::HashMap;
use crate::bw_commands::sync_vault;

// Configuration: Root folder name in Bitwarden
const ROOT_FOLDER_NAME: &str = "bw-env";

pub fn list_env_items() -> Result<()> {
    // Sync with Bitwarden server before listing
    sync_vault()?;
    
    // First, get the bw-env folder ID
    let folder_id = get_bw_env_folder_id()?;
    
    if let Some(_id) = folder_id {
        // List all items in the bw-env folder hierarchy
        list_items_in_folder()?;
    } else {
        println!("No '{}' folder found in Bitwarden. Use 'bw-env store' to create it and add items.", ROOT_FOLDER_NAME);
    }
    
    Ok(())
}

fn get_bw_env_folder_id() -> Result<Option<String>> {
    let folders_output = Command::new("bw")
        .args(["list", "folders"])
        .output()
        .context("Failed to list Bitwarden folders")?;
    
    if !folders_output.status.success() {
        anyhow::bail!("Failed to list Bitwarden folders");
    }
    
    let folders: Vec<serde_json::Value> = serde_json::from_slice(&folders_output.stdout)
        .context("Failed to parse folders JSON")?;
    
    for folder in folders {
        if let Some(folder_name) = folder["name"].as_str() {
            if folder_name == ROOT_FOLDER_NAME {
                if let Some(id) = folder["id"].as_str() {
                    return Ok(Some(id.to_string()));
                }
            }
        }
    }
    
    Ok(None)
}

fn list_items_in_folder() -> Result<()> {
    // First, get all folders to build a folder hierarchy map
    let folder_map = build_folder_hierarchy()?;
    
    // Get items from all nested folders within bw-env
    let all_items = get_all_items_in_bw_env_hierarchy(&folder_map)?;
    
    if all_items.is_empty() {
        println!("No .env files found in the '{}' folder or its subfolders.", ROOT_FOLDER_NAME);
        return Ok(());
    }
    
    println!("Found {} .env file(s) in '{}' and its subfolders:", all_items.len(), ROOT_FOLDER_NAME);
    println!();
    
    for item in all_items {
        if let (Some(name), Some(id)) = (item["name"].as_str(), item["id"].as_str()) {
            // Get the folder path and folder ID for this item
            let (folder_path, folder_id) = get_folder_info(&item, &folder_map);
            
            // Format dates inline
            let created = item["creationDate"].as_str().map(format_date).unwrap_or_else(|| "Unknown".to_string());
            let modified = item["revisionDate"].as_str().map(format_date).unwrap_or_else(|| "Unknown".to_string());
            
            // Generate Bitwarden vault link
            let vault_link = format!("https://vault.bitwarden.com/#/vault?folderId={}&itemId={}&action=view", folder_id, id);
            
            println!("📄 {} | {} | Created: {} | Modified: {} | {}", 
                name, folder_path, created, modified, vault_link);
        }
    }
    
    Ok(())
}

fn build_folder_hierarchy() -> Result<HashMap<String, serde_json::Value>> {
    let folders_output = Command::new("bw")
        .args(["list", "folders"])
        .output()
        .context("Failed to list all folders")?;
    
    if !folders_output.status.success() {
        anyhow::bail!("Failed to list all folders");
    }
    
    let folders: Vec<serde_json::Value> = serde_json::from_slice(&folders_output.stdout)
        .context("Failed to parse folders JSON")?;
    
    let mut folder_map = HashMap::new();
    
    for folder in folders {
        if let Some(id) = folder["id"].as_str() {
            folder_map.insert(id.to_string(), folder);
        }
    }
    
    Ok(folder_map)
}

fn get_all_items_in_bw_env_hierarchy(folder_map: &HashMap<String, serde_json::Value>) -> Result<Vec<serde_json::Value>> {
    let mut all_items = Vec::new();
    
    // Find all folders that are part of the bw-env hierarchy
    let bw_env_folders = get_bw_env_related_folders(folder_map);
    
    for folder_id in bw_env_folders {
        let items_output = Command::new("bw")
            .args(["list", "items", "--folderid", &folder_id])
            .output()
            .context("Failed to list items in folder")?;
        
        if items_output.status.success() {
            if let Ok(items) = serde_json::from_slice::<Vec<serde_json::Value>>(&items_output.stdout) {
                all_items.extend(items);
            }
        }
    }
    
    Ok(all_items)
}

fn get_bw_env_related_folders(folder_map: &HashMap<String, serde_json::Value>) -> Vec<String> {
    let mut bw_env_folders = Vec::new();
    
    for (folder_id, folder) in folder_map {
        if let Some(folder_name) = folder["name"].as_str() {
            // Include the root bw-env folder and any folder whose path starts with bw-env/
            if folder_name == ROOT_FOLDER_NAME || folder_name.starts_with(&format!("{}/", ROOT_FOLDER_NAME)) {
                bw_env_folders.push(folder_id.clone());
            }
        }
    }
    
    bw_env_folders
}

fn get_folder_info(item: &serde_json::Value, folder_map: &HashMap<String, serde_json::Value>) -> (String, String) {
    if let Some(folder_id) = item["folderId"].as_str() {
        if let Some(folder) = folder_map.get(folder_id) {
            if let Some(folder_name) = folder["name"].as_str() {
                return (folder_name.to_string(), folder_id.to_string());
            }
        }
        // If we have a folder ID but can't find the folder name, still return the ID
        return (ROOT_FOLDER_NAME.to_string(), folder_id.to_string());
    }
    
    // Fallback - try to find the root bw-env folder ID
    let root_folder_id = folder_map.iter()
        .find(|(_, folder)| {
            folder["name"].as_str() == Some(ROOT_FOLDER_NAME)
        })
        .map(|(id, _)| id.clone())
        .unwrap_or_else(|| "unknown".to_string());
    
    (ROOT_FOLDER_NAME.to_string(), root_folder_id)
}

fn format_date(date_str: &str) -> String {
    // Simple date formatting - just show the date part
    if date_str.len() >= 10 {
        date_str[..10].to_string() // Extract YYYY-MM-DD part
    } else {
        date_str.to_string()
    }
}