use anyhow::{anyhow, Context, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use crate::bw_commands::{find_folder_by_name, sync_vault};

const ROOT_FOLDER_NAME: &str = "bw-env";

pub fn retrieve_env() -> Result<()> {
    sync_vault()?;

    let folder_id = find_folder_by_name(ROOT_FOLDER_NAME)?
        .ok_or_else(|| anyhow!("No '{}' folder found in Bitwarden", ROOT_FOLDER_NAME))?;

    let items = list_items(&folder_id)?;
    if items.is_empty() {
        println!("No .env items found in '{}' folder.", ROOT_FOLDER_NAME);
        return Ok(());
    }

    // Try to infer context (org/repo) from current git remote
    let context_dir = current_context_dir();
    let groups = group_items_by_dir(&items);

    let mut options: Vec<OptionEntry> = Vec::new();
    let mut next_num: usize = 1;

    if let Some(ref dir) = context_dir {
        if let Some(context_indices) = groups.get(dir) {
            let context_set: HashSet<usize> = context_indices.iter().cloned().collect();

            println!("Found items matching {}, select what to download:\n", dir);

            if context_indices.len() >= 2 {
                let mut seen = HashSet::new();
                let segments: Vec<String> = context_indices
                    .iter()
                    .filter_map(|&idx| items[idx]["name"].as_str())
                    .map(|name| name.rsplit(['/', '\\']).next().unwrap_or(name).to_string())
                    .filter(|seg| seen.insert(seg.clone()))
                    .collect();

                let label = format!("Download everything ({})", segments.join(", "));
                options.push(OptionEntry {
                    label: label.clone(),
                    kind: SelectionKind::Group(context_indices.clone()),
                });
                println!("{}. {}", next_num, label);
                next_num += 1;
            }

            println!("\nDownload items individually:\n");
            for &idx in context_indices {
                let name = items[idx]["name"].as_str().unwrap_or("(unnamed)");
                options.push(OptionEntry {
                    label: name.to_string(),
                    kind: SelectionKind::Single(idx),
                });
                println!("{}. {}", next_num, name);
                next_num += 1;
            }

            println!("\nOther items found on your vault:\n");
            for (idx, item) in items.iter().enumerate() {
                if !context_set.contains(&idx) {
                    let name = item["name"].as_str().unwrap_or("(unnamed)");
                    options.push(OptionEntry {
                        label: name.to_string(),
                        kind: SelectionKind::Single(idx),
                    });
                    println!("{}. {}", next_num, name);
                    next_num += 1;
                }
            }
        } else {
            // No matches for context; fallback to grouped listing
            let fallback = build_options(&items);
            println!("Select what to download to the current directory:\n");
            options = fallback;
            for (idx, opt) in options.iter().enumerate() {
                println!("{}. {}", idx + 1, opt.label);
            }
        }
    } else {
        // No context available; fallback to grouped listing
        let fallback = build_options(&items);
        println!("Select what to download to the current directory:\n");
        options = fallback;
        for (idx, opt) in options.iter().enumerate() {
            println!("{}. {}", idx + 1, opt.label);
        }
    }

    print!("\nEnter your choice: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read selection")?;

    let choice: usize = input
        .trim()
        .parse()
        .context("Invalid choice, please enter a number")?;

    if choice == 0 || choice > options.len() {
        anyhow::bail!("Selection out of range");
    }
    let selected = &options[choice - 1];

    match &selected.kind {
        SelectionKind::Single(idx) => {
            let item = &items[*idx];
            let path = download_item(item)?;
            let raw_name = item["name"].as_str().unwrap_or("env");
            println!("Stored folder: '{}'", ROOT_FOLDER_NAME);
            println!("Downloaded item: '{}' -> {:?}", raw_name, path);
        }
        SelectionKind::Group(indices) => {
            println!("Stored folder: '{}'", ROOT_FOLDER_NAME);
            for &idx in indices {
                let item = &items[idx];
                let path = download_item(item)?;
                let raw_name = item["name"].as_str().unwrap_or("env");
                println!("Downloaded item: '{}' -> {:?}", raw_name, path);
            }
        }
    }
    Ok(())
}

struct OptionEntry {
    label: String,
    kind: SelectionKind,
}

enum SelectionKind {
    Single(usize),
    Group(Vec<usize>),
}

fn build_options(items: &[serde_json::Value]) -> Vec<OptionEntry> {
    let groups = group_items_by_dir(items);

    // Determine grouped dir order by first appearance and with >1 items
    let mut seen = HashSet::new();
    let grouped_dirs_order: Vec<String> = items
        .iter()
        .filter_map(|item| item["name"].as_str().and_then(item_dir))
        .filter(|dir| groups.get(dir).map_or(false, |v| v.len() > 1))
        .filter(|dir| seen.insert(dir.clone()))
        .collect();

    // Indices that belong to multi-item groups
    let grouped_indices: HashSet<usize> = grouped_dirs_order
        .iter()
        .flat_map(|dir| groups.get(dir).into_iter().flat_map(|v| v.iter().cloned()))
        .collect();

    // Build options: groups first, then nested singles, then lone singles
    let mut options: Vec<OptionEntry> = Vec::new();

    // Groups with nested
    options.extend(grouped_dirs_order.iter().flat_map(|dir| {
        let indices = groups.get(dir).cloned().unwrap_or_default();
        let mut entries = Vec::with_capacity(indices.len() + 1);
        entries.push(OptionEntry {
            label: format!("{}/* ({} items)", dir, indices.len()),
            kind: SelectionKind::Group(indices.clone()),
        });
        entries.extend(indices.into_iter().map(|idx| {
            let name = items[idx]["name"].as_str().unwrap_or("(unnamed)");
            OptionEntry {
                label: format!("  - {}", name),
                kind: SelectionKind::Single(idx),
            }
        }));
        entries
    }));

    // Singles not part of any multi-item group
    options.extend(
        items
            .iter()
            .enumerate()
            .filter(|(idx, _)| !grouped_indices.contains(idx))
            .map(|(idx, item)| OptionEntry {
                label: item["name"].as_str().unwrap_or("(unnamed)").to_string(),
                kind: SelectionKind::Single(idx),
            }),
    );

    options
}

fn item_dir(name: &str) -> Option<String> {
    name.rfind('/')
        .map(|pos| name[..pos].to_string())
        .filter(|dir| !dir.is_empty())
}

fn download_item(item: &serde_json::Value) -> Result<PathBuf> {
    let item_id = item["id"].as_str().ok_or_else(|| anyhow!("Missing item id"))?;
    let raw_name = item["name"].as_str().unwrap_or("env");
    let file_name = sanitize_filename(raw_name);
    let output_path = PathBuf::from(file_name.clone());

    let output_json = Command::new("bw")
        .args(["get", "item", item_id])
        .output()
        .context("Failed to execute Bitwarden CLI")?;

    if !output_json.status.success() {
        anyhow::bail!("Bitwarden CLI failed to retrieve item '{}'.", raw_name);
    }

    let json: serde_json::Value = serde_json::from_slice(&output_json.stdout)
        .context("Failed to parse Bitwarden item JSON")?;

    let notes = json["notes"].as_str().unwrap_or("");
    fs::write(&output_path, notes)
        .with_context(|| format!("Failed to write .env file to {:?}", output_path))?;

    Ok(output_path)
}

fn group_items_by_dir(items: &[serde_json::Value]) -> HashMap<String, Vec<usize>> {
    items
        .iter()
        .enumerate()
        .fold(HashMap::new(), |mut acc, (idx, item)| {
            if let Some(dir) = item["name"].as_str().and_then(item_dir) {
                acc.entry(dir).or_default().push(idx);
            }
            acc
        })
}

fn current_context_dir() -> Option<String> {
    // Infer org/repo from git remote
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let remote_url = String::from_utf8(output.stdout).ok()?;
    let remote_url = remote_url.trim();

    let repo_info = if remote_url.starts_with("https://github.com/") {
        remote_url
            .strip_prefix("https://github.com/")?
            .strip_suffix(".git")
            .unwrap_or(remote_url.strip_prefix("https://github.com/")?)
            .to_string()
    } else if remote_url.starts_with("git@github.com:") {
        remote_url
            .strip_prefix("git@github.com:")?
            .strip_suffix(".git")
            .unwrap_or(remote_url.strip_prefix("git@github.com:")?)
            .to_string()
    } else {
        return None;
    };

    let parts: Vec<&str> = repo_info.split('/').collect();
    if parts.len() == 2 {
        Some(format!("{}/{}", parts[0], parts[1]))
    } else {
        None
    }
}

fn list_items(folder_id: &str) -> Result<Vec<serde_json::Value>> {
    let items_output = Command::new("bw")
        .args(["list", "items", "--folderid", folder_id])
        .output()
        .context("Failed to list items in Bitwarden folder")?;

    if !items_output.status.success() {
        anyhow::bail!("Failed to list items for folder '{}'.", folder_id);
    }

    let items: Vec<serde_json::Value> = serde_json::from_slice(&items_output.stdout)
        .context("Failed to parse items JSON")?;

    Ok(items)
}

fn sanitize_filename(name: &str) -> String {
    let last_segment = name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(name);

    let mut sanitized = last_segment.replace(['/', '\\'], "_");
    if sanitized.is_empty() {
        sanitized = "env".to_string();
    }
    sanitized
}

