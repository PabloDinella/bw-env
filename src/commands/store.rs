use anyhow::{Result, Context};
use std::process::Command;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub fn store_env(path: &str, item_name: &str) -> Result<()> {
    let env_content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read .env file at {}", path))?;
    
    // Check for or create "dotenv files" folder
    let root_folder_id = ensure_dotenv_folder()?;
    
    // Get the desired folder structure from user
    let target_folder_id = get_target_folder(path, &root_folder_id)?;
    
    // Get the item template
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
    
    // Parse and modify the template
    let mut template: serde_json::Value = serde_json::from_str(&stdout_str)
        .context("Failed to parse template JSON")?;
    
    template["type"] = serde_json::Value::Number(2.into()); // Secure note type
    template["secureNote"] = serde_json::json!({"type": 0}); // Initialize secureNote object
    template["notes"] = serde_json::Value::String(env_content);
    template["name"] = serde_json::Value::String(item_name.to_string());
    template["folderId"] = serde_json::Value::String(target_folder_id);
    
    // Convert to JSON string
    let json_str = serde_json::to_string(&template)
        .context("Failed to serialize template")?;
    
    // Encode the JSON
    let mut encode_child = Command::new("bw")
        .arg("encode")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn bw encode")?;
    
    {
        let stdin = encode_child.stdin.as_mut().unwrap();
        stdin.write_all(json_str.as_bytes())
            .context("Failed to write to bw encode stdin")?;
    }
    
    let encode_result = encode_child.wait_with_output()
        .context("Failed to wait for bw encode")?;
    
    if !encode_result.status.success() {
        anyhow::bail!("Failed to encode item");
    }
    
    let encoded_data = String::from_utf8(encode_result.stdout)
        .context("Failed to parse encoded data")?;
    
    // Create the item
    let create_status = Command::new("bw")
        .args(["create", "item", encoded_data.trim()])
        .status()
        .context("Failed to create Bitwarden item")?;
    
    if !create_status.success() {
        anyhow::bail!("Bitwarden CLI failed to store item");
    }
    
    println!(".env file stored in Bitwarden as '{}'.", item_name);
    Ok(())
}

fn ensure_dotenv_folder() -> Result<String> {
    const FOLDER_NAME: &str = "dotenv files";
    
    // First, try to find existing folder
    let list_output = Command::new("bw")
        .args(["list", "folders"])
        .output()
        .context("Failed to list Bitwarden folders")?;
    
    if !list_output.status.success() {
        anyhow::bail!("Failed to list Bitwarden folders");
    }
    
    let folders: Vec<serde_json::Value> = serde_json::from_slice(&list_output.stdout)
        .context("Failed to parse folders JSON")?;
    
    // Check if "dotenv files" folder already exists
    for folder in &folders {
        if let Some(name) = folder["name"].as_str() {
            if name == FOLDER_NAME {
                if let Some(id) = folder["id"].as_str() {
                    println!("Found existing '{}' folder.", FOLDER_NAME);
                    return Ok(id.to_string());
                }
            }
        }
    }
    
    // Folder doesn't exist, create it
    println!("Creating '{}' folder in Bitwarden...", FOLDER_NAME);
    
    // Get folder template
    let template_output = Command::new("bw")
        .args(["get", "template", "folder"])
        .output()
        .context("Failed to get Bitwarden folder template")?;
    
    if !template_output.status.success() {
        anyhow::bail!("Failed to get Bitwarden folder template");
    }
    
    let stdout_str = String::from_utf8(template_output.stdout)
        .context("Failed to parse folder template output as UTF-8")?;
    
    let mut template: serde_json::Value = serde_json::from_str(&stdout_str)
        .context("Failed to parse folder template JSON")?;
    
    template["name"] = serde_json::Value::String(FOLDER_NAME.to_string());
    
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
        println!("Created '{}' folder successfully.", FOLDER_NAME);
        Ok(id.to_string())
    } else {
        anyhow::bail!("Failed to get folder ID from created folder response");
    }
}

