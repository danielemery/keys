use anyhow::{Context, Result};
use atty;
use colored::Colorize;
use reqwest::header::ACCEPT;
use serde::Deserialize;

use crate::utils::{pretty_print_table, ColumnConfig};

#[derive(Debug, Deserialize)]
pub struct PGPKeysResponse {
    pub version: String,
    pub keys: Vec<PGPKey>,
}

#[derive(Debug, Deserialize)]
pub struct PGPKey {
    pub name: String,
    pub key: String,
}

/// Function to pretty print the PGP keys with formatted columns and colors
pub fn pretty_print_pgp_keys(keys_response: &PGPKeysResponse) {
    // Find the maximum width for name column for better formatting
    let max_name_len = keys_response
        .keys
        .iter()
        .map(|k| k.name.len())
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
            header: "KEY".to_string(),
            color: |s| s.red(),
            width: 50, // Key is typically long, so use a reasonable default width
        },
    ];

    // Prepare the rows
    let rows: Vec<Vec<String>> = keys_response
        .keys
        .iter()
        .map(|key| vec![key.name.clone(), key.key.clone()])
        .collect();

    // Use the generic pretty print function
    pretty_print_table(
        "PGP Keys Server Version:",
        &keys_response.version,
        columns,
        rows,
        "No PGP keys found matching the criteria.",
    );
}

