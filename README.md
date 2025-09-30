# Bitwarden .env CLI Wrapper

A Rust CLI tool to store and retrieve `.env` files in Bitwarden using the Bitwarden CLI (`bw`).

## Features
- Store a `.env` file in Bitwarden
- Retrieve a `.env` file from Bitwarden

## Requirements
- [Bitwarden CLI](https://bitwarden.com/help/cli/) (`bw`) must be installed and logged in

## Usage

```sh
# Store a .env file (item name will be the filename, e.g., ".env")
bw-env store path/to/.env

# Retrieve a .env file (item name will be the filename, e.g., ".env")
bw-env retrieve --output path/to/.env
```

## Development

```sh
cargo build
cargo run -- --help
```
