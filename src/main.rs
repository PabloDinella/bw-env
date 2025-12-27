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
        Commands::Store { path } => store_env(&path)?,
        Commands::Retrieve { output } => retrieve_env(&output)?,
        Commands::List => list_env_items()?,
    }
    Ok(())
}
