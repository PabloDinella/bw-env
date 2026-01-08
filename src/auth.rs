use anyhow::{Context, Result};
use std::io::ErrorKind;
use std::process::{Command, Stdio};

/// Helper function to execute a bw command, falling back to flatpak if needed
pub fn run_bw_command(args: &[&str]) -> Result<std::process::Output> {
    match Command::new("bw")
        .args(args)
        .output()
    {
        Ok(output) => Ok(output),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            // Fall back to flatpak version
            let mut flatpak_args = vec!["run", "--command=bw", "com.bitwarden.desktop"];
            flatpak_args.extend_from_slice(args);
            Command::new("flatpak")
                .args(&flatpak_args)
                .output()
                .context("Failed to run bw command via flatpak")
        }
        Err(err) => Err(err).context("Failed to run bw command"),
    }
}

/// Helper function to execute an interactive bw command with inherited stdio
pub fn run_bw_command_interactive(args: &[&str]) -> Result<std::process::ExitStatus> {
    match Command::new("bw")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
    {
        Ok(status) => Ok(status),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            // Fall back to flatpak version
            let mut flatpak_args = vec!["run", "--command=bw", "com.bitwarden.desktop"];
            flatpak_args.extend_from_slice(args);
            Command::new("flatpak")
                .args(&flatpak_args)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .context("Failed to run bw command via flatpak")
        }
        Err(err) => Err(err).context("Failed to run bw command"),
    }
}

/// Helper function to execute a bw command with piped stdin/stdout
pub fn run_bw_command_piped(args: &[&str]) -> Result<std::process::Child> {
    match Command::new("bw")
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        Ok(child) => Ok(child),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            // Fall back to flatpak version
            let mut flatpak_args = vec!["run", "--command=bw", "com.bitwarden.desktop"];
            flatpak_args.extend_from_slice(args);
            Command::new("flatpak")
                .args(&flatpak_args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .context("Failed to spawn bw command via flatpak")
        }
        Err(err) => Err(err).context("Failed to spawn bw command"),
    }
}

pub fn unlock_vault() -> Result<()> {
    ensure_bw_cli_available()?;
    ensure_logged_in()?;

    println!("Unlocking Bitwarden vault...");
    let status = run_bw_command(&["unlock", "--check"])?;
    
    if !status.status.success() {
        // Vault is locked, need to unlock
        let unlock_status = run_bw_command_interactive(&["unlock", "--raw"])?;
        
        if !unlock_status.success() {
            anyhow::bail!("Failed to unlock Bitwarden vault");
        }
        
        // Get the session key
        let unlock_output = run_bw_command(&["unlock", "--raw"])?;
        
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
    let login_check = run_bw_command(&["login", "--check"])?;

    if login_check.status.success() {
        return Ok(());
    }

    println!("No active Bitwarden login found. Starting 'bw login'...");
    
    let login_status = run_bw_command_interactive(&["login"])?;

    if login_status.success() {
        println!("Login completed successfully.");
        Ok(())
    } else {
        anyhow::bail!("Bitwarden login failed. Please re-run and complete authentication.");
    }
}

/// Verify that the Bitwarden CLI binary is available before issuing commands.
pub fn ensure_bw_cli_available() -> Result<()> {
    // Try regular bw first
    let status = Command::new("bw")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match status {
        Ok(status) if status.success() => return Ok(()),
        Ok(_) => {
            // bw exists but failed, try flatpak as fallback
            let flatpak_check = Command::new("flatpak")
                .args(&["run", "--command=bw", "com.bitwarden.desktop", "--version"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            
            return match flatpak_check {
                Ok(status) if status.success() => Ok(()),
                Ok(_) => anyhow::bail!(
                    "Bitwarden CLI is installed but '--version' failed. Reinstall bw or the flatpak version and ensure it is accessible."
                ),
                Err(err) if err.kind() == ErrorKind::NotFound => anyhow::bail!(
                    "Bitwarden CLI 'bw' is not installed or not on your PATH, and Bitwarden flatpak 'com.bitwarden.desktop' is not installed. Install one of them: https://bitwarden.com/help/cli/ or flatpak install flathub com.bitwarden.desktop"
                ),
                Err(err) => Err(err).context("Failed to check for Bitwarden CLI"),
            };
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
            // bw not found, try flatpak as fallback
            let flatpak_check = Command::new("flatpak")
                .args(&["run", "--command=bw", "com.bitwarden.desktop", "--version"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            
            return match flatpak_check {
                Ok(status) if status.success() => Ok(()),
                Ok(_) => anyhow::bail!(
                    "Bitwarden flatpak 'com.bitwarden.desktop' is installed but '--version' failed. Reinstall it."
                ),
                Err(err) if err.kind() == ErrorKind::NotFound => anyhow::bail!(
                    "Bitwarden CLI 'bw' is not installed or not on your PATH, and Bitwarden flatpak 'com.bitwarden.desktop' is not installed. Install one of them: https://bitwarden.com/help/cli/ or flatpak install flathub com.bitwarden.desktop"
                ),
                Err(err) => Err(err).context("Failed to check for Bitwarden CLI via flatpak"),
            };
        }
        Err(err) => Err(err).context("Failed to check for Bitwarden CLI"),
    }
}