#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use mockito::Server;
    use predicates::prelude::*;
    use std::fs;
    use tempfile::TempDir;

    fn get_cmd() -> Command {
        #[allow(deprecated)]
        Command::cargo_bin("keys").unwrap()
    }

    // ==================== Help and Version Tests ====================

    #[test]
    fn test_cli_help() {
        get_cmd()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("CLI client for the keys server"));
    }

    #[test]
    fn test_cli_version() {
        get_cmd()
            .arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("keys"));
    }

    #[test]
    fn test_cli_no_subcommand_shows_error() {
        get_cmd()
            .assert()
            .failure()
            .stderr(predicate::str::contains("Usage"));
    }

    // ==================== SSH Subcommand Tests ====================

    #[test]
    fn test_ssh_help() {
        get_cmd()
            .args(["ssh", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("SSH keys"));
    }

    #[test]
    fn test_ssh_fetch_success() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/keys")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "version": "1.0.0",
                    "keys": [
                        {
                            "name": "laptop",
                            "user": "alice",
                            "key": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIExample",
                            "tags": []
                        }
                    ]
                }"#,
            )
            .create();

        get_cmd()
            .args(["--server", &server.url(), "ssh"])
            .assert()
            .success();

        mock.assert();
    }

    #[test]
    fn test_ssh_fetch_server_error() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/keys")
            .with_status(500)
            .with_body(r#"{"error": "Internal server error"}"#)
            .create();

        get_cmd()
            .args(["--server", &server.url(), "ssh"])
            .assert()
            .failure();

        mock.assert();
    }

    #[test]
    fn test_ssh_write_creates_file() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/keys")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "version": "1.0.0",
                    "keys": [
                        {
                            "name": "laptop",
                            "user": "alice",
                            "key": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIExample",
                            "tags": []
                        }
                    ]
                }"#,
            )
            .create();

        let temp_dir = TempDir::new().unwrap();
        let auth_keys_path = temp_dir.path().join("authorized_keys");

        get_cmd()
            .args([
                "--server",
                &server.url(),
                "ssh",
                "--write",
                auth_keys_path.to_str().unwrap(),
            ])
            .assert()
            .success();

        mock.assert();

        // Verify file was created with correct content
        assert!(auth_keys_path.exists());
        let content = fs::read_to_string(&auth_keys_path).unwrap();
        assert!(content.contains("ssh-ed25519"));
        assert!(content.contains("alice@laptop"));
    }

    #[test]
    fn test_ssh_write_with_force_flag() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/keys")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "version": "1.0.0",
                    "keys": [
                        {
                            "name": "new-laptop",
                            "user": "bob",
                            "key": "ssh-rsa AAAABNewKey",
                            "tags": []
                        }
                    ]
                }"#,
            )
            .create();

        let temp_dir = TempDir::new().unwrap();
        let auth_keys_path = temp_dir.path().join("authorized_keys");

        // Create existing file with different key
        fs::write(&auth_keys_path, "ssh-ed25519 OldKey old@machine\n").unwrap();

        get_cmd()
            .args([
                "--server",
                &server.url(),
                "ssh",
                "--write",
                auth_keys_path.to_str().unwrap(),
                "--force",
            ])
            .assert()
            .success();

        mock.assert();

        // Verify file was overwritten (old key removed)
        let content = fs::read_to_string(&auth_keys_path).unwrap();
        assert!(!content.contains("OldKey"));
        assert!(content.contains("AAAABNewKey"));
    }

    #[test]
    fn test_ssh_write_additive_mode() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/keys")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "version": "1.0.0",
                    "keys": [
                        {
                            "name": "new-laptop",
                            "user": "bob",
                            "key": "ssh-rsa AAAABNewKey",
                            "tags": []
                        }
                    ]
                }"#,
            )
            .create();

        let temp_dir = TempDir::new().unwrap();
        let auth_keys_path = temp_dir.path().join("authorized_keys");

        // Create existing file with different key
        fs::write(&auth_keys_path, "ssh-ed25519 OldKey old@machine\n").unwrap();

        // Without --force, should preserve existing keys
        get_cmd()
            .args([
                "--server",
                &server.url(),
                "ssh",
                "--write",
                auth_keys_path.to_str().unwrap(),
            ])
            .assert()
            .success();

        mock.assert();

        // Verify both keys are present
        let content = fs::read_to_string(&auth_keys_path).unwrap();
        assert!(content.contains("OldKey"));
        assert!(content.contains("AAAABNewKey"));
    }

    // ==================== PGP Subcommand Tests ====================

    #[test]
    fn test_pgp_help() {
        get_cmd()
            .args(["pgp", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("PGP keys"));
    }

    #[test]
    fn test_pgp_fetch_success() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/pgp")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "version": "1.0.0",
                    "keys": [
                        {
                            "name": "alice",
                            "key": "-----BEGIN PGP PUBLIC KEY BLOCK-----\nmQINB...\n-----END PGP PUBLIC KEY BLOCK-----"
                        }
                    ]
                }"#,
            )
            .create();

        get_cmd()
            .args(["--server", &server.url(), "pgp"])
            .assert()
            .success();

        mock.assert();
    }

    #[test]
    fn test_pgp_fetch_server_error() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/pgp")
            .with_status(500)
            .with_body(r#"{"error": "Internal server error"}"#)
            .create();

        get_cmd()
            .args(["--server", &server.url(), "pgp"])
            .assert()
            .failure();

        mock.assert();
    }

    // ==================== Known Hosts Subcommand Tests ====================

    #[test]
    fn test_known_hosts_help() {
        get_cmd()
            .args(["known-hosts", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("known hosts"));
    }

    #[test]
    fn test_known_hosts_fetch_success() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/known_hosts")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "version": "1.0.0",
                    "knownHosts": [
                        {
                            "name": "GitHub",
                            "hosts": ["github.com"],
                            "keys": [
                                {
                                    "type": "ssh-ed25519",
                                    "key": "AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl"
                                }
                            ]
                        }
                    ]
                }"#,
            )
            .create();

        get_cmd()
            .args(["--server", &server.url(), "known-hosts"])
            .assert()
            .success();

        mock.assert();
    }

    #[test]
    fn test_known_hosts_fetch_server_error() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/known_hosts")
            .with_status(500)
            .with_body(r#"{"error": "Internal server error"}"#)
            .create();

        get_cmd()
            .args(["--server", &server.url(), "known-hosts"])
            .assert()
            .failure();

        mock.assert();
    }

    #[test]
    fn test_known_hosts_write_creates_file() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/known_hosts")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "version": "1.0.0",
                    "knownHosts": [
                        {
                            "name": "GitHub",
                            "hosts": ["github.com"],
                            "keys": [
                                {
                                    "type": "ssh-ed25519",
                                    "key": "AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl"
                                }
                            ]
                        }
                    ]
                }"#,
            )
            .create();

        let temp_dir = TempDir::new().unwrap();
        let known_hosts_path = temp_dir.path().join("known_hosts");

        get_cmd()
            .args([
                "--server",
                &server.url(),
                "known-hosts",
                "--write",
                known_hosts_path.to_str().unwrap(),
            ])
            .assert()
            .success();

        mock.assert();

        // Verify file was created with correct content
        assert!(known_hosts_path.exists());
        let content = fs::read_to_string(&known_hosts_path).unwrap();
        assert!(content.contains("github.com"));
        assert!(content.contains("ssh-ed25519"));
    }

    // ==================== Init Subcommand Tests ====================

    #[test]
    fn test_init_help() {
        get_cmd()
            .args(["init", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("config"));
    }

    #[test]
    fn test_init_creates_config() {
        // Note: This test creates a real config file at the default location
        // We can't easily mock ProjectDirs, so we just verify it doesn't crash
        // and produces expected output
        get_cmd()
            .arg("init")
            .assert()
            .success()
            .stdout(predicate::str::contains("config"));
    }

    // ==================== Config File Tests ====================

    #[test]
    fn test_custom_config_file() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/keys")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "version": "1.0.0",
                    "keys": []
                }"#,
            )
            .create();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create config file pointing to mock server
        fs::write(&config_path, format!("server_url = \"{}\"", server.url())).unwrap();

        get_cmd()
            .args(["--config", config_path.to_str().unwrap(), "ssh"])
            .assert()
            .success();

        mock.assert();
    }

    #[test]
    fn test_server_flag_overrides_config() {
        let mut config_server = Server::new();
        let mut cli_server = Server::new();

        // Config server should NOT be called
        let _config_mock = config_server
            .mock("GET", "/keys")
            .with_status(200)
            .with_body(r#"{"version": "1.0.0", "keys": []}"#)
            .expect(0)
            .create();

        // CLI server SHOULD be called
        let cli_mock = cli_server
            .mock("GET", "/keys")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"version": "1.0.0", "keys": []}"#)
            .create();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create config file pointing to config_server
        fs::write(
            &config_path,
            format!("server_url = \"{}\"", config_server.url()),
        )
        .unwrap();

        // But use --server flag to override
        get_cmd()
            .args([
                "--config",
                config_path.to_str().unwrap(),
                "--server",
                &cli_server.url(),
                "ssh",
            ])
            .assert()
            .success();

        cli_mock.assert();
    }

    #[test]
    fn test_nonexistent_config_file_warning() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/keys")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"version": "1.0.0", "keys": []}"#)
            .create();

        // Use a non-existent config path, but provide server via CLI
        get_cmd()
            .args([
                "--config",
                "/nonexistent/config.toml",
                "--server",
                &server.url(),
                "ssh",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Warning"));

        mock.assert();
    }

    // ==================== Error Handling Tests ====================

    #[test]
    fn test_connection_refused() {
        // Use a port that's unlikely to have anything listening
        get_cmd()
            .args(["--server", "http://127.0.0.1:59999", "ssh"])
            .assert()
            .failure();
    }

    #[test]
    fn test_invalid_server_url() {
        get_cmd()
            .args(["--server", "not-a-valid-url", "ssh"])
            .assert()
            .failure();
    }

    #[test]
    fn test_malformed_json_response() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/keys")
            .with_status(200)
            .with_body("not valid json")
            .create();

        get_cmd()
            .args(["--server", &server.url(), "ssh"])
            .assert()
            .failure();

        mock.assert();
    }
}
