use clap::{Parser, Subcommand};
use std::process::Command;
use std::fs;
use std::io::Write;
use anyhow::{Result, Context};

#[derive(Parser)]
#[command(name = "bw-env")]
#[command(about = "Store and retrieve .env files in Bitwarden via CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Store a .env file in Bitwarden
    Store {
        /// Path to the .env file
        path: String,
        /// Bitwarden item name
        #[arg(long)]
        item_name: String,
    },
    /// Retrieve a .env file from Bitwarden
    Retrieve {
        /// Bitwarden item name
        #[arg(long)]
        item_name: String,
        /// Output path for the .env file
        #[arg(long)]
        output: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Unlock the vault first to avoid multiple password prompts
    unlock_vault()?;
    
    match cli.command {
        Commands::Store { path, item_name } => store_env(&path, &item_name)?,
        Commands::Retrieve { item_name, output } => retrieve_env(&item_name, &output)?,
    }
    Ok(())
}

fn unlock_vault() -> Result<()> {
    println!("Unlocking Bitwarden vault...");
    let status = Command::new("bw")
        .arg("unlock")
        .arg("--check")
        .stdin(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::null())
        .status()
        .context("Failed to check Bitwarden vault status")?;
    
    if !status.success() {
        // Vault is locked, need to unlock
        let unlock_output = Command::new("bw")
            .arg("unlock")
            .arg("--raw")
            .stdin(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()
            .context("Failed to unlock Bitwarden vault")?;
        
        if !unlock_output.status.success() {
            anyhow::bail!("Failed to unlock Bitwarden vault");
        }
        
        let session_key = String::from_utf8(unlock_output.stdout)
            .context("Failed to parse session key")?;
        
        // Set the session key as environment variable
        std::env::set_var("BW_SESSION", session_key.trim());
        println!("Vault unlocked successfully.");
    } else {
        println!("Vault is already unlocked.");
    }
    
    Ok(())
}

fn store_env(path: &str, item_name: &str) -> Result<()> {
    let env_content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read .env file at {}", path))?;
    
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

fn retrieve_env(item_name: &str, output: &str) -> Result<()> {
    let output_json = Command::new("bw")
        .args(["get", "item", item_name])
        .output()
        .context("Failed to execute Bitwarden CLI")?;
    
    if !output_json.status.success() {
        anyhow::bail!("Bitwarden CLI failed to retrieve item");
    }
    
    let json: serde_json::Value = serde_json::from_slice(&output_json.stdout)
        .context("Failed to parse Bitwarden item JSON")?;
    
    let notes = json["notes"].as_str().unwrap_or("");
    fs::write(output, notes)
        .with_context(|| format!("Failed to write .env file to {}", output))?;
    
    println!(".env file retrieved from Bitwarden and saved to '{}'.", output);
    Ok(())
}
