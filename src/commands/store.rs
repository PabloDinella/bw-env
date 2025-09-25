use anyhow::{Result, Context};
use std::process::Command;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

// Configuration: Root folder name in Bitwarden
const ROOT_FOLDER_NAME: &str = "bw-env";

pub fn store_env(path: &str, item_name: &str) -> Result<()> {
    // Sync with Bitwarden server before storing
    println!("Syncing with Bitwarden server...");
    let sync_status = Command::new("bw")
        .arg("sync")
        .status()
        .context("Failed to run bw sync")?;
    
    if !sync_status.success() {
        anyhow::bail!("Failed to sync with Bitwarden server");
    }
    
    let env_content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read .env file at {}", path))?;
    
    // Check for or create root folder
    let root_folder_id = ensure_root_folder()?;
    
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

fn ensure_root_folder() -> Result<String> {
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
    
    // Check if root folder already exists
    for folder in &folders {
        if let Some(name) = folder["name"].as_str() {
            if name == ROOT_FOLDER_NAME {
                if let Some(id) = folder["id"].as_str() {
                    println!("Found existing '{}' folder.", ROOT_FOLDER_NAME);
                    return Ok(id.to_string());
                }
            }
        }
    }
    
    // Folder doesn't exist, create it
    println!("Creating '{}' folder in Bitwarden...", ROOT_FOLDER_NAME);
    
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
    
    template["name"] = serde_json::Value::String(ROOT_FOLDER_NAME.to_string());
    
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
        println!("Created '{}' folder successfully.", ROOT_FOLDER_NAME);
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
    let dir_option = format!("{}/{}/{}", ROOT_FOLDER_NAME, dir_name, file_name);
    
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
        print!("Enter custom path (starting with '{}/''): ", ROOT_FOLDER_NAME);
        io::stdout().flush().unwrap();
        let mut custom_path = String::new();
        io::stdin().read_line(&mut custom_path)
            .context("Failed to read custom path")?;
        custom_path.trim().to_string()
    } else {
        anyhow::bail!("Invalid choice");
    };

    // Ensure path starts with root folder name
    if !selected_path.starts_with(ROOT_FOLDER_NAME) {
        anyhow::bail!("Path must start with '{}'", ROOT_FOLDER_NAME);
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
        Some(format!("{}/{}/{}/{}", ROOT_FOLDER_NAME, parts[0], parts[1], file_name))
    } else {
        None
    }
}

fn create_folder_structure(path: &str, root_folder_id: &str) -> Result<String> {
    // Check if path starts with root folder name
    let root_prefix = format!("{}/", ROOT_FOLDER_NAME);
    if !path.starts_with(&root_prefix) {
        anyhow::bail!("Invalid path format: must start with '{}'", root_prefix);
    }
    
    // Remove root folder prefix and split the remaining path
    let remaining_path = &path[root_prefix.len()..];
    let parts: Vec<&str> = remaining_path.split('/').filter(|s| !s.is_empty()).collect();
    
    // Start with the existing root folder ID
    let mut current_path = String::from(ROOT_FOLDER_NAME);
    let mut current_folder_id = root_folder_id.to_string();

    // Create each folder in the path hierarchy, excluding the last part (filename)
    for folder_name in &parts[..parts.len().saturating_sub(1)] {
        current_path.push('/');
        current_path.push_str(folder_name);
        // Ensure this complete path exists as a folder
        current_folder_id = ensure_subfolder_with_full_path(&current_path, &current_folder_id)?;
    }
    Ok(current_folder_id)
}

fn ensure_subfolder_with_full_path(full_path: &str, _parent_folder_id: &str) -> Result<String> {
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
    
    // Check if folder with this exact full path already exists
    for folder in &folders {
        if let (Some(name), Some(id)) = (folder["name"].as_str(), folder["id"].as_str()) {
            if name == full_path {
                return Ok(id.to_string());
            }
        }
    }
    
    // Folder doesn't exist, create it
    println!("Creating folder '{}'...", full_path);
    
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
    
    // Use the full path as the folder name
    template["name"] = serde_json::Value::String(full_path.to_string());
    
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