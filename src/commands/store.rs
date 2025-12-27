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

    // Prompt for path structure but put it in item name
    let item_name = get_item_name_with_path(path)?;

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

    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

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
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read user input")?;

    let choice: u32 = input
        .trim()
        .parse()
        .context("Invalid choice, please enter a number")?;

    let selected_name = if git_option.is_some() && choice == 1 {
        // Remove the ROOT_FOLDER_NAME prefix for the item name
        let git_path = git_option.unwrap();
        git_path
            .strip_prefix(&format!("{}/", ROOT_FOLDER_NAME))
            .unwrap_or(&git_path)
            .to_string()
    } else if choice == dir_option_num {
        // Remove the ROOT_FOLDER_NAME prefix for the item name
        dir_option
            .strip_prefix(&format!("{}/", ROOT_FOLDER_NAME))
            .unwrap_or(&dir_option)
            .to_string()
    } else if choice == filename_option_num {
        filename_option
    } else if choice == custom_option_num {
        print!(
            "Enter custom path (without '{}/' prefix): ",
            ROOT_FOLDER_NAME
        );
        io::stdout().flush().unwrap();
        let mut custom_path = String::new();
        io::stdin()
            .read_line(&mut custom_path)
            .context("Failed to read custom path")?;
        custom_path.trim().to_string()
    } else {
        anyhow::bail!("Invalid choice");
    };

    Ok(selected_name)
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
        Some(format!(
            "{}/{}/{}/{}",
            ROOT_FOLDER_NAME, parts[0], parts[1], file_name
        ))
    } else {
        None
    }
}
