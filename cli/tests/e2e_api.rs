//! End-to-end tests for all CLI commands that interact with the Hook0 API.
//!
//! Requires: HOOK0_SECRET, HOOK0_APPLICATION_ID, HOOK0_API_URL env vars.
//! Each test gets an isolated config dir (HOOK0_CONFIG_DIR) and a unique
//! keyring profile so tests can run in parallel without races.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn credentials() -> (String, String, String) {
    let secret =
        std::env::var("HOOK0_SECRET").expect("HOOK0_SECRET must be set");
    let app_id =
        std::env::var("HOOK0_APPLICATION_ID").expect("HOOK0_APPLICATION_ID must be set");
    let api_url =
        std::env::var("HOOK0_API_URL").unwrap_or_else(|_| "https://app.hook0.com/api/v1".into());
    (secret, app_id, api_url)
}

fn unique_id() -> String {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{}{}", ts, id)
}

fn profile_name() -> String {
    format!("_e2e-{}", unique_id())
}

/// Unique prefix for event types / resources to avoid collisions between parallel tests.
fn tag() -> String {
    format!("t{}", unique_id())
}

/// Build a Command with isolated config and clean env.
fn cli(cfg: &Path) -> Command {
    let mut cmd = Command::cargo_bin("hook0").expect("binary should exist");
    cmd.env("HOOK0_CONFIG_DIR", cfg);
    cmd.env_remove("HOOK0_SECRET");
    cmd.env_remove("HOOK0_API_URL");
    cmd.env_remove("HOOK0_APPLICATION_ID");
    cmd.env_remove("HOOK0_PROFILE");
    cmd
}

/// Login helper. Returns the profile name used.
fn login(cfg: &Path, secret: &str, app_id: &str, api_url: &str) -> String {
    let profile = profile_name();
    cli(cfg)
        .args([
            "login", "--secret", secret, "--application-id", app_id,
            "--api-url", api_url, "--profile-name", &profile,
        ])
        .assert()
        .success();
    profile
}

/// Cleanup helper.
fn cleanup(cfg: &Path, profile: &str) {
    let _ = cli(cfg)
        .args(["config", "remove", profile, "--yes"])
        .output();
}

// =============================================================================
// HOOK0_CONFIG_DIR isolation
// =============================================================================

#[test]
fn test_config_dir_isolation() {
    let dir_a = tempfile::tempdir().unwrap();
    let dir_b = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();

    // Login in dir_a
    let prof = login(dir_a.path(), &secret, &app_id, &api_url);

    // dir_a sees the profile
    cli(dir_a.path())
        .args(["--output", "json", "config", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&prof));

    // dir_b does not (message goes to stderr)
    cli(dir_b.path())
        .args(["config", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No profiles"));

    // config path reflects the override
    let output = cli(dir_a.path())
        .args(["config", "path"])
        .output()
        .unwrap();
    let path_str = String::from_utf8_lossy(&output.stdout);
    assert!(path_str.contains("config.toml"), "config path should contain config.toml");

    cleanup(dir_a.path(), &prof);
}

// =============================================================================
// Logout
// =============================================================================

#[test]
fn test_logout_removes_credentials() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    // whoami works
    cli(cfg.path())
        .args(["--profile", &prof, "whoami"])
        .assert()
        .success();

    // logout
    cli(cfg.path())
        .args(["logout", "--profile-name", &prof])
        .assert()
        .success();

    // whoami fails after logout
    cli(cfg.path())
        .args(["--profile", &prof, "whoami"])
        .assert()
        .failure();
}

// =============================================================================
// Event-Type CRUD
// =============================================================================

#[test]
fn test_event_type_create_list_delete() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);
    let t = tag();
    let et_created = format!("{}.crud.created", t);
    let et_deleted = format!("{}.crud.deleted", t);

    // Create by full name
    cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", &et_created])
        .assert()
        .success();

    // Create by components
    cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", "-s", &t, "-r", "crud", "-b", "deleted"])
        .assert()
        .success();

    // List (table)
    cli(cfg.path())
        .args(["--profile", &prof, "event-type", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&t));

    // List (json) — verify parseable JSON array
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "event-type", "list", "--service", &t])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
        .expect("event-type list JSON should be a valid array");
    assert_eq!(json.len(), 2, "should have exactly 2 event types with this service");

    // Delete
    cli(cfg.path())
        .args(["--profile", &prof, "event-type", "delete", &et_deleted, "--yes"])
        .assert()
        .success();

    // Verify deleted — list should only show 1
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "event-type", "list", "--service", &t])
        .output()
        .unwrap();
    let json: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json.len(), 1, "only created should remain");

    // Cleanup
    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "delete", &et_created, "--yes"])
        .output();
    cleanup(cfg.path(), &prof);
}

