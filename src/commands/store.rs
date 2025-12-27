use crate::bw_commands::{create_item, ensure_folder_exists, sync_vault};
use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

// Configuration: Root folder name in Bitwarden
const ROOT_FOLDER_NAME: &str = "bw-env";

pub fn store_env(path: &str) -> Result<()> {
    // Sync with Bitwarden server before storing
    sync_vault()?;

    let env_content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read .env file at {}", path))?;

    // Check for or create root folder (still need this as a container)
    let root_folder_id = ensure_folder_exists(ROOT_FOLDER_NAME)?;

    // Derive an item name; if git info exists, use that structure, otherwise fallback to filename.
    // Allow a custom item name, but always store in the fixed bw-env folder.
    let item_name = get_item_name_with_path(path)?;

    // Create the item directly in the root folder with the chosen path in the name
    create_item(&item_name, &env_content, &root_folder_id)?;

    println!("Stored folder: '{}'", ROOT_FOLDER_NAME);
    println!("Stored item name: '{}'", item_name);
    Ok(())
}

fn get_item_name_with_path(file_path: &str) -> Result<String> {
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    // Prefer git-style path without the bw-env prefix; otherwise use filename. Offer a custom name option; folder remains bw-env via folderId.
    let default_name = if let Some(git_path) = get_git_repo_path(&current_dir, file_name) {
        git_path
    } else {
        file_name.to_string()
    };

    println!("\nChoose the item name:");
    println!("1. {} (default)", default_name);
    println!("2. Custom name");
    println!(); 
    println!("Your item will be stored in 📁{} folder.", ROOT_FOLDER_NAME);
    println!(); 
    print!("Enter your choice: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read user input")?;

    let choice: u32 = input
        .trim()
        .parse()
        .context("Invalid choice, please enter a number")?;

    match choice {
        1 => Ok(default_name),
        2 => {
            print!("Enter custom item name (folder stays 'bw-env'): ");
            io::stdout().flush().unwrap();
            let mut custom = String::new();
            io::stdin()
                .read_line(&mut custom)
                .context("Failed to read custom name")?;
            Ok(custom.trim().to_string())
        }
        _ => anyhow::bail!("Invalid choice"),
    }
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
        remote_url
            .strip_prefix("https://github.com/")?
            .strip_suffix(".git")
            .unwrap_or(remote_url.strip_prefix("https://github.com/")?)
    } else if remote_url.starts_with("git@github.com:") {
        remote_url
            .strip_prefix("git@github.com:")?
            .strip_suffix(".git")
            .unwrap_or(remote_url.strip_prefix("git@github.com:")?)
    } else {
        return None;
    };

    // Split into owner/repo
    let parts: Vec<&str> = repo_info.split('/').collect();
    if parts.len() == 2 {
        // Return path without bw-env prefix; folder is controlled separately via folderId
        Some(format!("{}/{}/{}", parts[0], parts[1], file_name))
    } else {
        None
    }
}
