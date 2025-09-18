use anyhow::{Result, Context};
use std::process::Command;

pub fn unlock_vault() -> Result<()> {
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