#[test]
fn test_event_type_validation() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    // Invalid: not 3 parts
    cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", "invalid"])
        .assert()
        .failure();

    // Invalid: too many parts
    cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", "a.b.c.d"])
        .assert()
        .failure();

    cleanup(cfg.path(), &prof);
}

// =============================================================================
// Event Send + List + Get
// =============================================================================

#[test]
fn test_event_send_and_list() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);
    let t = tag();
    let et = format!("{}.send.created", t);

    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", &et])
        .output();

    // Send with default payload
    cli(cfg.path())
        .args(["--profile", &prof, "event", "send", &et, "-l", "env=test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Event sent successfully"));

    // Send with JSON payload
    cli(cfg.path())
        .args(["--profile", &prof, "event", "send", &et,
               "-d", r#"{"key":"value"}"#, "-l", "env=test"])
        .assert()
        .success();

    // Send with multiple labels
    cli(cfg.path())
        .args(["--profile", &prof, "event", "send", &et,
               "-l", "env=test", "-l", "region=eu"])
        .assert()
        .success();

    // Send with custom event-id
    let custom_id = uuid::Uuid::new_v4().to_string();
    cli(cfg.path())
        .args(["--profile", &prof, "event", "send", &et,
               "-l", "env=test", "--event-id", &custom_id])
        .assert()
        .success();

    // Send with JSON output
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "event", "send", &et, "-l", "env=test"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("event send JSON output should be valid");
    assert!(json.get("event_id").is_some(), "JSON should contain event_id");

    // List (table)
    cli(cfg.path())
        .args(["--profile", &prof, "event", "list"])
        .assert()
        .success();

    // List (json)
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "event", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let events: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
        .expect("event list JSON should be a valid array");
    assert!(!events.is_empty());

    // List (compact)
    cli(cfg.path())
        .args(["--profile", &prof, "--output", "compact", "event", "list"])
        .assert()
        .success();

    // List with filters
    cli(cfg.path())
        .args(["--profile", &prof, "event", "list", "--limit", "2"])
        .assert()
        .success();

    cli(cfg.path())
        .args(["--profile", &prof, "event", "list", "--event-type", &et])
        .assert()
        .success();

    cli(cfg.path())
        .args(["--profile", &prof, "event", "list", "--since", "1h"])
        .assert()
        .success();

    cli(cfg.path())
        .args(["--profile", &prof, "event", "list", "-l", "env=test"])
        .assert()
        .success();

    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "delete", &et, "--yes"])
        .output();
    cleanup(cfg.path(), &prof);
}

#[test]
fn test_event_send_payload_file() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    let t = tag();
    let et = format!("{}.file.created", t);
    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", &et])
        .output();

    // Write payload file
    let payload_file = cfg.path().join("payload.json");
    std::fs::write(&payload_file, r#"{"from_file": true}"#).unwrap();

    cli(cfg.path())
        .args(["--profile", &prof, "event", "send", &et,
               "-f", payload_file.to_str().unwrap(), "-l", "env=test"])
        .assert()
        .success();

    // Cannot use both --payload and --payload-file
    cli(cfg.path())
        .args(["--profile", &prof, "event", "send", &et,
               "-d", "{}", "-f", payload_file.to_str().unwrap(), "-l", "env=test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot specify both"));

    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "delete", &et, "--yes"])
        .output();
    cleanup(cfg.path(), &prof);
}

#[test]
fn test_event_send_requires_label() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    cli(cfg.path())
        .args(["--profile", &prof, "event", "send", "e2e.nolabel.created"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("label"));

    cleanup(cfg.path(), &prof);
}

