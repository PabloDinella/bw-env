# Bitwarden .env CLI Wrapper

A Rust CLI tool to store and retrieve `.env` files in Bitwarden using the Bitwarden CLI (`bw`).

## Features
- Store `.env` files in Bitwarden as a secure note in a path like `your-repo-name/path/to/.env` (or custom path)
- Retrieve `.env` files easily with automatic path-based lookup
- List all stored `.env` files
- Automatic Bitwarden vault synchronization

## Requirements
- [Bitwarden CLI](https://bitwarden.com/help/cli/) (`bw`) must be installed and logged in

## Usage

The tool prompts you to choose how to structure the item name, then stores that path structure directly in the item name.

```sh
# Store a .env file (prompts for path structure to use in item name)
bw-env store path/to/.env

# Retrieve a .env file (searches by filename only, e.g., ".env")
bw-env retrieve --output path/to/.env

# List all stored .env files (shows folder, dates, and Bitwarden vault link)
bw-env list
```

### Examples

```sh
# Store .env file with path structure prompting
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
```

## Development

```sh
cargo build
cargo run -- --help
```
