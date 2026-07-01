# Keys CLI

A command-line interface for interacting with the keys server.

## Features

- Fetch and display keys from a keys server
- Raw mode for scripting and automation
- Safely update `authorized_keys` files without risk of losing ssh access
- Import PGP keys directly into your local GnuPG keyring
- Safely update `known_hosts` files without removing existing entries
- TODO: Filter keys by user or tag (exclusions or inclusions)

## Usage

```bash
# Fetch SSH keys using default server from config or localhost
keys ssh

# Fetch SSH keys with explicit server
keys --server http://localhost:8000 ssh

# Fetch PGP keys
keys pgp

# Import PGP keys into your local GnuPG keyring
keys pgp --import

# Fetch Known hosts
keys known-hosts

# Safely add known hosts to a file, preserving existing entries
keys known-hosts --write ~/.ssh/known_hosts

# Display help for the whole CLI
keys --help

# Display help for a specific subcommand
keys ssh --help
```

## Safely Updating authorized_keys

The CLI can safely update your SSH `authorized_keys` file with keys from the
server:

```bash
# Only add new keys from server, preserving existing keys
keys ssh --write ~/.ssh/authorized_keys

# Replace all keys with the server's keys
keys ssh --write ~/.ssh/authorized_keys --force
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

## Safely Updating known_hosts

The `known-hosts` command writes entries with the same safety model as
`ssh --write`:

```bash
# Only add new entries from the server, preserving existing ones
keys known-hosts --write ~/.ssh/known_hosts

# Replace the entire file with the server's entries
keys known-hosts --write ~/.ssh/known_hosts --force
```

By default (without `--force`), existing entries are preserved, new server
entries are appended, and entries that match a server entry have their flags and
comment refreshed. Entries present locally but absent from the server are kept
and reported. Entries are matched on their `hosts key_type key` identity,
ignoring marker flags and comments.

With `--force`, the file is replaced entirely with the server's entries.

## Importing PGP keys into GnuPG

The CLI can fetch the PGP keys from the server and import them straight into
your local GnuPG keyring:

```bash
# Fetch the PGP keys and import them into your keyring
keys pgp --import
```

This pipes the fetched keys into `gpg --import`, so it requires the `gpg`
executable (GnuPG) to be installed and available on your `PATH`. If GnuPG is not
found, the command exits with a clear, actionable error pointing you to
<https://gnupg.org/download/> rather than a raw OS error.

If the server has no PGP keys, the command reports that there is nothing to
import and exits successfully without invoking `gpg`.

## Configuration

The CLI supports reading configuration from a TOML file. By default, it looks
for configuration in:

```text
~/.config/keys/config.toml  # On Linux/macOS
%APPDATA%\keys\config.toml  # On Windows
```

You can initialize a default config file using the `init` command:

```bash
keys init
```

The configuration file format is:

```toml
# Keys CLI Configuration

# Server URL (default: http://localhost:8000)
server_url = "https://keys.example.com"
```

You can also specify a custom config file location:

```bash
keys --config /path/to/config.toml ssh
```

If a `--config` path is given but the file doesn't exist, the CLI exits with an
error rather than falling back to defaults, so a mistyped path is caught instead
of silently using `http://localhost:8000`.

Command-line options take precedence over configuration file settings.

## Building

```bash
cd cli
cargo build --release
```

The compiled binary will be available in `target/release/keys`.

## Installation with Nix

If you have Nix installed, you can build and install the CLI using the flake:

```bash
# Build the CLI
nix build

# Run directly without installing
nix run

# Install to your profile
nix profile install .

# Or install from GitHub
nix profile install github:danielemery/keys
```

### NixOS System Configuration

To install the CLI system-wide on NixOS, add it as a flake input and include it
in your system packages:

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    keys.url = "github:danielemery/keys";
  };

  outputs = { self, nixpkgs, keys }: {
    nixosConfigurations.your-hostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        {
          environment.systemPackages = [
            keys.packages.x86_64-linux.default
          ];
        }
      ];
    };
  };
}
```

The Nix flake will build the Rust binary and make it available as the `keys`
command.

## Development

To test the CLI during development, you can run:

```bash
# Run with the ssh subcommand
cargo run -- --server http://localhost:8000 ssh
```
