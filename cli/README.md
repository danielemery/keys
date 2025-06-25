# Keys CLI

A command-line interface for interacting with the keys server.

## Features

- Fetch keys from the server
- Colorized columnar output for better readability
- Raw mode for scripting and automation
- Configuration file support for default server URL
- Safely update `authorized_keys` files
- TODO: Filter keys by user or tag (exlusions or inclusions)
- TODO: Safely update `known_hosts` files
- TODO: Package for nix

## Usage

```bash
# Fetch SSH keys using default server from config or localhost
keys-cli keys

# Fetch SSH keys with explicit server
keys-cli --server http://localhost:8000 keys

# Write SSH keys to authorized_keys file (only adds new keys by default)
keys-cli keys --write ~/.ssh/authorized_keys

# Force overwrite authorized_keys file with server keys
keys-cli keys --write ~/.ssh/authorized_keys --force

# Initialize default config file
keys-cli init

# Use a custom config file
keys-cli --config ~/.keys-cli-config.toml keys

# Display help for the whole CLI
keys-cli --help

# Display help for a specific subcommand
keys-cli keys --help
```

## Building

```bash
cd cli
cargo build --release
```

The compiled binary will be available in `target/release/keys-cli`.

## Configuration

The CLI supports reading configuration from a TOML file. By default, it looks
for configuration in:

```
~/.config/keys-cli/config.toml  # On Linux/macOS
%APPDATA%\keys-cli\config.toml  # On Windows
```

You can initialize a default config file using the `init` command:

```bash
keys-cli init
```

The configuration file format is:

```toml
# Keys CLI Configuration

# Server URL (default: http://localhost:8000)
server_url = "https://keys.example.com"
```

You can also specify a custom config file location:

```bash
keys-cli --config /path/to/config.toml keys
```

Command-line options take precedence over configuration file settings.

## Safely Updating authorized_keys

The CLI can safely update your SSH `authorized_keys` file with keys from the
server:

```bash
# Only add new keys from server, preserving existing keys
keys-cli keys --write ~/.ssh/authorized_keys

# Replace all keys with the server's keys
keys-cli keys --write ~/.ssh/authorized_keys --force
```

By default (without `--force`), the CLI will:

1. Preserve all existing keys in the file
2. Add any new keys from the server
3. Never remove keys that are in the file but not on the server

This is designed to be safe for automation (e.g., in a cron job) as it won't
lock you out of your server if the keys server is down or returns incomplete
results.

When `--force` is used, the file will be completely replaced with the keys from
the server.

## Development

To test the CLI during development, you can run:

```bash
# Run with the keys subcommand
cargo run -- --server http://localhost:8000 keys
```
