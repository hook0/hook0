use assert_cmd::Command;
use predicates::prelude::*;

/// Test that the CLI shows help when invoked without arguments
#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("CLI for Hook0 webhooks platform"));
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

/// Test login command requires authentication
#[test]
fn test_login_invalid_secret() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.args(["login", "--secret", "not-a-valid-uuid"])
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
        .failure()
        .stderr(predicate::str::contains("login"));
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

/// Test that application list requires authentication
#[test]
fn test_application_list_requires_auth() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.env("HOME", "/tmp/hook0-test-nonexistent")
        .args(["application", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("login"));
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

/// Test that listen command requires target URL
#[test]
fn test_listen_requires_target() {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.env("HOME", "/tmp/hook0-test-nonexistent")
        .args(["listen"])
        .assert()
        .failure();
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
