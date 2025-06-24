use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration structure for the keys CLI
#[derive(Debug, Deserialize)]
pub struct Config {
    /// URL of the keys server
    #[serde(default = "default_server_url")]
    pub server_url: String,
    // Add more config options here as needed
}

fn default_server_url() -> String {
    "http://localhost:8000".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_url: default_server_url(),
        }
    }
}

/// Load configuration from file or return default if not found
pub fn load_config(config_path: Option<&str>) -> Result<Config> {
    // Try to use the provided config path if specified
    if let Some(path) = config_path {
        let config_file = Path::new(path);
        if config_file.exists() {
            return load_config_from_path(config_file);
        } else {
            println!("Warning: Specified config file not found at {path}");
            // Fall back to default config if specified file doesn't exist
            return Ok(Config::default());
        }
    }

    // Try to load from default locations
    if let Some(config_path) = get_default_config_path() {
        if config_path.exists() {
            return load_config_from_path(&config_path);
        }
    }

    // If no config file found, return default config
    Ok(Config::default())
}

/// Get the default config file path
pub fn get_default_config_path() -> Option<PathBuf> {
    // Use the directories crate to find the standard config directory
    // We use "io.github" as a namespace to avoid conflicts with other applications
    if let Some(proj_dirs) = ProjectDirs::from("io.github", "danielemery", "keys") {
        let config_dir = proj_dirs.config_dir();
        let config_file = config_dir.join("config.toml");
        return Some(config_file);
    }
    None
}

/// Load configuration from a specific path
fn load_config_from_path(path: &Path) -> Result<Config> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config: Config = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse TOML config from: {}", path.display()))?;

    Ok(config)
}

/// Creates a default config file at the default location if it doesn't exist
pub fn ensure_default_config_exists() -> Result<PathBuf> {
    if let Some(config_path) = get_default_config_path() {
        if !config_path.exists() {
            // Create parent directory if it doesn't exist
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create config directory: {}", parent.display())
                })?;
            }

            // Write default config to file
            let default_config = format!(
                "# Keys CLI Configuration\n\n# Server URL (default: http://localhost:8000)\nserver_url = \"{}\"\n",
                default_server_url()
            );

            fs::write(&config_path, default_config).with_context(|| {
                format!(
                    "Failed to write default config to: {}",
                    config_path.display()
                )
            })?;

            println!("Created default config file at: {}", config_path.display());
        }
        return Ok(config_path);
    }

    Err(anyhow::anyhow!("Could not determine default config path"))
}