pub fn fetch_pgp_keys(server_url: &str) -> Result<()> {
    let url = format!("{server_url}/pgp");

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header(ACCEPT, "application/json")
        .send()
        .context("Failed to send request to PGP keys server")?;

    let status = response.status();

    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "Server returned error code: {} - {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("Unknown")
        ));
    }

    let keys_response: PGPKeysResponse =
        response.json().context("Failed to parse JSON response")?;

    // Check if the output is being piped (not connected to a terminal)
    // Use raw/minimal output when piped to another command
    if !atty::is(atty::Stream::Stdout) {
        for key in &keys_response.keys {
            println!("{}", key.key);
        }
        return Ok(());
    }

    // Use the pretty print function for interactive terminal output
    pretty_print_pgp_keys(&keys_response);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    // Helper function to create a mock server
    fn setup_mock_server(response_body: &str) -> (String, mockito::ServerGuard) {
        let mut mock_server = mockito::Server::new();

        mock_server
            .mock("GET", "/pgp")
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
            .mock("GET", "/pgp")
            .with_status(status_code)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create();

        (mock_server.url(), mock_server)
    }

    #[test]
    fn test_fetch_pgp_keys_success() {
        // Setup mock server with PGP keys response
        let mock_response = r#"
        {
            "version": "1.0.0",
            "keys": [
                {
                    "name": "John Doe",
                    "key": "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v2\n\nmQENBFY...\n-----END PGP PUBLIC KEY BLOCK-----"
                }
            ]
        }
        "#;

        let (server_url, _server) = setup_mock_server(mock_response);

        // Call function
        let result = fetch_pgp_keys(&server_url);
        assert!(result.is_ok(), "fetch_pgp_keys failed: {:?}", result.err());
    }

    #[test]
    fn test_fetch_pgp_keys_multiple_keys() {
        // Setup mock server with multiple PGP keys
        let mock_response = r#"
        {
            "version": "2.1.0",
            "keys": [
                {
                    "name": "Alice Smith",
                    "key": "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v2\n\nmQENBFYAlice...\n-----END PGP PUBLIC KEY BLOCK-----"
                },
                {
                    "name": "Bob Johnson",
                    "key": "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v2\n\nmQENBFYBob...\n-----END PGP PUBLIC KEY BLOCK-----"
                },
                {
                    "name": "Charlie Brown",
                    "key": "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v2\n\nmQENBFYCharlie...\n-----END PGP PUBLIC KEY BLOCK-----"
                }
            ]
        }
        "#;

        let (server_url, _server) = setup_mock_server(mock_response);

        // Call function
        let result = fetch_pgp_keys(&server_url);
        assert!(result.is_ok(), "fetch_pgp_keys failed: {:?}", result.err());
    }

    #[test]
    fn test_fetch_pgp_keys_empty_response() {
        // Setup mock server with empty keys array
        let mock_response = r#"
        {
            "version": "1.0.0",
            "keys": []
        }
        "#;

        let (server_url, _server) = setup_mock_server(mock_response);

        // Call function
        let result = fetch_pgp_keys(&server_url);
        assert!(result.is_ok(), "fetch_pgp_keys failed: {:?}", result.err());
    }

    #[test]
    fn test_fetch_pgp_keys_server_error() {
        // Setup mock server with error response
        let (server_url, _server) =
            setup_mock_server_with_error(500, r#"{"error": "Internal server error"}"#);

        // Call function
        let result = fetch_pgp_keys(&server_url);

        // Should return an error
        assert!(result.is_err());
        let error_msg = result.err().unwrap().to_string();
        assert!(error_msg.contains("Server returned error code: 500"));
    }

    #[test]
    fn test_fetch_pgp_keys_malformed_response() {
        // Setup mock server with malformed JSON
        let (server_url, _server) =
            setup_mock_server(r#"{"version": "1.0.0", "keys": [{"incomplete": true}]}"#);

        // Call function
        let result = fetch_pgp_keys(&server_url);

        // Should return an error due to missing required fields
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_pgp_keys_network_error() {
        // Test with invalid URL to simulate network error
        let result = fetch_pgp_keys("http://invalid-url-that-does-not-exist.local");
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_pgp_keys_unauthorized() {
        // Setup mock server with 401 unauthorized
        let (server_url, _server) =
            setup_mock_server_with_error(401, r#"{"error": "Unauthorized"}"#);

        // Call function
        let result = fetch_pgp_keys(&server_url);

        // Should return an error
        assert!(result.is_err());
        let error_msg = result.err().unwrap().to_string();
        assert!(error_msg.contains("Server returned error code: 401"));
    }

    #[test]
    fn test_fetch_pgp_keys_not_found() {
        // Setup mock server with 404 not found
        let (server_url, _server) = setup_mock_server_with_error(404, r#"{"error": "Not found"}"#);

        // Call function
        let result = fetch_pgp_keys(&server_url);

        // Should return an error
        assert!(result.is_err());
        let error_msg = result.err().unwrap().to_string();
        assert!(error_msg.contains("Server returned error code: 404"));
    }

    #[test]
    fn test_pretty_print_pgp_keys() {
        // Create a test response with various PGP key data
        let keys_response = PGPKeysResponse {
            version: "1.0.0".to_string(),
            keys: vec![
                PGPKey {
                    name: "Alice Smith".to_string(),
                    key: "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v2\n\nmQENBFYAlice...\n-----END PGP PUBLIC KEY BLOCK-----".to_string(),
                },
                PGPKey {
                    name: "Bob Johnson".to_string(),
                    key: "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v2\n\nmQENBFYBob...\n-----END PGP PUBLIC KEY BLOCK-----".to_string(),
                },
                PGPKey {
                    name: "Charlie Brown with a very long name that tests column width".to_string(),
                    key: "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v2\n\nmQENBFYCharlie...\n-----END PGP PUBLIC KEY BLOCK-----".to_string(),
                }
            ],
        };

        // This test primarily verifies the function doesn't panic and handles the data correctly
        // Since pretty_print_pgp_keys outputs to stdout, we can't easily capture and verify output
        // in this test environment, but we can verify it completes without errors
        pretty_print_pgp_keys(&keys_response);
    }

    #[test]
    fn test_pretty_print_pgp_keys_empty() {
        // Test with empty keys list
        let keys_response = PGPKeysResponse {
            version: "1.0.0".to_string(),
            keys: vec![],
        };

        // Should handle empty keys gracefully
        pretty_print_pgp_keys(&keys_response);
    }

    #[test]
    fn test_pretty_print_pgp_keys_single_key() {
        // Test with a single key
        let keys_response = PGPKeysResponse {
            version: "2.0.0".to_string(),
            keys: vec![
                PGPKey {
                    name: "Single User".to_string(),
                    key: "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v2\n\nmQENBFYSingle...\n-----END PGP PUBLIC KEY BLOCK-----".to_string(),
                }
            ],
        };

        pretty_print_pgp_keys(&keys_response);
    }

    #[test]
    fn test_pretty_print_pgp_keys_long_names() {
        // Test with very long names to test column width calculations
        let keys_response = PGPKeysResponse {
            version: "1.0.0".to_string(),
            keys: vec![
                PGPKey {
                    name: "This is a very long name that should test the column width calculation and make sure everything aligns properly even with extremely long names".to_string(),
                    key: "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v2\n\nmQENBFYLong...\n-----END PGP PUBLIC KEY BLOCK-----".to_string(),
                },
                PGPKey {
                    name: "Short".to_string(),
                    key: "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v2\n\nmQENBFYShort...\n-----END PGP PUBLIC KEY BLOCK-----".to_string(),
                }
            ],
        };

        pretty_print_pgp_keys(&keys_response);
    }

    #[test]
    fn test_deserialize_pgp_keys_response() {
        // Test JSON deserialization with valid data
        let json_data = r#"
        {
            "version": "1.0.0",
            "keys": [
                {
                    "name": "Test User",
                    "key": "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v2\n\nmQENBFYTest...\n-----END PGP PUBLIC KEY BLOCK-----"
                }
            ]
        }
        "#;

        let result: Result<PGPKeysResponse, _> = serde_json::from_str(json_data);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.version, "1.0.0");
        assert_eq!(response.keys.len(), 1);
        assert_eq!(response.keys[0].name, "Test User");
        assert!(response.keys[0].key.contains("BEGIN PGP PUBLIC KEY BLOCK"));
    }

    #[test]
    fn test_deserialize_pgp_keys_response_multiple() {
        // Test JSON deserialization with multiple keys
        let json_data = r#"
        {
            "version": "2.0.0",
            "keys": [
                {
                    "name": "User One",
                    "key": "-----BEGIN PGP PUBLIC KEY BLOCK-----\nKey1\n-----END PGP PUBLIC KEY BLOCK-----"
                },
                {
                    "name": "User Two",
                    "key": "-----BEGIN PGP PUBLIC KEY BLOCK-----\nKey2\n-----END PGP PUBLIC KEY BLOCK-----"
                }
            ]
        }
        "#;

        let result: Result<PGPKeysResponse, _> = serde_json::from_str(json_data);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.version, "2.0.0");
        assert_eq!(response.keys.len(), 2);
        assert_eq!(response.keys[0].name, "User One");
        assert_eq!(response.keys[1].name, "User Two");
        assert!(response.keys[0].key.contains("Key1"));
        assert!(response.keys[1].key.contains("Key2"));
    }

    #[test]
    fn test_deserialize_pgp_keys_response_empty() {
        // Test JSON deserialization with empty keys array
        let json_data = r#"
        {
            "version": "1.0.0",
            "keys": []
        }
        "#;

        let result: Result<PGPKeysResponse, _> = serde_json::from_str(json_data);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.version, "1.0.0");
        assert_eq!(response.keys.len(), 0);
    }

    #[test]
    fn test_deserialize_pgp_keys_response_invalid() {
        // Test JSON deserialization with missing required fields
        let json_data = r#"
        {
            "version": "1.0.0",
            "keys": [
                {
                    "name": "Incomplete User"
                }
            ]
        }
        "#;

        let result: Result<PGPKeysResponse, _> = serde_json::from_str(json_data);
        assert!(result.is_err()); // Should fail due to missing 'key' field
    }

    #[test]
    fn test_deserialize_pgp_keys_response_missing_version() {
        // Test JSON deserialization with missing version field
        let json_data = r#"
        {
            "keys": [
                {
                    "name": "Test User",
                    "key": "-----BEGIN PGP PUBLIC KEY BLOCK-----\nTest\n-----END PGP PUBLIC KEY BLOCK-----"
                }
            ]
        }
        "#;

        let result: Result<PGPKeysResponse, _> = serde_json::from_str(json_data);
        assert!(result.is_err()); // Should fail due to missing 'version' field
    }

    #[test]
    fn test_deserialize_pgp_keys_response_invalid_json() {
        // Test with completely invalid JSON
        let json_data = r#"{"invalid": json structure"#;

        let result: Result<PGPKeysResponse, _> = serde_json::from_str(json_data);
        assert!(result.is_err()); // Should fail due to invalid JSON syntax
    }

    #[test]
    fn test_pgp_key_with_special_characters() {
        // Test with names containing special characters
        let keys_response = PGPKeysResponse {
            version: "1.0.0".to_string(),
            keys: vec![
                PGPKey {
                    name: "François Müller <francois@example.com>".to_string(),
                    key: "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v2\n\nmQENBFYSpecial...\n-----END PGP PUBLIC KEY BLOCK-----".to_string(),
                },
                PGPKey {
                    name: "José García (Company) [Developer]".to_string(),
                    key: "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v2\n\nmQENBFYJose...\n-----END PGP PUBLIC KEY BLOCK-----".to_string(),
                }
            ],
        };

        // Should handle special characters in names gracefully
        pretty_print_pgp_keys(&keys_response);
    }

    #[test]
    fn test_pgp_key_with_different_key_formats() {
        // Test with different PGP key formats (RSA, DSA, etc.)
        let keys_response = PGPKeysResponse {
            version: "1.0.0".to_string(),
            keys: vec![
                PGPKey {
                    name: "RSA User".to_string(),
                    key: "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v2\n\nmQENBFYRSA... (RSA)\n-----END PGP PUBLIC KEY BLOCK-----".to_string(),
                },
                PGPKey {
                    name: "DSA User".to_string(),
                    key: "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v1\n\nmQGiBFYDSA... (DSA)\n-----END PGP PUBLIC KEY BLOCK-----".to_string(),
                },
                PGPKey {
                    name: "Ed25519 User".to_string(),
                    key: "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: GnuPG v2\n\nmDMEZEd25519... (Ed25519)\n-----END PGP PUBLIC KEY BLOCK-----".to_string(),
                }
            ],
        };

        pretty_print_pgp_keys(&keys_response);
    }
}
