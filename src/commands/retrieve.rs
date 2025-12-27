use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use crate::bw_commands::{find_folder_by_name, sync_vault};

const ROOT_FOLDER_NAME: &str = "bw-env";

pub fn retrieve_env() -> Result<()> {
    sync_vault()?;

    let folder_id = find_folder_by_name(ROOT_FOLDER_NAME)?
        .ok_or_else(|| anyhow!("No '{}' folder found in Bitwarden", ROOT_FOLDER_NAME))?;

    let items = list_items(&folder_id)?;
    if items.is_empty() {
        println!("No .env items found in '{}' folder.", ROOT_FOLDER_NAME);
        return Ok(());
    }

    println!("Select a .env item to download to the current directory:\n");
    for (idx, item) in items.iter().enumerate() {
        let name = item["name"].as_str().unwrap_or("(unnamed)");
        println!("{}. {}", idx + 1, name);
    }

    print!("\nEnter your choice: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read selection")?;

    let choice: usize = input
        .trim()
        .parse()
        .context("Invalid choice, please enter a number")?;

    if choice == 0 || choice > items.len() {
        anyhow::bail!("Selection out of range");
    }

    let selected = &items[choice - 1];
    let item_id = selected["id"].as_str().ok_or_else(|| anyhow!("Missing item id"))?;
    let raw_name = selected["name"].as_str().unwrap_or("env");
    let file_name = sanitize_filename(raw_name);
    let output_path = PathBuf::from(file_name.clone());

    let output_json = Command::new("bw")
        .args(["get", "item", item_id])
        .output()
        .context("Failed to execute Bitwarden CLI")?;

    if !output_json.status.success() {
        anyhow::bail!("Bitwarden CLI failed to retrieve item '{}'.", raw_name);
    }

    let json: serde_json::Value = serde_json::from_slice(&output_json.stdout)
        .context("Failed to parse Bitwarden item JSON")?;

    let notes = json["notes"].as_str().unwrap_or("");
    fs::write(&output_path, notes)
        .with_context(|| format!("Failed to write .env file to {:?}", output_path))?;

    println!("Stored folder: '{}'", ROOT_FOLDER_NAME);
    println!("Downloaded item: '{}' -> {:?}", raw_name, output_path);
    Ok(())
}

fn list_items(folder_id: &str) -> Result<Vec<serde_json::Value>> {
    let items_output = Command::new("bw")
        .args(["list", "items", "--folderid", folder_id])
        .output()
        .context("Failed to list items in Bitwarden folder")?;

    if !items_output.status.success() {
        anyhow::bail!("Failed to list items for folder '{}'.", folder_id);
    }

    let items: Vec<serde_json::Value> = serde_json::from_slice(&items_output.stdout)
        .context("Failed to parse items JSON")?;

    Ok(items)
}

fn sanitize_filename(name: &str) -> String {
    let mut sanitized = name.replace(['/', '\\'], "_");
    if sanitized.is_empty() {
        sanitized = "env".to_string();
    }
    sanitized
}

