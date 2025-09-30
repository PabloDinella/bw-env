use anyhow::{Result, Context};
use std::process::Command;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use crate::bw_commands::{sync_vault, ensure_folder_exists, create_item};

// Configuration: Root folder name in Bitwarden
const ROOT_FOLDER_NAME: &str = "bw-env";

/// Generate an item name from the file path
fn generate_item_name(file_path: &str, use_full_path: bool) -> Result<String> {
    let path = Path::new(file_path);
    
    if use_full_path {
        // Just use the filename as before when using folder structure
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("env-file");
        Ok(file_name.to_string())
    } else {
        // Use the full path relative to current directory as item name
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
}

pub fn store_env(path: &str, create_folder_structure: bool) -> Result<()> {
    // Sync with Bitwarden server before storing
    sync_vault()?;
    
    let env_content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read .env file at {}", path))?;
    
    // Check for or create root folder (still need this as a container)
    let root_folder_id = ensure_folder_exists(ROOT_FOLDER_NAME)?;
    
    let item_name = if create_folder_structure {
        // Use the old behavior with actual folder structure
        let filename = generate_item_name(path, true)?;
        
        // Get the desired folder structure from user and create folders
        let target_folder_id = get_target_folder(path, &root_folder_id)?;
        
        // Create the item in the folder structure
        create_item(&filename, &env_content, &target_folder_id)?;
        
        println!(".env file stored in Bitwarden as '{}' in folder structure.", filename);
        return Ok(());
    } else {
        // New behavior: prompt for path structure but put it in item name
        get_item_name_with_path(path)?
    };
    
    // Create the item directly in the root folder with the chosen path in the name
    create_item(&item_name, &env_content, &root_folder_id)?;
    
    println!(".env file stored in Bitwarden as '{}'.", item_name);
    Ok(())
}



fn get_item_name_with_path(file_path: &str) -> Result<String> {
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
    
    // Option 3: Just the filename
    let filename_option = file_name.to_string();
    
    // Display options to user
    println!("\nChoose the path structure for the item name:");
    
    let mut option_num = 1;
    
    if let Some(ref git_path) = git_option {
        println!("{}. {} (Git repository structure)", option_num, git_path);
        option_num += 1;
    }
    
    println!("{}. {} (Directory name)", option_num, dir_option);
    let dir_option_num = option_num;
    option_num += 1;
    
    println!("{}. {} (Just filename)", option_num, filename_option);
    let filename_option_num = option_num;
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
    
    let selected_name = if git_option.is_some() && choice == 1 {
        // Remove the ROOT_FOLDER_NAME prefix for the item name
        let git_path = git_option.unwrap();
        git_path.strip_prefix(&format!("{}/", ROOT_FOLDER_NAME))
            .unwrap_or(&git_path)
            .to_string()
    } else if choice == dir_option_num {
        // Remove the ROOT_FOLDER_NAME prefix for the item name
        dir_option.strip_prefix(&format!("{}/", ROOT_FOLDER_NAME))
            .unwrap_or(&dir_option)
            .to_string()
    } else if choice == filename_option_num {
        filename_option
    } else if choice == custom_option_num {
        print!("Enter custom path (without '{}/' prefix): ", ROOT_FOLDER_NAME);
        io::stdout().flush().unwrap();
        let mut custom_path = String::new();
        io::stdin().read_line(&mut custom_path)
            .context("Failed to read custom path")?;
        custom_path.trim().to_string()
    } else {
        anyhow::bail!("Invalid choice");
    };

    Ok(selected_name)
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
        current_folder_id = ensure_folder_exists(&current_path)?;
    }
    Ok(current_folder_id)
}

