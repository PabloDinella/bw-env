use crate::bw_commands::get_template::{get_template, TemplateType};
use crate::auth::run_bw_command;
use crate::auth::run_bw_command_piped;
use anyhow::{Context, Result};
use std::io::Write;

/// Create a secure note item in Bitwarden
pub fn create_item(name: &str, notes: &str, folder_id: &str) -> Result<String> {
    println!("Creating item '{}'...", name);

    // Get the item template
    let mut template = get_template(TemplateType::Item)?;

    // Configure the template for a secure note
    template["type"] = serde_json::Value::Number(2.into()); // Secure note type
    template["secureNote"] = serde_json::json!({"type": 0}); // Initialize secureNote object
    template["notes"] = serde_json::Value::String(notes.to_string());
    template["name"] = serde_json::Value::String(name.to_string());
    template["folderId"] = serde_json::Value::String(folder_id.to_string());

    // Convert to JSON string
    let json_str = serde_json::to_string(&template).context("Failed to serialize template")?;

    // Encode the JSON using bw (with automatic flatpak fallback)
    let mut encode_child = run_bw_command_piped(&["encode"])?;

    {
        let stdin = encode_child.stdin.as_mut().unwrap();
        stdin
            .write_all(json_str.as_bytes())
            .context("Failed to write to bw encode stdin")?;
    }

    let encode_result = encode_child
        .wait_with_output()
        .context("Failed to wait for bw encode")?;

    if !encode_result.status.success() {
        anyhow::bail!("Failed to encode item");
    }

    let encoded_data =
        String::from_utf8(encode_result.stdout).context("Failed to parse encoded data")?;

    // Create the item
    let create_output = run_bw_command(&["create", "item", encoded_data.trim()])?;

    if !create_output.status.success() {
        anyhow::bail!("Bitwarden CLI failed to store item");
    }

    // Parse the created item response to get the ID
    let created_item: serde_json::Value = serde_json::from_slice(&create_output.stdout)
        .context("Failed to parse created item JSON")?;

    if let Some(id) = created_item["id"].as_str() {
        println!("Created item '{}' successfully with ID: {}", name, id);
        Ok(id.to_string())
    } else {
        anyhow::bail!("Failed to get item ID from created item response");
    }
}