fn get_target_folder(file_path: &str, root_folder_id: &str) -> Result<String> {
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;
    
    // Option 1: Git repository path (if applicable)
    let git_option = get_git_repo_path(&current_dir, file_name);
    
    // Option 2: Directory name path
    let dir_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let dir_option = format!("dotenv files/{}/{}", dir_name, file_name);
    
    // Display options to user
    println!("\nChoose the folder structure for storing '{}':", file_name);
    
    let mut option_num = 1;
    
    if let Some(ref git_path) = git_option {
        println!("{}. {} (Git repository structure)", option_num, git_path);
        option_num += 1;
    }
    
    println!("{}. {} (Directory name)", option_num, dir_option);
    let dir_option_num = option_num;
    option_num += 1;
    
    println!("{}. Custom path (you type the path)", option_num);
    let custom_option_num = option_num;
    
    print!("\nEnter your choice: ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .context("Failed to read user input")?;
    
    let choice: u32 = input.trim().parse()
        .context("Invalid choice, please enter a number")?;
    
    let selected_path = if git_option.is_some() && choice == 1 {
        git_option.unwrap()
    } else if choice == dir_option_num {
        dir_option
    } else if choice == custom_option_num {
        print!("Enter custom path (starting with 'dotenv files/'): ");
        io::stdout().flush().unwrap();
        let mut custom_path = String::new();
        io::stdin().read_line(&mut custom_path)
            .context("Failed to read custom path")?;
        custom_path.trim().to_string()
    } else {
        anyhow::bail!("Invalid choice");
    };
    
    // Ensure path starts with "dotenv files"
    if !selected_path.starts_with("dotenv files") {
        anyhow::bail!("Path must start with 'dotenv files'");
    }
    
    // Create the folder structure and return the final folder ID
    create_folder_structure(&selected_path, root_folder_id)
}

fn get_git_repo_path(current_dir: &Path, file_name: &str) -> Option<String> {
    // Check if we're in a git repository
    let git_output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(current_dir)
        .output()
        .ok()?;
    
    if !git_output.status.success() {
        return None;
    }
    
    let remote_url = String::from_utf8(git_output.stdout).ok()?;
    let remote_url = remote_url.trim();
    
    // Parse GitHub URL (handles both HTTPS and SSH)
    let repo_info = if remote_url.starts_with("https://github.com/") {
        remote_url.strip_prefix("https://github.com/")?
            .strip_suffix(".git")
            .unwrap_or(remote_url.strip_prefix("https://github.com/")?)
    } else if remote_url.starts_with("git@github.com:") {
        remote_url.strip_prefix("git@github.com:")?
            .strip_suffix(".git")
            .unwrap_or(remote_url.strip_prefix("git@github.com:")?)
    } else {
        return None;
    };
    
    // Split into owner/repo
    let parts: Vec<&str> = repo_info.split('/').collect();
    if parts.len() == 2 {
        Some(format!("dotenv files/{}/{}/{}", parts[0], parts[1], file_name))
    } else {
        None
    }
}

fn create_folder_structure(path: &str, root_folder_id: &str) -> Result<String> {
    // Check if path starts with "dotenv files/"
    if !path.starts_with("dotenv files/") {
        anyhow::bail!("Invalid path format: must start with 'dotenv files/'");
    }
    
    // Remove "dotenv files/" prefix and split the remaining path
    let remaining_path = &path[13..]; // "dotenv files/" is 13 characters
    let parts: Vec<&str> = remaining_path.split('/').collect();
    
    let mut current_folder_id = root_folder_id.to_string();
    
    // Create each folder in the path, excluding the last part (filename)
    for folder_name in &parts[..parts.len().saturating_sub(1)] {
        if !folder_name.is_empty() {
            current_folder_id = ensure_subfolder(folder_name, &current_folder_id)?;
        }
    }
    
    Ok(current_folder_id)
}

fn ensure_subfolder(folder_name: &str, parent_folder_id: &str) -> Result<String> {
    // List all folders and check if this subfolder exists
    let list_output = Command::new("bw")
        .args(["list", "folders"])
        .output()
        .context("Failed to list Bitwarden folders")?;
    
    if !list_output.status.success() {
        anyhow::bail!("Failed to list Bitwarden folders");
    }
    
    let folders: Vec<serde_json::Value> = serde_json::from_slice(&list_output.stdout)
        .context("Failed to parse folders JSON")?;
    
    // Check if subfolder already exists with the same name and parent
    for folder in &folders {
        if let (Some(name), Some(id)) = (folder["name"].as_str(), folder["id"].as_str()) {
            // Bitwarden doesn't store parent folder info in folder objects,
            // so we'll just check by name for now
            if name == folder_name {
                return Ok(id.to_string());
            }
        }
    }
    
    // Folder doesn't exist, create it
    println!("Creating folder '{}'...", folder_name);
    
    let template_output = Command::new("bw")
        .args(["get", "template", "folder"])
        .output()
        .context("Failed to get Bitwarden folder template")?;
    
    if !template_output.status.success() {
        anyhow::bail!("Failed to get Bitwarden folder template");
    }
    
    let stdout_str = String::from_utf8(template_output.stdout)
        .context("Failed to parse folder template output as UTF-8")?;
    
    let mut template: serde_json::Value = serde_json::from_str(&stdout_str)
        .context("Failed to parse folder template JSON")?;
    
    template["name"] = serde_json::Value::String(folder_name.to_string());
    
    let json_str = serde_json::to_string(&template)
        .context("Failed to serialize folder template")?;
    
    // Encode and create the folder
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
    
    let create_output = Command::new("bw")
        .args(["create", "folder", encoded_data.trim()])
        .output()
        .context("Failed to create Bitwarden folder")?;
    
    if !create_output.status.success() {
        anyhow::bail!("Bitwarden CLI failed to create folder");
    }
    
    let created_folder: serde_json::Value = serde_json::from_slice(&create_output.stdout)
        .context("Failed to parse created folder JSON")?;
    
    if let Some(id) = created_folder["id"].as_str() {
        Ok(id.to_string())
    } else {
        anyhow::bail!("Failed to get folder ID from created folder response");
    }
}