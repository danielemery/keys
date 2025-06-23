# Keys CLI

A command-line interface for interacting with the keys server.

## Features

- Fetch keys from the server
- Colorized columnar output for better readability
- Raw mode for scripting and automation
- TODO: Filter keys by user or tag (exlusions or inclusions)
- TODO: Support config file for default server URL
- TODO: Support for PGP keys
- TODO: Support for known hosts
- TODO: Safely update `authorized_keys` and `known_hosts` files
- TODO: Package for nix

## Usage

```bash
# Basic usage
keys-cli --server http://localhost:8000

# Display help
keys-cli --help
```

## Building

```bash
cd cli
cargo build --release
```

The compiled binary will be available in `target/release/keys-cli`.

## Development

To test the CLI during development, you can run:

```bash
cargo run -- --server http://localhost:8000
```
