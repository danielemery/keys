use anyhow::{Context, Result};
use atty;
use colored::Colorize;
use reqwest::header::ACCEPT;
use serde::Deserialize;

use crate::utils::{ColumnConfig, pretty_print_table};

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
        let output = format_keys_for_pipe(&keys_response);
        if !output.is_empty() {
            println!("{output}");
        }
        return Ok(());
    }

    // Use the pretty print function for interactive terminal output
    pretty_print_ssh_keys(&keys_response);

    Ok(())
}

/// Helper function to format keys for non-TTY output (used for testing)
fn format_keys_for_pipe(keys_response: &KeysResponse) -> String {
    keys_response
        .keys
        .iter()
        .map(|key| format!("{} {}@{}", key.key, key.user, key.name))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Helper function to extract just the key part from an SSH key line (without comment)
fn extract_key_part(ssh_line: &str) -> String {
    let parts: Vec<&str> = ssh_line.split_whitespace().collect();
    if parts.len() >= 2 {
        // Return "ssh-rsa AAAAB..." (type + key, no comment)
        format!("{} {}", parts[0], parts[1])
    } else if parts.len() == 1 {
        parts[0].to_string()
    } else {
        ssh_line.trim().to_string()
    }
}

/// Helper function to format a server key with user@host comment
fn format_server_key(ssh_key: &SSHKey) -> String {
    format!("{} {}@{}", ssh_key.key, ssh_key.user, ssh_key.name)
}

pub fn write_ssh_keys(server_url: &str, file_path: &str, force: bool) -> Result<()> {
    // Fetch keys from the server
    let keys_response = fetch_keys_from_server(server_url)?;

    // Expand ~ to home directory if present
    let expanded_path = shellexpand::tilde(file_path);
    let path = std::path::Path::new(expanded_path.as_ref());

    // Read existing authorized_keys file if it exists
    let existing_lines = if path.exists() {
        std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read existing file: {}", path.display()))?
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    // Extract server keys (just the key part for comparison)
    let server_key_parts: Vec<String> = keys_response.keys.iter().map(|k| k.key.clone()).collect();

    // Extract existing key parts for comparison
    let existing_key_parts: Vec<String> = existing_lines
        .iter()
        .map(|line| extract_key_part(line))
        .collect();

    // Find keys that are present locally but not on the server
    let local_only_keys: Vec<&String> = existing_lines
        .iter()
        .enumerate()
        .filter(|(i, _)| !server_key_parts.contains(&existing_key_parts[*i]))
        .map(|(_, line)| line)
        .collect();
    let num_local_only = local_only_keys.len();

    // Create directory if it doesn't exist
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
    }

    let mut updated_keys_count = 0;

    // Define the file content based on the force flag
    let file_content = if force {
        // Force mode: overwrite with server keys (with user@host comments)
        keys_response
            .keys
            .iter()
            .map(format_server_key)
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        // Safe mode: merge existing keys with server keys
        let mut result_lines = Vec::new();

        // First, add existing keys, updating comments if the key matches a server key
        for existing_line in &existing_lines {
            let existing_key_part = extract_key_part(existing_line);

            // Check if this key matches any server key
            if let Some(server_key) = keys_response
                .keys
                .iter()
                .find(|k| k.key == existing_key_part)
            {
                // Update the comment part with server info
                let new_line = format_server_key(server_key);
                if new_line != *existing_line {
                    updated_keys_count += 1;
                }
                result_lines.push(new_line);
            } else {
                // Keep existing local key as-is
                result_lines.push(existing_line.clone());
            }
        }

        // Then, add new server keys that weren't already present
        for server_key in &keys_response.keys {
            if !existing_key_parts.contains(&server_key.key) {
                result_lines.push(format_server_key(server_key));
            }
        }

        result_lines.join("\n")
    };

    // Write to file
    std::fs::write(path, file_content)
        .with_context(|| format!("Failed to write to file: {}", path.display()))?;

    // Count stats
    let num_server_keys = keys_response.keys.len();
    let num_existing = existing_lines.len();
    let num_final = if force {
        num_server_keys
    } else {
        // In additive mode, we need to count unique keys
        let mut combined_key_parts = existing_key_parts.clone();
        for server_key in &keys_response.keys {
            if !combined_key_parts.contains(&server_key.key) {
                combined_key_parts.push(server_key.key.clone());
            }
        }
        combined_key_parts.len()
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
            let mut message = format!(
                "✅ Added {} new keys to {} (now {} total keys)",
                num_added,
                path.display(),
                num_final
            );
            if updated_keys_count > 0 {
                message.push_str(&format!(
                    " and updated comments for {updated_keys_count} existing keys"
                ));
            }
            println!("{message}");
        } else {
            let mut message = format!(
                "✅ Server keys are already present locally at {} ({} total keys)",
                path.display(),
                num_final
            );
            if updated_keys_count > 0 {
                message.push_str(&format!(
                    " but updated comments for {updated_keys_count} keys"
                ));
            }
            println!("{message}");
        }

        // Print warning about local-only keys if they exist
        if num_local_only > 0 {
            println!(
                "{}  {} local keys were not removed (use {} to remove)",
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
        assert_eq!(
            content,
            "ssh-rsa AAAAB1 user1@key1\nssh-rsa AAAAB2 user2@key2"
        );

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
        assert_eq!(
            content,
            "ssh-rsa AAAAB1 user1@key1\nssh-rsa AAAAB2 user2@key2"
        );

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

    #[test]
    fn test_pretty_print_ssh_keys() {
        // Create a test response with various key data
        let keys_response = KeysResponse {
            version: "1.2.3".to_string(),
            keys: vec![
                SSHKey {
                    key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC7".to_string(),
                    user: "alice".to_string(),
                    name: "work-laptop".to_string(),
                    tags: vec!["dev".to_string(), "work".to_string()],
                },
                SSHKey {
                    key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI".to_string(),
                    user: "bob".to_string(),
                    name: "home-desktop".to_string(),
                    tags: vec!["personal".to_string()],
                },
                SSHKey {
                    key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQD".to_string(),
                    user: "charlie".to_string(),
                    name: "server".to_string(),
                    tags: vec![],
                },
            ],
        };

        // This test primarily verifies the function doesn't panic and handles the data correctly
        // Since pretty_print_ssh_keys outputs to stdout, we can't easily capture and verify output
        // in this test environment, but we can verify it completes without errors
        pretty_print_ssh_keys(&keys_response);
    }

    #[test]
    fn test_pretty_print_ssh_keys_empty() {
        // Test with empty keys list
        let keys_response = KeysResponse {
            version: "1.0.0".to_string(),
            keys: vec![],
        };

        // Should handle empty keys gracefully
        pretty_print_ssh_keys(&keys_response);
    }

    #[test]
    fn test_fetch_ssh_keys_piped_output() {
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

        // Note: Testing the non-TTY path is challenging because atty::is() checks the actual stdout
        // In a real test environment, we can't easily mock this behavior
        // This test verifies the function completes successfully, but the actual output format
        // depends on whether the test is run in a TTY or not
        let result = fetch_ssh_keys(&server_url);
        assert!(result.is_ok(), "fetch_ssh_keys failed: {:?}", result.err());
    }

    #[test]
    fn test_format_keys_for_pipe() {
        // Test with multiple keys
        let keys_response = KeysResponse {
            version: "1.0.0".to_string(),
            keys: vec![
                SSHKey {
                    key: "ssh-rsa AAAAB1".to_string(),
                    user: "user1".to_string(),
                    name: "key1".to_string(),
                    tags: vec!["dev".to_string()],
                },
                SSHKey {
                    key: "ssh-ed25519 AAAAC1".to_string(),
                    user: "user2".to_string(),
                    name: "key2".to_string(),
                    tags: vec!["prod".to_string()],
                },
            ],
        };

        let output = format_keys_for_pipe(&keys_response);
        assert_eq!(
            output,
            "ssh-rsa AAAAB1 user1@key1\nssh-ed25519 AAAAC1 user2@key2"
        );
    }

    #[test]
    fn test_format_keys_for_pipe_empty() {
        // Test with no keys
        let keys_response = KeysResponse {
            version: "1.0.0".to_string(),
            keys: vec![],
        };

        let output = format_keys_for_pipe(&keys_response);
        assert_eq!(output, "");
    }

    #[test]
    fn test_format_keys_for_pipe_single_key() {
        // Test with a single key
        let keys_response = KeysResponse {
            version: "1.0.0".to_string(),
            keys: vec![SSHKey {
                key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC7".to_string(),
                user: "alice".to_string(),
                name: "laptop".to_string(),
                tags: vec!["work".to_string(), "dev".to_string()],
            }],
        };

        let output = format_keys_for_pipe(&keys_response);
        assert_eq!(
            output,
            "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC7 alice@laptop"
        );
    }

    #[test]
    fn test_write_ssh_keys_update_comments() {
        // Setup mock server with keys that have user@host info
        let mock_response = r#"
        {
            "version": "1.0.0",
            "keys": [
                {"key": "ssh-rsa AAAAB1", "user": "newuser", "name": "newname", "tags": ["dev"]},
                {"key": "ssh-rsa AAAAB2", "user": "user2", "name": "key2", "tags": ["prod"]}
            ]
        }
        "#;

        let (server_url, _server) = setup_mock_server(mock_response);

        // Setup existing file with keys that have old comments
        let existing_content = "ssh-rsa AAAAB1 olduser@oldname\nssh-rsa AAAAB3 localuser@localhost";
        let (temp_dir, file_path) = setup_temp_dir_and_file(Some(existing_content));

        // Call function with force=false (additive mode)
        let result = write_ssh_keys(&server_url, file_path.to_str().unwrap(), false);
        assert!(result.is_ok(), "write_ssh_keys failed: {:?}", result.err());

        // Verify file contents
        let content = fs::read_to_string(&file_path).unwrap();

        // Should contain the updated comment for AAAAB1
        assert!(content.contains("ssh-rsa AAAAB1 newuser@newname"));

        // Should contain the new key AAAAB2
        assert!(content.contains("ssh-rsa AAAAB2 user2@key2"));

        // Should still contain the local-only key AAAAB3
        assert!(content.contains("ssh-rsa AAAAB3 localuser@localhost"));

        // Should NOT contain the old comment for AAAAB1
        assert!(!content.contains("olduser@oldname"));

        // Count lines - should be 3 lines total
        let lines: Vec<&str> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        assert_eq!(lines.len(), 3);

        // Cleanup
        drop(temp_dir);
    }

    #[test]
    fn test_extract_key_part() {
        // Test with comment
        assert_eq!(
            extract_key_part("ssh-rsa AAAAB123 user@host"),
            "ssh-rsa AAAAB123"
        );

        // Test without comment
        assert_eq!(extract_key_part("ssh-rsa AAAAB123"), "ssh-rsa AAAAB123");

        // Test with extra whitespace
        assert_eq!(
            extract_key_part("  ssh-rsa AAAAB123   user@host  "),
            "ssh-rsa AAAAB123"
        );

        // Test with multiple comment parts
        assert_eq!(
            extract_key_part("ssh-rsa AAAAB123 user@host some extra info"),
            "ssh-rsa AAAAB123"
        );

        // Test edge case - only key type
        assert_eq!(extract_key_part("ssh-rsa"), "ssh-rsa");
    }
}
