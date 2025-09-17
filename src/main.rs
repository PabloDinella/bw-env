use clap::{Parser, Subcommand};
use std::process::Command;
use std::fs;
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
    match cli.command {
        Commands::Store { path, item_name } => store_env(&path, &item_name)?,
        Commands::Retrieve { item_name, output } => retrieve_env(&item_name, &output)?,
    }
    Ok(())
}

fn store_env(path: &str, item_name: &str) -> Result<()> {
    let env_content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read .env file at {}", path))?;
    let status = Command::new("bw")
        .args(["create", "item", "note", &format!("--name={}", item_name), "--notes", &env_content])
        .status()
        .context("Failed to execute Bitwarden CLI")?;
    if !status.success() {
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
