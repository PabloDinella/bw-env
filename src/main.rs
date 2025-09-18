mod auth;
mod commands;
mod cli;

use clap::Parser;
use anyhow::Result;
use cli::{Cli, Commands};
use auth::unlock_vault;
use commands::{store_env, retrieve_env};

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
