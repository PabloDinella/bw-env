use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bw-env")]
#[command(about = "Store and retrieve .env files in Bitwarden via CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Store a .env file in Bitwarden
    Store {
        /// Path to the .env file
        path: String,
        /// Create folder structure in Bitwarden (default: store path in item name)
        #[arg(long)]
        create_folder_structure: bool,
    },
    /// Retrieve a .env file from Bitwarden
    Retrieve {
        /// Output path for the .env file
        #[arg(long)]
        output: String,
        /// Use folder structure for item lookup (default: use filename only)
        #[arg(long)]
        create_folder_structure: bool,
    },
    /// List all .env files stored in Bitwarden
    List,
}