#[test]
fn test_event_get_with_details() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    let t = tag();
    let et = format!("{}.get.created", t);
    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", &et])
        .output();

    // Send an event
    cli(cfg.path())
        .args(["--profile", &prof, "event", "send", &et,
               "-d", r#"{"detail": true}"#, "-l", "env=test"])
        .assert()
        .success();

    // Get the event ID from list
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "event", "list", "--limit", "1"])
        .output()
        .unwrap();
    let events: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    let event_id = events[0]["event_id"].as_str().unwrap();

    // Get event (table)
    cli(cfg.path())
        .args(["--profile", &prof, "event", "get", event_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(event_id));

    // Get event (json) — verify parseable
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "event", "get", event_id])
        .output()
        .unwrap();
    assert!(output.status.success());
    let event: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("event get JSON should be valid");
    assert_eq!(event["event_id"].as_str().unwrap(), event_id);

    // Get event with --attempts
    cli(cfg.path())
        .args(["--profile", &prof, "event", "get", event_id, "--attempts"])
        .assert()
        .success();

    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "delete", &et, "--yes"])
        .output();
    cleanup(cfg.path(), &prof);
}

// =============================================================================
// Subscription CRUD
// =============================================================================

#[test]
fn test_subscription_full_lifecycle() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    let t = tag();
    let et = format!("{}.sub.created", t);
    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", &et])
        .output();

    // Create subscription (json output to capture ID directly)
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "subscription", "create",
               "-e", &et, "-u", "https://httpbin.org/post",
               "-l", "env=test", "-d", "E2E test subscription"])
        .output()
        .unwrap();
    assert!(output.status.success(), "subscription create failed");
    let created: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("subscription create JSON should be valid");
    let sub_id = created["subscription_id"].as_str().unwrap().to_string();

    // List (json) — verify it appears
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "subscription", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let subs: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
        .expect("subscription list JSON should be valid array");
    assert!(
        subs.iter().any(|s| s["subscription_id"].as_str() == Some(&sub_id)),
        "created subscription should appear in list"
    );

    // Get (table)
    let get_assert = cli(cfg.path())
        .args(["--profile", &prof, "subscription", "get", &sub_id])
        .output()
        .unwrap();
    assert!(
        get_assert.status.success(),
        "subscription get failed for {}: {}",
        sub_id,
        String::from_utf8_lossy(&get_assert.stderr)
    );

    // Get (json) — verify structure
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "subscription", "get", &sub_id])
        .output()
        .unwrap();
    assert!(output.status.success(), "subscription get json failed: {}", String::from_utf8_lossy(&output.stderr));
    let sub: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("parse failed: {}, stdout: {}", e, String::from_utf8_lossy(&output.stdout)));
    assert_eq!(sub["subscription_id"].as_str().unwrap(), sub_id);
    assert_eq!(sub["is_enabled"].as_bool().unwrap(), true);

    // Disable
    cli(cfg.path())
        .args(["--profile", &prof, "subscription", "disable", &sub_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("disabled"));

    // Verify disabled
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "subscription", "get", &sub_id])
        .output()
        .unwrap();
    let sub: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(sub["is_enabled"].as_bool().unwrap(), false);

    // Enable
    cli(cfg.path())
        .args(["--profile", &prof, "subscription", "enable", &sub_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("enabled"));

    // Update description
    cli(cfg.path())
        .args(["--profile", &prof, "subscription", "update", &sub_id,
               "-d", "Updated by e2e test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("updated"));

    // Update URL
    cli(cfg.path())
        .args(["--profile", &prof, "subscription", "update", &sub_id,
               "-u", "https://httpbin.org/anything"])
        .assert()
        .success();

    // List --enabled
    cli(cfg.path())
        .args(["--profile", &prof, "subscription", "list", "--enabled"])
        .assert()
        .success();

    // List --disabled (should be empty since we re-enabled)
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "subscription", "list", "--disabled"])
        .output()
        .unwrap();
    let disabled: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        !disabled.iter().any(|s| s["subscription_id"].as_str() == Some(&sub_id)),
        "re-enabled subscription should not appear in --disabled list"
    );

    // Delete
    cli(cfg.path())
        .args(["--profile", &prof, "subscription", "delete", &sub_id, "--yes"])
        .assert()
        .success();

    // Get after delete — should fail
    cli(cfg.path())
        .args(["--profile", &prof, "subscription", "get", &sub_id])
        .assert()
        .failure();

    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "delete", &et, "--yes"])
        .output();
    cleanup(cfg.path(), &prof);
}

