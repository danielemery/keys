use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod config;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The server URL (overrides config file)
    #[arg(short, long)]
    server: Option<String>,

    /// Path to config file (default: ~/.config/keys-cli/config.toml)
    #[arg(short = 'c', long)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Fetch SSH keys from the server
    Keys {},

    /// Fetch PGP keys from the server
    PgpKeys {},

    /// Fetch known hosts from the server
    KnownHosts {},

    /// Initialize a default config file
    Init {},
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration, with CLI-provided path if specified
    let config = config::load_config(cli.config.as_deref())?;

    // CLI server arg takes precedence over config file
    let server_url = cli.server.unwrap_or(config.server_url);

    match &cli.command {
        Commands::Keys {} => {
            commands::fetch_ssh_keys(&server_url)?;
        }
        Commands::PgpKeys {} => {
            commands::fetch_pgp_keys(&server_url)?;
        }
        Commands::KnownHosts {} => {
            commands::fetch_known_hosts(&server_url)?;
        }
        Commands::Init {} => {
            // Create a default config file
            let config_path = config::ensure_default_config_exists()?;
            println!("Configuration file created at: {}", config_path.display());
        }
    }

    Ok(())
}
