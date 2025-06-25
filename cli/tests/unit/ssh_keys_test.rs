use mockito;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

// Import the function we want to test
use keys::commands::ssh_keys::write_ssh_keys;

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