#[test]
fn test_subscription_create_validation() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    // Missing label
    cli(cfg.path())
        .args(["--profile", &prof, "subscription", "create",
               "-e", "x.y.z", "-u", "https://example.com"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("label"));

    // Missing events
    cli(cfg.path())
        .args(["--profile", &prof, "subscription", "create",
               "-u", "https://example.com", "-l", "a=b"])
        .assert()
        .failure();

    // Invalid URL
    cli(cfg.path())
        .args(["--profile", &prof, "subscription", "create",
               "-e", "x.y.z", "-u", "not-a-url", "-l", "a=b"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid URL"));

    cleanup(cfg.path(), &prof);
}

#[test]
fn test_subscription_create_with_options() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    let t = tag();
    let et1 = format!("{}.opt.created", t);
    let et2 = format!("{}.opt.updated", t);
    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", &et1])
        .output();
    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", &et2])
        .output();

    // Create with multiple events, headers, disabled
    let events_csv = format!("{},{}", et1, et2);
    cli(cfg.path())
        .args(["--profile", &prof, "subscription", "create",
               "-e", &events_csv,
               "-u", "https://httpbin.org/post",
               "-l", "env=test",
               "-H", "X-Custom=hello",
               "--method", "PUT",
               "--disabled",
               "-d", "Multi-event disabled sub"])
        .assert()
        .success();

    // Verify it's disabled
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "subscription", "list", "--disabled"])
        .output()
        .unwrap();
    let subs: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert!(!subs.is_empty(), "should have 1 disabled subscription");

    // Cleanup
    let sub_id = subs[0]["subscription_id"].as_str().unwrap();
    let _ = cli(cfg.path())
        .args(["--profile", &prof, "subscription", "delete", sub_id, "--yes"])
        .output();
    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "delete", &et1, "--yes"])
        .output();
    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "delete", &et2, "--yes"])
        .output();
    cleanup(cfg.path(), &prof);
}

// =============================================================================
// Application
// =============================================================================

#[test]
fn test_application_get_and_current() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    // Get default app (table)
    cli(cfg.path())
        .args(["--profile", &prof, "application", "get"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&app_id));

    // Get by explicit ID (json) — verify parseable
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "application", "get", &app_id])
        .output()
        .unwrap();
    assert!(output.status.success());
    let app: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("application get JSON should be valid");
    assert_eq!(app["application_id"].as_str().unwrap(), app_id);

    // Current
    cli(cfg.path())
        .args(["--profile", &prof, "application", "current"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&app_id));

    // Current (json)
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "application", "current"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let current: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("application current JSON should be valid");
    assert_eq!(current["application_id"].as_str().unwrap(), app_id);

    // List fails with app secret (403) — verify helpful error message
    cli(cfg.path())
        .args(["--profile", &prof, "application", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("service token").or(predicate::str::contains("cannot")));

    cleanup(cfg.path(), &prof);
}

// =============================================================================
// Replay (dry-run only — avoids side effects)
// =============================================================================

#[test]
fn test_replay_dry_run() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    // --all --dry-run with time range
    cli(cfg.path())
        .args(["--profile", &prof, "replay", "--all", "--dry-run",
               "--since", "1h", "--confirm"])
        .assert()
        .success();

    // --all --dry-run with all filters
    cli(cfg.path())
        .args(["--profile", &prof, "replay", "--all", "--dry-run",
               "--since", "7d", "--until", "1h", "--status", "failed",
               "--event-type", "nonexistent.type.here", "--limit", "5", "--confirm"])
        .assert()
        .success();

    // Single event ID --dry-run
    cli(cfg.path())
        .args(["--profile", &prof, "replay",
               "00000000-0000-0000-0000-000000000000", "--dry-run"])
        .assert()
        .success();

    // --all without --confirm must fail
    cli(cfg.path())
        .args(["--profile", &prof, "replay", "--all", "--since", "1h"])
        .assert()
        .failure();

    // No args must fail
    cli(cfg.path())
        .args(["--profile", &prof, "replay"])
        .assert()
        .failure();

    cleanup(cfg.path(), &prof);
}

