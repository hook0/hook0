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

/// Helper: build a command with no inherited auth env vars
fn unauthenticated_cmd() -> Command {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.env("HOME", "/tmp/hook0-test-nonexistent");
    cmd.env_remove("HOOK0_SECRET");
    cmd.env_remove("HOOK0_API_URL");
    cmd.env_remove("HOOK0_APPLICATION_ID");
    cmd.env_remove("HOOK0_PROFILE");
    cmd
}

/// Test that event command requires authentication
#[test]
fn test_event_send_requires_auth() {
    unauthenticated_cmd()
        .args(["event", "send", "test.event.created"])
        .assert()
        .failure();
}

/// Test that subscription list requires authentication
#[test]
fn test_subscription_list_requires_auth() {
    unauthenticated_cmd()
        .args(["subscription", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("login"));
}

/// Test that event-type list requires authentication
#[test]
fn test_event_type_list_requires_auth() {
    unauthenticated_cmd()
        .args(["event-type", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("login"));
}

/// Test that application list requires organization ID or authentication
#[test]
fn test_application_list_requires_auth() {
    unauthenticated_cmd()
        .args(["application", "list"])
        .assert()
        .failure();
}

/// Test that whoami requires authentication
#[test]
fn test_whoami_requires_auth() {
    unauthenticated_cmd().args(["whoami"]).assert().failure();
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
    cmd.args(["invalid-command"]).assert().failure();
}

/// Test verbose flag
#[test]
fn test_verbose_flag() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args(["-v", "--help"]).assert().success();
}

/// Test multiple verbose flags
#[test]
fn test_multiple_verbose_flags() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args(["-vvv", "--help"]).assert().success();
}

// =============================================================================
// Integration tests with real API (requires HOOK0_SECRET and HOOK0_APPLICATION_ID)
// =============================================================================

use std::sync::atomic::{AtomicU32, Ordering};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn get_test_credentials() -> (String, String, String) {
    let secret =
        std::env::var("HOOK0_SECRET").expect("HOOK0_SECRET must be set. Run: source cli/.envrc");
    let app_id = std::env::var("HOOK0_APPLICATION_ID")
        .expect("HOOK0_APPLICATION_ID must be set. Run: source cli/.envrc");
    let api_url = std::env::var("HOOK0_API_URL")
        .unwrap_or_else(|_| "https://app.hook0.com/api/v1".to_string());
    (secret, app_id, api_url)
}

/// Generate a unique profile name for test isolation (avoids keyring races)
fn unique_profile_name() -> String {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("_test-{}-{}", ts, id)
}

/// Create a Command with an isolated config dir via HOOK0_CONFIG_DIR.
/// Each test gets its own tempdir so they can run in parallel without races.
/// Clears HOOK0_SECRET/HOOK0_API_URL to prevent the parent env from leaking
/// into the child and triggering the override code path.
fn hook0_cmd(config_dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.env("HOOK0_CONFIG_DIR", config_dir);
    cmd.env_remove("HOOK0_SECRET");
    cmd.env_remove("HOOK0_API_URL");
    cmd.env_remove("HOOK0_APPLICATION_ID");
    cmd.env_remove("HOOK0_PROFILE");
    cmd
}

/// Test successful login with valid credentials (uses real keychain)
#[test]
#[ignore]
fn test_login_with_valid_credentials() {
    let (secret, app_id, api_url) = get_test_credentials();
    let config_dir = tempfile::tempdir().expect("Failed to create temp dir");

    hook0_cmd(config_dir.path())
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

    // Verify the profile is visible in config list
    hook0_cmd(config_dir.path())
        .args(["--output", "json", "config", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-profile"));

    // Cleanup keyring
    let _ = hook0_cmd(config_dir.path())
        .args(["config", "remove", "test-profile", "--yes"])
        .output();
}

/// Test whoami after login shows correct application info (uses real keychain)
#[test]
#[ignore]
fn test_whoami_after_login() {
    let (secret, app_id, api_url) = get_test_credentials();
    let config_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let profile = unique_profile_name();

    // Login
    hook0_cmd(config_dir.path())
        .args([
            "login",
            "--secret",
            &secret,
            "--application-id",
            &app_id,
            "--api-url",
            &api_url,
            "--profile-name",
            &profile,
        ])
        .assert()
        .success();

    // Whoami using the profile (reads secret from real keychain)
    hook0_cmd(config_dir.path())
        .args(["--profile", &profile, "whoami"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&app_id));

    // Cleanup keyring
    let _ = hook0_cmd(config_dir.path())
        .args(["config", "remove", &profile, "--yes"])
        .output();
}

/// Test that login stores profile configuration correctly
#[test]
#[ignore]
fn test_login_stores_profile() {
    let (secret, app_id, api_url) = get_test_credentials();
    let config_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let profile = unique_profile_name();

    hook0_cmd(config_dir.path())
        .args([
            "login",
            "--secret",
            &secret,
            "--application-id",
            &app_id,
            "--api-url",
            &api_url,
            "--profile-name",
            &profile,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Credentials saved to profile"));

    // Verify config file was created in the isolated dir
    assert!(
        config_dir.path().join("config.toml").exists(),
        "Config file should be created at {:?}",
        config_dir.path().join("config.toml")
    );

    // Verify profile contains correct details
    hook0_cmd(config_dir.path())
        .args(["--output", "json", "config", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&app_id))
        .stdout(predicate::str::contains(&api_url));

    // Cleanup keyring
    let _ = hook0_cmd(config_dir.path())
        .args(["config", "remove", &profile, "--yes"])
        .output();
}

/// Regression test: keyring must persist the secret across commands.
/// This catches the bug where keyring used MockCredential (no persistence).
/// Login stores the secret, then event-type list retrieves it from keyring
/// and authenticates with the API. If the keyring is mocked, this fails with
/// "No matching entry found in secure storage".
#[test]
#[ignore]
fn test_keyring_persists_secret_across_commands() {
    let (secret, app_id, api_url) = get_test_credentials();
    let config_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let profile = unique_profile_name();

    // Step 1: Login (stores secret in keyring)
    hook0_cmd(config_dir.path())
        .args([
            "login",
            "--secret",
            &secret,
            "--application-id",
            &app_id,
            "--api-url",
            &api_url,
            "--profile-name",
            &profile,
        ])
        .assert()
        .success();

    // Step 2: Run a command that hits the API using the keyring secret.
    // If the keyring is mocked, this fails because the secret was never stored.
    hook0_cmd(config_dir.path())
        .args(["--profile", &profile, "event-type", "list"])
        .assert()
        .success();

    // Cleanup keyring
    let _ = hook0_cmd(config_dir.path())
        .args(["config", "remove", &profile, "--yes"])
        .output();
}

/// Test JSON output format with authenticated request
#[test]
#[ignore]
fn test_json_output_authenticated() {
    let (secret, app_id, api_url) = get_test_credentials();
    let config_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let profile = unique_profile_name();

    // Login
    hook0_cmd(config_dir.path())
        .args([
            "login",
            "--secret",
            &secret,
            "--application-id",
            &app_id,
            "--api-url",
            &api_url,
            "--profile-name",
            &profile,
        ])
        .assert()
        .success();

    // Whoami with JSON output (verify JSON structure)
    hook0_cmd(config_dir.path())
        .args(["--profile", &profile, "--output", "json", "whoami"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"application_id\""))
        .stdout(predicate::str::contains(&app_id));

    // Cleanup keyring
    let _ = hook0_cmd(config_dir.path())
        .args(["config", "remove", &profile, "--yes"])
        .output();
}
