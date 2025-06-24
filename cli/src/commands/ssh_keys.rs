use anyhow::{Context, Result};
use atty;
use colored::Colorize;
use reqwest::header::ACCEPT;
use serde::Deserialize;

use crate::utils::{pretty_print_table, ColumnConfig};

#[derive(Debug, Deserialize)]
pub struct KeysResponse {
    pub version: String,
    pub keys: Vec<SSHKey>,
}

#[derive(Debug, Deserialize)]
pub struct SSHKey {
    pub key: String,
    pub user: String,
    pub name: String,
    pub tags: Vec<String>,
}

/// Function to pretty print the SSH keys with formatted columns and colors
pub fn pretty_print_ssh_keys(keys_response: &KeysResponse) {
    // Find the maximum width for each column for better formatting
    let max_name_len = keys_response
        .keys
        .iter()
        .map(|k| k.name.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let max_user_len = keys_response
        .keys
        .iter()
        .map(|k| k.user.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let max_tags_len = keys_response
        .keys
        .iter()
        .map(|k| k.tags.join(", ").len())
        .max()
        .unwrap_or(4)
        .max(4);

    // Define the columns
    let columns = vec![
        ColumnConfig {
            header: "NAME".to_string(),
            color: |s| s.green(),
            width: max_name_len,
        },
        ColumnConfig {
            header: "USER".to_string(),
            color: |s| s.blue(),
            width: max_user_len,
        },
        ColumnConfig {
            header: "TAGS".to_string(),
            color: |s| s.yellow(),
            width: max_tags_len,
        },
        ColumnConfig {
            header: "KEY".to_string(),
            color: |s| s.red(),
            width: 50, // Key is typically long, so use a reasonable default width
        },
    ];

    // Prepare the rows
    let rows: Vec<Vec<String>> = keys_response
        .keys
        .iter()
        .map(|key| {
            vec![
                key.name.clone(),
                key.user.clone(),
                key.tags.join(", "),
                key.key.clone(),
            ]
        })
        .collect();

    // Use the generic pretty print function
    pretty_print_table(
        "Keys Server Version:",
        &keys_response.version,
        columns,
        rows,
        "No SSH keys found matching the criteria.",
    );
}

/// Private function to fetch SSH keys from the server
///
/// This function handles the HTTP request to the keys server,
/// validates the response, and parses the JSON into a KeysResponse.
///
/// # Arguments
/// * `server_url` - The base URL of the keys server
///
/// # Returns
/// * `Result<KeysResponse>` - The parsed keys response or an error
fn fetch_keys_from_server(server_url: &str) -> Result<KeysResponse> {
    let url = format!("{server_url}/keys");

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header(ACCEPT, "application/json")
        .send()
        .context("Failed to send request to keys server")?;

    let status = response.status();

    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "Server returned error code: {} - {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("Unknown")
        ));
    }

    response
        .json::<KeysResponse>()
        .context("Failed to parse JSON response")
}

pub fn fetch_ssh_keys(server_url: &str) -> Result<()> {
    let keys_response = fetch_keys_from_server(server_url)?;

    // Check if the output is being piped (not connected to a terminal)
    // Use raw/minimal output when piped to another command
    if !atty::is(atty::Stream::Stdout) {
        for key in &keys_response.keys {
            println!("{}", key.key);
        }
        return Ok(());
    }

    // Use the pretty print function for interactive terminal output
    pretty_print_ssh_keys(&keys_response);

    Ok(())
}