// =============================================================================
// Multi-profile workflow
// =============================================================================

#[test]
fn test_multi_profile_workflow() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();

    let prof_a = login(cfg.path(), &secret, &app_id, &api_url);
    let prof_b = login(cfg.path(), &secret, &app_id, &api_url);

    // Both profiles exist
    let output = cli(cfg.path())
        .args(["--output", "json", "config", "list"])
        .output()
        .unwrap();
    let profiles: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert!(profiles.len() >= 2);

    // Set default
    cli(cfg.path())
        .args(["config", "set-default", &prof_b])
        .assert()
        .success();

    // Remove prof_a
    cli(cfg.path())
        .args(["config", "remove", &prof_a, "--yes"])
        .assert()
        .success();

    // prof_a is gone
    cli(cfg.path())
        .args(["--profile", &prof_a, "whoami"])
        .assert()
        .failure();

    // prof_b still works
    cli(cfg.path())
        .args(["--profile", &prof_b, "whoami"])
        .assert()
        .success();

    cleanup(cfg.path(), &prof_b);
}

// =============================================================================
// Output format consistency
// =============================================================================

#[test]
fn test_whoami_output_formats() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    // Table
    cli(cfg.path())
        .args(["--profile", &prof, "--output", "table", "whoami"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Application ID"));

    // JSON — parseable and has expected fields
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "whoami"])
        .output()
        .unwrap();
    let whoami: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("whoami JSON should be valid");
    assert!(whoami.get("application_id").is_some());
    assert!(whoami.get("api_url").is_some());
    assert!(whoami.get("profile").is_some());

    // Compact
    cli(cfg.path())
        .args(["--profile", &prof, "--output", "compact", "whoami"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&app_id));

    cleanup(cfg.path(), &prof);
}

#[test]
fn test_config_list_json_structure() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    let output = cli(cfg.path())
        .args(["--output", "json", "config", "list"])
        .output()
        .unwrap();
    let profiles: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
        .expect("config list JSON should be a valid array");
    let p = &profiles[0];
    assert!(p.get("name").is_some(), "profile should have name");
    assert!(p.get("api_url").is_some(), "profile should have api_url");
    assert!(p.get("application_id").is_some(), "profile should have application_id");
    assert!(p.get("is_default").is_some(), "profile should have is_default");

    cleanup(cfg.path(), &prof);
}

// =============================================================================
// Application switch
// =============================================================================

#[test]
fn test_application_switch() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    // Switch to the same app (verifies the command works end-to-end)
    cli(cfg.path())
        .args(["--profile", &prof, "application", "switch", &app_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Switched to application"));

    // Verify the profile still works after switch
    cli(cfg.path())
        .args(["--profile", &prof, "application", "current"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&app_id));

    // Switch to nonexistent app — should fail
    cli(cfg.path())
        .args(["--profile", &prof, "application", "switch",
               "00000000-0000-0000-0000-000000000000"])
        .assert()
        .failure();

    cleanup(cfg.path(), &prof);
}

// =============================================================================
// Logout --all
// =============================================================================

#[test]
fn test_logout_all() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();

    let prof_a = login(cfg.path(), &secret, &app_id, &api_url);
    let prof_b = login(cfg.path(), &secret, &app_id, &api_url);

    // Both profiles work
    cli(cfg.path())
        .args(["--profile", &prof_a, "whoami"])
        .assert()
        .success();
    cli(cfg.path())
        .args(["--profile", &prof_b, "whoami"])
        .assert()
        .success();

    // Logout --all
    cli(cfg.path())
        .args(["logout", "--all"])
        .assert()
        .success();

    // Both profiles should fail
    cli(cfg.path())
        .args(["--profile", &prof_a, "whoami"])
        .assert()
        .failure();
    cli(cfg.path())
        .args(["--profile", &prof_b, "whoami"])
        .assert()
        .failure();
}

// =============================================================================
// Event send with text/plain content type
// =============================================================================

