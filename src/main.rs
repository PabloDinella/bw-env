mod auth;
mod commands;
mod cli;
mod bw_commands;

use clap::Parser;
use anyhow::Result;
use cli::{Cli, Commands};
use auth::unlock_vault;
use commands::{store_env, retrieve_env, list_env_items};

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Unlock the vault first to avoid multiple password prompts
    unlock_vault()?;
    
    match cli.command {
        Commands::Store { path, create_folder_structure } => store_env(&path, create_folder_structure)?,
        Commands::Retrieve { output, create_folder_structure } => retrieve_env(&output, create_folder_structure)?,
        Commands::List => list_env_items()?,
    }
    Ok(())
}
