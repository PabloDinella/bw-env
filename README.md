# Bitwarden .env CLI Wrapper

A Rust CLI tool to store and retrieve `.env` files in Bitwarden using the Bitwarden CLI (`bw`).

## Features
- Store `.env` files in Bitwarden with path-based naming (default) or folder structure organization
- Retrieve `.env` files by filename with automatic path-based lookup
- List all stored `.env` files with metadata
- Support for both simple path-based storage and complex folder hierarchies
- Automatic Bitwarden vault synchronization

## Requirements
- [Bitwarden CLI](https://bitwarden.com/help/cli/) (`bw`) must be installed and logged in

## Usage

### Default Behavior (Recommended)

By default, the tool prompts you to choose how to structure the item name, then stores that path structure directly in the item name rather than creating complex folder structures in Bitwarden.

```sh
# Store a .env file (prompts for path structure to use in item name)
bw-env store path/to/.env

# Retrieve a .env file (searches by filename only, e.g., ".env")
bw-env retrieve --output path/to/.env

# List all stored .env files (shows folder, dates, and Bitwarden vault link)
bw-env list
```

### Folder Structure Mode

Use the `--create-folder-structure` flag to enable the legacy behavior that creates actual folder hierarchies in Bitwarden:

```sh
# Store a .env file with folder structure (prompts for folder organization)
bw-env store path/to/.env --create-folder-structure

# Retrieve a .env file using folder structure lookup
bw-env retrieve --output path/to/.env --create-folder-structure
```

### Examples

```sh
# Store .env file with path structure prompting (default mode)
bw-env store frontend/.env
# Prompts:
# 1. github-user/repo-name/.env (Git repository structure)
# 2. project-folder/.env (Directory name) 
# 3. .env (Just filename)
# 4. Custom path (you type the path)
# Choose option and item gets stored with that name

# Store different files with chosen structures
bw-env store backend/api/.env       # You choose: "backend/api/.env" or "api/.env" etc.
bw-env store config/local.env       # You choose: "config/local.env" or "local.env" etc.

# Retrieve any of them by just specifying the desired output path
bw-env retrieve --output .env       # Finds and retrieves any item ending with ".env"
bw-env retrieve --output local.env  # Finds and retrieves any item ending with "local.env"

# Use folder structure mode for organized storage with actual folders (legacy behavior)
bw-env store .env --create-folder-structure  # Creates actual folders in Bitwarden
```

## Development

```sh
cargo build
cargo run -- --help
```
