use anyhow::{Context, Result};
use std::io::ErrorKind;
use std::process::{Command, Stdio};

pub fn unlock_vault() -> Result<()> {
    ensure_bw_cli_available()?;
    ensure_logged_in()?;

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

/// Ensure the user is logged into Bitwarden; if not, start the interactive login flow.
pub fn ensure_logged_in() -> Result<()> {
    let login_check = Command::new("bw")
        .arg("login")
        .arg("--check")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("Failed to check Bitwarden login state")?;

    if login_check.success() {
        return Ok(());
    }

    println!("No active Bitwarden login found. Starting 'bw login'...");
    let login_status = Command::new("bw")
        .arg("login")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run Bitwarden login")?;

    if login_status.success() {
        println!("Login completed successfully.");
        Ok(())
    } else {
        anyhow::bail!("Bitwarden login failed. Please re-run and complete authentication.");
    }
}

/// Verify that the Bitwarden CLI binary is available before issuing commands.
pub fn ensure_bw_cli_available() -> Result<()> {
    let status = Command::new("bw")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => anyhow::bail!(
            "Bitwarden CLI 'bw' is installed but '--version' failed. Reinstall bw and ensure it is on your PATH."
        ),
        Err(err) if err.kind() == ErrorKind::NotFound => anyhow::bail!(
            "Bitwarden CLI 'bw' is not installed or not on your PATH. Install it from https://bitwarden.com/help/cli/."
        ),
        Err(err) => Err(err).context("Failed to check for Bitwarden CLI"),
    }
}