pub fn write_ssh_keys(server_url: &str, file_path: &str, force: bool) -> Result<()> {
    // Fetch keys from the server
    let keys_response = fetch_keys_from_server(server_url)?;

    // Expand ~ to home directory if present
    let expanded_path = shellexpand::tilde(file_path);
    let path = std::path::Path::new(expanded_path.as_ref());

    // Read existing authorized_keys file if it exists
    let existing_keys = if path.exists() {
        std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read existing file: {}", path.display()))?
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    // Extract server keys
    let server_keys: Vec<String> = keys_response.keys.iter().map(|k| k.key.clone()).collect();

    // Find keys that are present locally but not on the server
    let local_only_keys: Vec<&String> = existing_keys
        .iter()
        .filter(|key| !server_keys.contains(key))
        .collect();
    let num_local_only = local_only_keys.len();

    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create parent directory: {}", parent.display())
            })?;
        }
    }

    // Define the file content based on the force flag
    let file_content = if force {
        // Force mode: overwrite with server keys
        server_keys.join("\n")
    } else {
        // Safe mode: only add new keys
        let mut combined_keys = existing_keys.clone();

        for key in &server_keys {
            if !existing_keys.contains(key) {
                combined_keys.push(key.clone());
            }
        }

        combined_keys.join("\n")
    };

    // Write to file
    std::fs::write(path, file_content)
        .with_context(|| format!("Failed to write to file: {}", path.display()))?;

    // Count stats
    let num_server_keys = keys_response.keys.len();
    let num_existing = existing_keys.len();
    let num_final = if force {
        num_server_keys
    } else {
        // In additive mode, we need to count unique keys
        let mut combined = existing_keys.clone();
        for key in &server_keys {
            if !combined.contains(key) {
                combined.push(key.clone());
            }
        }
        combined.len()
    };

    // Print a message about what happened
    if force {
        println!(
            "✅ Wrote {} keys to {} (overwriting {} existing keys)",
            num_server_keys,
            path.display(),
            num_existing
        );
    } else {
        let num_added = num_final - num_existing;
        if num_added > 0 {
            println!(
                "✅ Added {} new keys to {} (now {} total keys)",
                num_added,
                path.display(),
                num_final
            );
        } else {
            println!(
                "✅ Keys are already in sync with server at {} ({} total keys)",
                path.display(),
                num_final
            );
        }

        // Print warning about local-only keys if they exist
        if num_local_only > 0 {
            println!(
                "{} {} local keys were not removed (use {} to remove)",
                "⚠️".yellow().bold(),
                num_local_only.to_string().yellow().bold(),
                "--force".yellow().bold()
            );

            // List the first few keys that would be removed
            const MAX_KEYS_TO_SHOW: usize = 3;
            if !local_only_keys.is_empty() {
                let sample_keys = if local_only_keys.len() <= MAX_KEYS_TO_SHOW {
                    local_only_keys
                        .iter()
                        .map(|k| {
                            let trimmed = k.trim();
                            // Extract the comment part which typically contains name/host information
                            let parts: Vec<&str> = trimmed.split_whitespace().collect();
                            if parts.len() >= 3 {
                                // Last part is typically user@host or key name
                                parts[2..].join(" ")
                            } else {
                                // If no comment part, just indicate it's a local key
                                "local key".to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                } else {
                    format!("{num_local_only} keys")
                };

                println!("   Keys that would be removed: {}", sample_keys.yellow());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Helper function to create a temp directory and file
    fn setup_temp_dir_and_file(content: Option<&str>) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("authorized_keys");

        if let Some(content) = content {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "{content}").unwrap();
        }

        (temp_dir, file_path)
    }

    // Helper function to create a mock server
    fn setup_mock_server(response_body: &str) -> (String, mockito::ServerGuard) {
        let mut mock_server = mockito::Server::new();

        mock_server
            .mock("GET", "/keys")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create();

        (mock_server.url(), mock_server)
    }

    // Helper function to create a mock server with error response
    fn setup_mock_server_with_error(
        status_code: usize,
        response_body: &str,
    ) -> (String, mockito::ServerGuard) {
        let mut mock_server = mockito::Server::new();

        mock_server
            .mock("GET", "/keys")
            .with_status(status_code)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create();

        (mock_server.url(), mock_server)
    }

    #[test]
    fn test_write_ssh_keys_force_mode() {
        // Setup mock server
        let mock_response = r#"
        {
            "version": "1.0.0",
            "keys": [
                {"key": "ssh-rsa AAAAB1", "user": "user1", "name": "key1", "tags": ["dev"]},
                {"key": "ssh-rsa AAAAB2", "user": "user2", "name": "key2", "tags": ["prod"]}
            ]
        }
        "#;

        let (server_url, _server) = setup_mock_server(mock_response);

        // Setup existing file with content that should be overwritten
        let existing_content =
            "ssh-rsa AAAABX old_key user@host\nssh-rsa AAAABY another_old_key user@host";
        let (temp_dir, file_path) = setup_temp_dir_and_file(Some(existing_content));

        // Call function with force=true
        let result = write_ssh_keys(&server_url, file_path.to_str().unwrap(), true);
        assert!(result.is_ok(), "write_ssh_keys failed: {:?}", result.err());

        // Verify file contents
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "ssh-rsa AAAAB1\nssh-rsa AAAAB2");

        // Verify it doesn't contain old keys
        assert!(!content.contains("AAAABX"));
        assert!(!content.contains("AAAABY"));

        // Cleanup
        drop(temp_dir);
    }

    #[test]
    fn test_write_ssh_keys_additive_mode() {
        // Setup mock server
        let mock_response = r#"
        {
            "version": "1.0.0",
            "keys": [
                {"key": "ssh-rsa AAAAB1", "user": "user1", "name": "key1", "tags": ["dev"]},
                {"key": "ssh-rsa AAAAB2", "user": "user2", "name": "key2", "tags": ["prod"]}
            ]
        }
        "#;

        let (server_url, _server) = setup_mock_server(mock_response);

        // Setup existing file with one key that's also in the response and one that isn't
        let existing_content = "ssh-rsa AAAAB1\nssh-rsa AAAABZ local_key user@host";
        let (temp_dir, file_path) = setup_temp_dir_and_file(Some(existing_content));

        // Call function with force=false (additive mode)
        let result = write_ssh_keys(&server_url, file_path.to_str().unwrap(), false);
        assert!(result.is_ok(), "write_ssh_keys failed: {:?}", result.err());

        // Verify file contents - should contain both old and new keys
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("ssh-rsa AAAAB1"));
        assert!(content.contains("ssh-rsa AAAAB2")); // New key added
        assert!(content.contains("ssh-rsa AAAABZ")); // Old local key retained

        // Count occurrences of AAAAB1 (should only appear once)
        let count_key1 = content.matches("AAAAB1").count();
        assert_eq!(count_key1, 1, "Duplicate key found: AAAAB1");

        // Cleanup
        drop(temp_dir);
    }

    #[test]
    fn test_write_ssh_keys_new_file() {
        // Setup mock server
        let mock_response = r#"
        {
            "version": "1.0.0",
            "keys": [
                {"key": "ssh-rsa AAAAB1", "user": "user1", "name": "key1", "tags": ["dev"]},
                {"key": "ssh-rsa AAAAB2", "user": "user2", "name": "key2", "tags": ["prod"]}
            ]
        }
        "#;

        let (server_url, _server) = setup_mock_server(mock_response);

        // Don't create an existing file
        let (temp_dir, file_path) = setup_temp_dir_and_file(None);

        // Call function (with either force mode)
        let result = write_ssh_keys(&server_url, file_path.to_str().unwrap(), true);
        assert!(result.is_ok(), "write_ssh_keys failed: {:?}", result.err());

        // Verify file was created with correct contents
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "ssh-rsa AAAAB1\nssh-rsa AAAAB2");

        // Cleanup
        drop(temp_dir);
    }

    #[test]
    fn test_write_ssh_keys_empty_response() {
        // Setup mock server with empty keys array
        let mock_response = r#"
        {
            "version": "1.0.0",
            "keys": []
        }
        "#;

        let (server_url, _server) = setup_mock_server(mock_response);

        // Setup existing file
        let existing_content = "ssh-rsa AAAABZ local_key user@host";
        let (temp_dir, file_path) = setup_temp_dir_and_file(Some(existing_content));

        // Test force mode with empty response (should clear the file)
        let result = write_ssh_keys(&server_url, file_path.to_str().unwrap(), true);
        assert!(result.is_ok(), "write_ssh_keys failed: {:?}", result.err());

        // Verify file contents (should be empty)
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "");

        // Cleanup
        drop(temp_dir);
    }

    #[test]
    fn test_write_ssh_keys_server_error() {
        // Setup mock server with error response
        let (server_url, _server) =
            setup_mock_server_with_error(500, r#"{"error": "Internal server error"}"#);

        // Setup temp file
        let (temp_dir, file_path) = setup_temp_dir_and_file(Some("existing-content"));

        // Call function
        let result = write_ssh_keys(&server_url, file_path.to_str().unwrap(), false);

        // Should return an error
        assert!(result.is_err());

        // Verify file wasn't modified
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "existing-content\n");

        // Cleanup
        drop(temp_dir);
    }

    #[test]
    fn test_write_ssh_keys_malformed_response() {
        // Setup mock server with malformed JSON
        let (server_url, _server) =
            setup_mock_server(r#"{"version": "1.0.0", "keys": [{"incomplete": true}]}"#);

        // Setup temp file
        let (temp_dir, file_path) = setup_temp_dir_and_file(Some("existing-content"));

        // Call function
        let result = write_ssh_keys(&server_url, file_path.to_str().unwrap(), false);

        // Should return an error
        assert!(result.is_err());

        // Verify file wasn't modified
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "existing-content\n");

        // Cleanup
        drop(temp_dir);
    }

    #[test]
    fn test_write_ssh_keys_local_only_keys() {
        // Setup mock server
        let mock_response = r#"
        {
            "version": "1.0.0",
            "keys": [
                {"key": "ssh-rsa AAAAB1", "user": "user1", "name": "key1", "tags": ["dev"]}
            ]
        }
        "#;

        let (server_url, _server) = setup_mock_server(mock_response);

        // Setup existing file with one key from the server and two local-only keys
        let existing_content =
            "ssh-rsa AAAAB1\nssh-rsa LOCALK1 local1@host\nssh-rsa LOCALK2 local2@host";
        let (temp_dir, file_path) = setup_temp_dir_and_file(Some(existing_content));

        // Call function with force=false (additive mode)
        let result = write_ssh_keys(&server_url, file_path.to_str().unwrap(), false);
        assert!(result.is_ok(), "write_ssh_keys failed: {:?}", result.err());

        // Verify file contents - should contain both server keys and local keys
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("ssh-rsa AAAAB1"));
        assert!(content.contains("ssh-rsa LOCALK1"));
        assert!(content.contains("ssh-rsa LOCALK2"));

        // Now try with force=true
        let result = write_ssh_keys(&server_url, file_path.to_str().unwrap(), true);
        assert!(result.is_ok(), "write_ssh_keys failed: {:?}", result.err());

        // Verify file contents - should only contain the server key
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("ssh-rsa AAAAB1"));
        assert!(!content.contains("ssh-rsa LOCALK1"));
        assert!(!content.contains("ssh-rsa LOCALK2"));

        // Cleanup
        drop(temp_dir);
    }

    #[test]
    fn test_write_ssh_keys_already_in_sync() {
        // Setup mock server
        let mock_response = r#"
        {
            "version": "1.0.0",
            "keys": [
                {"key": "ssh-rsa AAAAB1", "user": "user1", "name": "key1", "tags": ["dev"]},
                {"key": "ssh-rsa AAAAB2", "user": "user2", "name": "key2", "tags": ["prod"]}
            ]
        }
        "#;

        let (server_url, _server) = setup_mock_server(mock_response);

        // Setup existing file with the exact same keys that are on the server
        let existing_content = "ssh-rsa AAAAB1\nssh-rsa AAAAB2";
        let (temp_dir, file_path) = setup_temp_dir_and_file(Some(existing_content));

        // Call function with force=false (additive mode)
        let result = write_ssh_keys(&server_url, file_path.to_str().unwrap(), false);
        assert!(result.is_ok(), "write_ssh_keys failed: {:?}", result.err());

        // Verify file contents - should be unchanged
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("ssh-rsa AAAAB1"));
        assert!(content.contains("ssh-rsa AAAAB2"));

        // Count occurrences of each key (should only appear once)
        let count_key1 = content.matches("AAAAB1").count();
        let count_key2 = content.matches("AAAAB2").count();
        assert_eq!(count_key1, 1, "Duplicate key found: AAAAB1");
        assert_eq!(count_key2, 1, "Duplicate key found: AAAAB2");

        // Cleanup
        drop(temp_dir);
    }
}