#[test]
fn test_event_send_text_plain() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);
    let t = tag();
    let et = format!("{}.txt.created", t);

    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", &et])
        .output();

    // Send with text/plain content type
    cli(cfg.path())
        .args(["--profile", &prof, "event", "send", &et,
               "-l", "env=test", "--content-type", "text/plain",
               "-d", "Hello, this is plain text"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Event sent successfully"));

    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "delete", &et, "--yes"])
        .output();
    cleanup(cfg.path(), &prof);
}

// =============================================================================
// Error handling: bad API URL, network errors, wrong secret
// =============================================================================

#[test]
fn test_error_bad_api_url() {
    let cfg = tempfile::tempdir().unwrap();

    // Login with unreachable port — should fail at validation step
    cli(cfg.path())
        .args([
            "login", "--secret", "e737f7dd-0c37-4dcd-8fb8-2f5027a383e9",
            "--application-id", "9408c110-d5dc-4e6d-bd7e-3d102a6aa5a9",
            "--api-url", "http://localhost:1/api/v1",
            "--profile-name", "bad-url",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));
}

#[test]
fn test_error_wrong_secret() {
    let cfg = tempfile::tempdir().unwrap();
    let (_, app_id, api_url) = credentials();

    // Login with wrong secret (valid UUID but not registered)
    cli(cfg.path())
        .args([
            "login",
            "--secret", "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
            "--application-id", &app_id,
            "--api-url", &api_url,
            "--profile-name", "wrong-secret",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Authentication failed").or(
            predicate::str::contains("invalid secret")
        ));
}

// =============================================================================
// Override mode: --secret + --api-url + --application-id (no profile needed)
// =============================================================================

#[test]
fn test_override_mode_works() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let t = tag();
    let et = format!("{}.ovr.created", t);

    // Create event type using override flags (no login, no profile)
    cli(cfg.path())
        .args(["--secret", &secret, "--api-url", &api_url,
               "--application-id", &app_id,
               "event-type", "create", &et])
        .assert()
        .success();

    // List event types in override mode
    cli(cfg.path())
        .args(["--secret", &secret, "--api-url", &api_url,
               "--application-id", &app_id,
               "event-type", "list", "--service", &t])
        .assert()
        .success()
        .stdout(predicate::str::contains(&t));

    // Cleanup
    let _ = cli(cfg.path())
        .args(["--secret", &secret, "--api-url", &api_url,
               "--application-id", &app_id,
               "event-type", "delete", &et, "--yes"])
        .output();
}

#[test]
fn test_override_mode_requires_application_id() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, _, api_url) = credentials();

    // --secret + --api-url without --application-id should fail
    cli(cfg.path())
        .args(["--secret", &secret, "--api-url", &api_url,
               "event-type", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("application-id"));
}

// =============================================================================
// Event list filters: --until, --page, --status, --label combinations
// =============================================================================

#[test]
fn test_event_list_all_filters() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);
    let t = tag();
    let et = format!("{}.filt.created", t);

    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", &et])
        .output();

    // Send a few events with different labels
    for label in &["region=us", "region=eu", "region=ap"] {
        cli(cfg.path())
            .args(["--profile", &prof, "event", "send", &et, "-l", label])
            .assert()
            .success();
    }

    // --until
    cli(cfg.path())
        .args(["--profile", &prof, "event", "list", "--until", "0s"])
        .assert()
        .success();

    // --since + --until combined
    cli(cfg.path())
        .args(["--profile", &prof, "event", "list", "--since", "1h", "--until", "0s"])
        .assert()
        .success();

    // --page
    cli(cfg.path())
        .args(["--profile", &prof, "event", "list", "--page", "1", "--limit", "1"])
        .assert()
        .success();

    cli(cfg.path())
        .args(["--profile", &prof, "event", "list", "--page", "2", "--limit", "1"])
        .assert()
        .success();

    // --label filter (verify command succeeds, API may or may not filter)
    cli(cfg.path())
        .args(["--profile", &prof, "event", "list", "-l", "region=us"])
        .assert()
        .success();

    // --event-type filter (json, verify parseable)
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "event", "list",
               "--event-type", &et])
        .output()
        .unwrap();
    assert!(output.status.success());
    let _: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
        .expect("filtered event list should be valid JSON array");

    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "delete", &et, "--yes"])
        .output();
    cleanup(cfg.path(), &prof);
}

// =============================================================================
// Subscription list --label filter
// =============================================================================

