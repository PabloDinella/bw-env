# Bitwarden .env CLI Wrapper

A Rust CLI tool to store and retrieve `.env` files in Bitwarden using the Bitwarden CLI (`bw`).

## Features
- Store a `.env` file in Bitwarden
- Retrieve a `.env` file from Bitwarden

## Requirements
- [Bitwarden CLI](https://bitwarden.com/help/cli/) (`bw`) must be installed and logged in
- Rust toolchain (install via [rustup](https://rustup.rs/))

## Usage

```sh
# Store a .env file
bw-env store path/to/.env --item-name "My Project ENV"

# Retrieve a .env file
bw-env retrieve --item-name "My Project ENV" --output path/to/.env
```

## Development

```sh
cargo build
cargo run -- --help
```
