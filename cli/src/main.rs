use anyhow::Result;
use clap::{Parser, Subcommand};

use keys::{commands, config};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The server URL (overrides config file)
    #[arg(short, long, global = true)]
    server: Option<String>,

    /// Path to config file (default: ~/.config/keys/config.toml)
    #[arg(short = 'c', long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Fetch or write SSH keys from the server
    Ssh {
        /// Write keys to authorized_keys file
        #[arg(short, long)]
        write: Option<String>,

        /// Force overwrite existing keys (default is to only add new keys)
        #[arg(short, long)]
        force: bool,
    },

    /// Fetch PGP keys from the server, or import them into your local GnuPG keyring
    Pgp {
        /// Import the fetched PGP keys into your local GnuPG keyring (requires the `gpg` executable)
        #[arg(short, long)]
        import: bool,
    },

    /// Fetch known hosts from the server
    KnownHosts {
        /// Write known hosts to a file (adds new entries, preserving existing ones)
        #[arg(short, long)]
        write: Option<String>,

        /// Force overwrite the file with the server's entries (default is to only add new entries)
        #[arg(short, long)]
        force: bool,
    },

    /// Initialize a default config file
    Init {},
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // `init` creates the config file rather than reading it, so handle it before
    // the normal config-loading flow and target the user-selected path (if any).
    if let Commands::Init {} = cli.command {
        let config_path = config::ensure_config_exists(cli.config.as_deref())?;
        println!("Configuration file created at: {}", config_path.display());
        return Ok(());
    }

    // Load configuration, with CLI-provided path if specified
    let config = config::load_config(cli.config.as_deref())?;

    // CLI server arg takes precedence over config file
    let server_url = cli.server.unwrap_or(config.server_url);

    match &cli.command {
        Commands::Ssh { write, force } => {
            if let Some(path) = write {
                commands::ssh_keys::write_ssh_keys(&server_url, path, *force)?;
            } else {
                commands::ssh_keys::fetch_ssh_keys(&server_url)?;
            }
        }
        Commands::Pgp { import } => {
            if *import {
                commands::pgp_keys::import_pgp_keys(&server_url)?;
            } else {
                commands::pgp_keys::fetch_pgp_keys(&server_url)?;
            }
        }
        Commands::KnownHosts { write, force } => {
            if let Some(path) = write {
                commands::known_hosts::write_known_hosts(&server_url, path, *force)?;
            } else {
                commands::known_hosts::fetch_known_hosts(&server_url)?;
            }
        }
        // `Init` is handled above, before config loading.
        Commands::Init {} => unreachable!("Init is handled before config loading"),
    }

    Ok(())
}
