use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The server URL
    #[arg(short, long, default_value = "http://localhost:8000")]
    server: String,

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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Keys {} => {
            commands::fetch_ssh_keys(&cli.server)?;
        }
        Commands::PgpKeys {} => {
            commands::fetch_pgp_keys(&cli.server)?;
        }
        Commands::KnownHosts {} => {
            commands::fetch_known_hosts(&cli.server)?;
        }
    }

    Ok(())
}