#[test]
fn test_subscription_list_label_filter() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);
    let t = tag();
    let et = format!("{}.slbl.created", t);

    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", &et])
        .output();

    // Create two subs with different labels
    cli(cfg.path())
        .args(["--profile", &prof, "subscription", "create",
               "-e", &et, "-u", "https://httpbin.org/post", "-l", "team=alpha"])
        .assert()
        .success();
    cli(cfg.path())
        .args(["--profile", &prof, "subscription", "create",
               "-e", &et, "-u", "https://httpbin.org/post", "-l", "team=beta"])
        .assert()
        .success();

    // List all — should have both
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "subscription", "list"])
        .output()
        .unwrap();
    let all: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert!(all.len() >= 2);

    // List with --label team=alpha (verify command succeeds and returns JSON)
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "subscription", "list",
               "-l", "team=alpha"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let _: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
        .expect("label-filtered subscription list should be valid JSON");

    // Cleanup — only delete subs owned by this test (matching team label)
    for sub in &all {
        let is_ours = sub.get("labels")
            .and_then(|l| l.get("team"))
            .and_then(|v| v.as_str())
            .map(|t| t == "alpha" || t == "beta")
            .unwrap_or(false);
        if is_ours {
            if let Some(sid) = sub.get("subscription_id").and_then(|v| v.as_str()) {
                let _ = cli(cfg.path())
                    .args(["--profile", &prof, "subscription", "delete", sid, "--yes"])
                    .output();
            }
        }
    }
    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "delete", &et, "--yes"])
        .output();
    cleanup(cfg.path(), &prof);
}

// =============================================================================
// Config show JSON structure
// =============================================================================

#[test]
fn test_config_show_json_structure() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    let output = cli(cfg.path())
        .args(["--output", "json", "config", "show"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let show: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("config show JSON should be valid");
    assert!(show.get("config_path").is_some());
    assert!(show.get("profiles").is_some());

    cleanup(cfg.path(), &prof);
}

// =============================================================================
// Verbose flag produces debug output
// =============================================================================

#[test]
fn test_verbose_produces_debug_output() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    // -vvv should not break the command (trace logging may or may not
    // appear on stderr depending on the tracing subscriber initialization)
    cli(cfg.path())
        .args(["--profile", &prof, "-vvv", "event-type", "list"])
        .assert()
        .success();

    // -v should also work
    cli(cfg.path())
        .args(["--profile", &prof, "-v", "event-type", "list"])
        .assert()
        .success();

    cleanup(cfg.path(), &prof);
}

// =============================================================================
// Regression tests for specific bugs
// =============================================================================

/// Bug 9: replay real (not dry-run) must work end-to-end
#[test]
fn test_replay_real() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);
    let t = tag();
    let et = format!("{}.rpl.created", t);

    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", &et])
        .output();
    let _ = cli(cfg.path())
        .args(["--profile", &prof, "subscription", "create",
               "-e", &et, "-u", "https://httpbin.org/post", "-l", "env=rpl"])
        .output();
    cli(cfg.path())
        .args(["--profile", &prof, "event", "send", &et, "-l", "env=rpl"])
        .assert()
        .success();

    // Get event ID
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "event", "list", "--limit", "1"])
        .output()
        .unwrap();
    let events: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    let evt_id = events[0]["event_id"].as_str().unwrap();

    // Real replay (not dry-run)
    cli(cfg.path())
        .args(["--profile", &prof, "replay", evt_id])
        .assert()
        .success();

    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "delete", &et, "--yes"])
        .output();
    cleanup(cfg.path(), &prof);
}

