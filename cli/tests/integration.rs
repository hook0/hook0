use assert_cmd::Command;
use predicates::prelude::*;

/// Test that the CLI shows help when invoked without arguments
#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Webhooks as a Service"));
}

/// Test that version flag works
#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("hook0"));
}

/// Test login command requires application-id and validates secret format
#[test]
fn test_login_invalid_secret() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args([
        "login",
        "--application-id",
        "550e8400-e29b-41d4-a716-446655440000",
        "--secret",
        "not-a-valid-uuid",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("Invalid secret format"));
}

/// Test completion command generates shell completions
#[test]
fn test_completion_bash() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args(["completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_hook0"));
}

/// Test completion command generates zsh completions
#[test]
fn test_completion_zsh() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args(["completion", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_hook0"));
}

/// Test completion command generates fish completions
#[test]
fn test_completion_fish() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args(["completion", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hook0"));
}

/// Test config path command shows config file location
#[test]
fn test_config_path() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args(["config", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hook0"));
}

/// Test config list with no configuration
#[test]
fn test_config_list_empty() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    // Use a temp home directory to avoid using real config
    cmd.env("HOME", "/tmp/hook0-test-nonexistent")
        .args(["config", "list"])
        .assert()
        .success();
}

/// Test that event command requires authentication
#[test]
fn test_event_send_requires_auth() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.env("HOME", "/tmp/hook0-test-nonexistent")
        .args(["event", "send", "test.event.created"])
        .assert()
        .failure();
}

/// Test that subscription list requires authentication
#[test]
fn test_subscription_list_requires_auth() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.env("HOME", "/tmp/hook0-test-nonexistent")
        .args(["subscription", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("login"));
}

/// Test that event-type list requires authentication
#[test]
fn test_event_type_list_requires_auth() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.env("HOME", "/tmp/hook0-test-nonexistent")
        .args(["event-type", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("login"));
}

/// Test that application list requires organization ID or authentication
#[test]
fn test_application_list_requires_auth() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.env("HOME", "/tmp/hook0-test-nonexistent")
        .args(["application", "list"])
        .assert()
        .failure();
}

/// Test that whoami requires authentication
#[test]
fn test_whoami_requires_auth() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.env("HOME", "/tmp/hook0-test-nonexistent")
        .args(["whoami"])
        .assert()
        .failure();
}

/// Test output format flag works with JSON
#[test]
fn test_output_format_json() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.env("HOME", "/tmp/hook0-test-nonexistent")
        .args(["--output", "json", "config", "list"])
        .assert()
        .success();
}

/// Test output format flag works with table
#[test]
fn test_output_format_table() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.env("HOME", "/tmp/hook0-test-nonexistent")
        .args(["--output", "table", "config", "list"])
        .assert()
        .success();
}

/// Test help for subcommands
#[test]
fn test_event_help() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args(["event", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("send"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"));
}

/// Test help for subscription subcommands
#[test]
fn test_subscription_help() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args(["subscription", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("enable"))
        .stdout(predicate::str::contains("disable"));
}

/// Test help for event-type subcommands
#[test]
fn test_event_type_help() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args(["event-type", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("delete"));
}

/// Test help for application subcommands
#[test]
fn test_application_help() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args(["application", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("switch"));
}

/// Test help for config subcommands
#[test]
fn test_config_help() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("set-default"))
        .stdout(predicate::str::contains("remove"))
        .stdout(predicate::str::contains("path"));
}

/// Test invalid subcommand
#[test]
fn test_invalid_subcommand() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args(["invalid-command"])
        .assert()
        .failure();
}

/// Test verbose flag
#[test]
fn test_verbose_flag() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args(["-v", "--help"])
        .assert()
        .success();
}

/// Test multiple verbose flags
#[test]
fn test_multiple_verbose_flags() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args(["-vvv", "--help"])
        .assert()
        .success();
}

// =============================================================================
// Integration tests with real API (requires HOOK0_SECRET and HOOK0_APPLICATION_ID)
// =============================================================================

