use anyhow::{Result, Context};
use std::process::Command;
use std::io::Write;
use crate::bw_commands::get_template::get_item_template;

/// Create a secure note item in Bitwarden
pub fn create_item(name: &str, notes: &str, folder_id: &str) -> Result<String> {
    println!("Creating item '{}'...", name);
    
    // Get the item template
    let mut template = get_item_template()?;
    
    // Configure the template for a secure note
    template["type"] = serde_json::Value::Number(2.into()); // Secure note type
    template["secureNote"] = serde_json::json!({"type": 0}); // Initialize secureNote object
    template["notes"] = serde_json::Value::String(notes.to_string());
    template["name"] = serde_json::Value::String(name.to_string());
    template["folderId"] = serde_json::Value::String(folder_id.to_string());
    
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
    let create_output = Command::new("bw")
        .args(["create", "item", encoded_data.trim()])
        .output()
        .context("Failed to create Bitwarden item")?;
    
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

/// Create a login item in Bitwarden
pub fn create_login_item(name: &str, username: &str, password: &str, folder_id: &str, uris: Option<Vec<String>>) -> Result<String> {
    println!("Creating login item '{}'...", name);
    
    // Get the item template
    let mut template = get_item_template()?;
    
    // Configure the template for a login
    template["type"] = serde_json::Value::Number(1.into()); // Login type
    template["name"] = serde_json::Value::String(name.to_string());
    template["folderId"] = serde_json::Value::String(folder_id.to_string());
    
    // Set login details
    let mut login = serde_json::json!({
        "username": username,
        "password": password
    });
    
    // Add URIs if provided
    if let Some(uri_list) = uris {
        let uris_json: Vec<serde_json::Value> = uri_list
            .into_iter()
            .map(|uri| serde_json::json!({"match": null, "uri": uri}))
            .collect();
        login["uris"] = serde_json::Value::Array(uris_json);
    }
    
    template["login"] = login;
    
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
    let create_output = Command::new("bw")
        .args(["create", "item", encoded_data.trim()])
        .output()
        .context("Failed to create Bitwarden item")?;
    
    if !create_output.status.success() {
        anyhow::bail!("Bitwarden CLI failed to store item");
    }
    
    // Parse the created item response to get the ID
    let created_item: serde_json::Value = serde_json::from_slice(&create_output.stdout)
        .context("Failed to parse created item JSON")?;
    
    if let Some(id) = created_item["id"].as_str() {
        println!("Created login item '{}' successfully with ID: {}", name, id);
        Ok(id.to_string())
    } else {
        anyhow::bail!("Failed to get item ID from created item response");
    }
}