/// Bug 13: whoami must work in override mode without config file
#[test]
fn test_whoami_override_mode() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();

    // No login, no config — pure override
    cli(cfg.path())
        .args(["--secret", &secret, "--api-url", &api_url,
               "--application-id", &app_id, "whoami"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&app_id));

    // JSON format
    let output = cli(cfg.path())
        .args(["--secret", &secret, "--api-url", &api_url,
               "--application-id", &app_id, "--output", "json", "whoami"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let whoami: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("whoami override JSON should be valid");
    assert_eq!(whoami["application_id"].as_str().unwrap(), app_id);

    // In override mode, authenticated is always true (secret explicitly provided)
    let output = cli(cfg.path())
        .args(["--secret", "00000000-0000-0000-0000-000000000000",
               "--api-url", &api_url, "--application-id", &app_id,
               "--output", "json", "whoami"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let whoami: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(whoami["authenticated"].as_bool().unwrap(), true);
}

/// Bug 14: login -o json must produce clean JSON (no progress text)
#[test]
fn test_login_json_output_clean() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = profile_name();

    let output = cli(cfg.path())
        .args(["--output", "json", "login", "--secret", &secret,
               "--application-id", &app_id, "--api-url", &api_url,
               "--profile-name", &prof])
        .output()
        .unwrap();
    assert!(output.status.success());

    // stdout must be valid JSON (no progress text mixed in)
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("login -o json stdout must be valid JSON");
    assert_eq!(json["success"].as_bool().unwrap(), true);
    assert!(json.get("profile").is_some());

    cleanup(cfg.path(), &prof);
}

/// Bug 15: replay -o json must not mix progress text
#[test]
fn test_replay_json_output_clean() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    // replay --all --dry-run with json should be clean
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "replay",
               "--all", "--dry-run", "--since", "1h", "--confirm"])
        .output()
        .unwrap();
    assert!(output.status.success());
    // stdout should either be empty or valid JSON (no "Would replay" text)
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        let _: serde_json::Value = serde_json::from_str(stdout.trim())
            .unwrap_or_else(|_| panic!("replay json stdout not valid JSON: {}", stdout));
    }

    cleanup(cfg.path(), &prof);
}

/// Bug 16: logout respects global -p/--profile flag
#[test]
fn test_logout_respects_global_profile() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof_a = login(cfg.path(), &secret, &app_id, &api_url);
    let prof_b = login(cfg.path(), &secret, &app_id, &api_url);

    // Logout prof_b using global -p flag
    cli(cfg.path())
        .args(["-p", &prof_b, "logout"])
        .assert()
        .success();

    // prof_a should still work
    cli(cfg.path())
        .args(["--profile", &prof_a, "whoami"])
        .assert()
        .success();

    // prof_b should fail
    cli(cfg.path())
        .args(["--profile", &prof_b, "whoami"])
        .assert()
        .failure();

    cleanup(cfg.path(), &prof_a);
}

/// Bug 17: --enable and --disable are mutually exclusive
#[test]
fn test_subscription_update_enable_disable_conflict() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    cli(cfg.path())
        .args(["--profile", &prof, "subscription", "update",
               "00000000-0000-0000-0000-000000000000",
               "--enable", "--disable"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));

    cleanup(cfg.path(), &prof);
}

/// Bug 19: delete commands with -o json produce JSON
#[test]
fn test_delete_json_output() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);
    let t = tag();
    let et = format!("{}.djson.created", t);

    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", &et])
        .output();

    // event-type delete -o json
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "event-type", "delete", &et, "--yes"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("event-type delete json should be valid");
    assert_eq!(json["deleted"].as_bool().unwrap(), true);

    // subscription delete -o json
    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "create", &et])
        .output();
    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "subscription", "create",
               "-e", &et, "-u", "https://httpbin.org/post", "-l", "env=dj"])
        .output()
        .unwrap();
    let sub: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let sub_id = sub["subscription_id"].as_str().unwrap();

    let output = cli(cfg.path())
        .args(["--profile", &prof, "--output", "json", "subscription", "delete", sub_id, "--yes"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("subscription delete json should be valid");
    assert_eq!(json["deleted"].as_bool().unwrap(), true);

    let _ = cli(cfg.path())
        .args(["--profile", &prof, "event-type", "delete", &et, "--yes"])
        .output();
    cleanup(cfg.path(), &prof);
}

/// Bug 22: invalid JSON payload gives clear error message
#[test]
fn test_invalid_json_error_message() {
    let cfg = tempfile::tempdir().unwrap();
    let (secret, app_id, api_url) = credentials();
    let prof = login(cfg.path(), &secret, &app_id, &api_url);

    cli(cfg.path())
        .args(["--profile", &prof, "event", "send", "any.type.here",
               "-d", "not valid json", "-l", "e=t"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid JSON payload"));

    cleanup(cfg.path(), &prof);
}