fn get_test_credentials() -> (String, String, String) {
    let secret = std::env::var("HOOK0_SECRET")
        .expect("HOOK0_SECRET must be set. Run: source cli/.envrc");
    let app_id = std::env::var("HOOK0_APPLICATION_ID")
        .expect("HOOK0_APPLICATION_ID must be set. Run: source cli/.envrc");
    let api_url = std::env::var("HOOK0_API_URL")
        .unwrap_or_else(|_| "https://app.hook0.com/api/v1".to_string());
    (secret, app_id, api_url)
}

/// Test successful login with valid credentials from environment
#[test]
fn test_login_with_valid_credentials() {
    let (secret, app_id, api_url) = get_test_credentials();
    let temp_home = tempfile::tempdir().expect("Failed to create temp dir");

    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.env("HOME", temp_home.path())
        .args([
            "login",
            "--secret",
            &secret,
            "--application-id",
            &app_id,
            "--api-url",
            &api_url,
            "--profile-name",
            "test-profile",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Authenticated successfully"));
}

/// Test whoami after login shows correct application info
#[test]
fn test_whoami_after_login() {
    let (secret, app_id, api_url) = get_test_credentials();
    let temp_home = tempfile::tempdir().expect("Failed to create temp dir");

    // First login
    let mut login_cmd = Command::cargo_bin("hook0").expect("binary should exist");
    login_cmd
        .env("HOME", temp_home.path())
        .args([
            "login",
            "--secret",
            &secret,
            "--application-id",
            &app_id,
            "--api-url",
            &api_url,
        ])
        .assert()
        .success();

    // Then whoami (verifies profile was saved, keyring access may vary by platform)
    let mut whoami_cmd = Command::cargo_bin("hook0").expect("binary should exist");
    whoami_cmd
        .env("HOME", temp_home.path())
        .args(["whoami"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&app_id));
}

/// Test that login stores profile configuration correctly
#[test]
fn test_login_stores_profile() {
    let (secret, app_id, api_url) = get_test_credentials();
    let temp_home = tempfile::tempdir().expect("Failed to create temp dir");

    // Login should create profile
    // Set appropriate env vars based on OS for config directory resolution
    let mut login_cmd = Command::cargo_bin("hook0").expect("binary should exist");
    login_cmd.env("HOME", temp_home.path());

    // On Windows, dirs crate uses APPDATA, not HOME
    #[cfg(target_os = "windows")]
    login_cmd.env("APPDATA", temp_home.path());

    login_cmd
        .args([
            "login",
            "--secret",
            &secret,
            "--application-id",
            &app_id,
            "--api-url",
            &api_url,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Credentials saved to profile"));

    // Verify config file was created (path varies by OS)
    // macOS: ~/Library/Application Support/hook0/config.toml
    // Linux: ~/.config/hook0/config.toml
    // Windows: %APPDATA%/hook0/config.toml
    let config_path = if cfg!(target_os = "macos") {
        temp_home.path().join("Library/Application Support/hook0/config.toml")
    } else if cfg!(target_os = "windows") {
        temp_home.path().join("hook0/config.toml")
    } else {
        temp_home.path().join(".config/hook0/config.toml")
    };
    assert!(config_path.exists(), "Config file should be created at {:?}", config_path);
}

/// Test JSON output format with authenticated request
#[test]
fn test_json_output_authenticated() {
    let (secret, app_id, api_url) = get_test_credentials();
    let temp_home = tempfile::tempdir().expect("Failed to create temp dir");

    // First login
    let mut login_cmd = Command::cargo_bin("hook0").expect("binary should exist");
    login_cmd
        .env("HOME", temp_home.path())
        .args([
            "login",
            "--secret",
            &secret,
            "--application-id",
            &app_id,
            "--api-url",
            &api_url,
        ])
        .assert()
        .success();

    // Then whoami with JSON output (verify JSON structure, keyring auth may vary by platform)
    let mut whoami_cmd = Command::cargo_bin("hook0").expect("binary should exist");
    whoami_cmd
        .env("HOME", temp_home.path())
        .args(["--output", "json", "whoami"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"application_id\""))
        .stdout(predicate::str::contains(&app_id));
}
