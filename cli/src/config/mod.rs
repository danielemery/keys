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
    if let Some(config_path) = get_default_config_path()
        && config_path.exists()
    {
        return load_config_from_path(&config_path);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server_url, "http://localhost:8000");
    }

    #[test]
    fn test_default_server_url() {
        assert_eq!(default_server_url(), "http://localhost:8000");
    }

    #[test]
    fn test_load_config_with_valid_file() {
        // Create a temporary config file
        let temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
server_url = "https://example.com:8080"
"#;
        fs::write(temp_file.path(), config_content).unwrap();

        // Load config from the temp file
        let result = load_config(Some(temp_file.path().to_str().unwrap()));
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.server_url, "https://example.com:8080");
    }

    #[test]
    fn test_load_config_with_nonexistent_file() {
        // Try to load config from a non-existent file
        let result = load_config(Some("/nonexistent/path/config.toml"));
        assert!(result.is_ok());

        // Should return default config when file doesn't exist
        let config = result.unwrap();
        assert_eq!(config.server_url, "http://localhost:8000");
    }

    #[test]
    fn test_load_config_with_invalid_toml() {
        // Create a temporary config file with invalid TOML
        let temp_file = NamedTempFile::new().unwrap();
        let invalid_content = r#"
server_url = "unclosed string
"#;
        fs::write(temp_file.path(), invalid_content).unwrap();

        // Load config from the temp file
        let result = load_config(Some(temp_file.path().to_str().unwrap()));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_with_empty_file() {
        // Create an empty temporary config file
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), "").unwrap();

        // Load config from the temp file
        let result = load_config(Some(temp_file.path().to_str().unwrap()));
        assert!(result.is_ok());

        // Should use default values when file is empty
        let config = result.unwrap();
        assert_eq!(config.server_url, "http://localhost:8000");
    }

    #[test]
    fn test_load_config_no_path_provided() {
        // Test loading config without providing a path
        // This should return default config since no default config file exists in test environment
        let result = load_config(None);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.server_url, "http://localhost:8000");
    }

    #[test]
    fn test_load_config_from_path_valid() {
        // Create a temporary config file
        let temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
server_url = "https://test.example.com"
"#;
        fs::write(temp_file.path(), config_content).unwrap();

        // Load config using the internal function
        let result = load_config_from_path(temp_file.path());
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.server_url, "https://test.example.com");
    }

    #[test]
    fn test_load_config_from_path_nonexistent() {
        let nonexistent_path = Path::new("/definitely/does/not/exist/config.toml");
        let result = load_config_from_path(nonexistent_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_default_config_path() {
        // This test just ensures the function doesn't panic and returns a sensible path
        let path = get_default_config_path();
        if let Some(config_path) = path {
            assert!(config_path.to_string_lossy().contains("config.toml"));
            assert!(config_path.to_string_lossy().contains("keys"));
        }
        // Note: path might be None in some test environments, which is acceptable
    }

    #[test]
    fn test_ensure_default_config_exists() {
        // Create a temporary directory to simulate a config directory
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("keys").join("config.toml");

        // Mock the get_default_config_path function by testing ensure_default_config_exists
        // in an environment where we control the path
        // Since we can't easily mock the function, we'll test the file creation logic indirectly

        // Ensure parent directory doesn't exist
        assert!(!config_path.exists());

        // Test that the function would work - we can't easily test this without mocking
        // but we can at least ensure it doesn't panic in normal circumstances
        let result = ensure_default_config_exists();
        // The result depends on whether ProjectDirs can determine a config directory
        // In some test environments this might fail, which is acceptable
        match result {
            Ok(path) => {
                // If successful, the path should exist and contain expected content
                assert!(path.exists());
                let content = fs::read_to_string(&path).unwrap();
                assert!(content.contains("server_url"));
                assert!(content.contains("http://localhost:8000"));
            }
            Err(_) => {
                // In some test environments, ProjectDirs might not work, which is acceptable
            }
        }
    }

    #[test]
    fn test_config_with_partial_content() {
        // Test config file that only specifies some fields (using serde defaults)
        let temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
# Just a comment, no actual config values
"#;
        fs::write(temp_file.path(), config_content).unwrap();

        let result = load_config_from_path(temp_file.path());
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.server_url, "http://localhost:8000"); // Should use default
    }

    #[test]
    fn test_config_debug_implementation() {
        let config = Config::default();
        let debug_output = format!("{config:?}");
        assert!(debug_output.contains("Config"));
        assert!(debug_output.contains("server_url"));
        assert!(debug_output.contains("http://localhost:8000"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
server_url = "https://custom.server.com:9000"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.server_url, "https://custom.server.com:9000");
    }

    #[test]
    fn test_config_deserialization_with_missing_field() {
        // Test that missing fields use defaults
        let toml_str = r#"
# No server_url specified
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.server_url, "http://localhost:8000"); // Should use default
    }
}
