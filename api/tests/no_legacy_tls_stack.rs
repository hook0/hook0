//! Supply-chain guard: the workspace lockfile must not carry the legacy rustls 0.21 stack.
//!
//! Design: aws-sdk-s3's default `rustls` feature compiles a second TLS stack (rustls 0.21 over
//! rustls-webpki 0.101, hyper 0.14, hyper-rustls 0.24) beside the one the S3 clients actually run.
//! That line gets advisories with no 0.21.x fix, so the only way to keep the osv scan green is to
//! keep it out of the tree. api/Cargo.toml and output-worker/Cargo.toml therefore declare
//! aws-sdk-s3 with `default-features = false`. Feature unification is workspace-wide: one member
//! falling back to the default feature set brings the whole legacy stack back for everyone, and
//! nothing in the build would notice. This test reads Cargo.lock only (no network, no database), so
//! it fails in `cargo test` before the scanner has to.

use std::path::Path;

/// Crates that only the legacy TLS stack pulls, with the version prefix that identifies that copy.
/// The current stack uses rustls 0.23, rustls-webpki 0.103, hyper 1 and hyper-rustls 0.27, so a
/// prefix match never trips on them.
const LEGACY: &[(&str, &str)] = &[
    ("rustls", "0.21."),
    ("rustls-webpki", "0.101."),
    ("hyper-rustls", "0.24."),
    ("tokio-rustls", "0.24."),
];

/// `name version` pairs from the workspace Cargo.lock that match a [`LEGACY`] entry, sorted.
fn legacy_tls_packages() -> Vec<String> {
    let lockfile = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the api crate always sits one level below the repository root")
        .join("Cargo.lock");
    let content = std::fs::read_to_string(&lockfile).expect("the workspace Cargo.lock is readable");

    // Cargo.lock lists each package as a `[[package]]` table whose first two keys are `name` and
    // `version`, in that order, which is all this needs; no TOML parser required.
    let mut found = Vec::new();
    let mut name: Option<&str> = None;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("name = ") {
            name = Some(value.trim_matches('"'));
        } else if let Some(value) = line.strip_prefix("version = ") {
            let version = value.trim_matches('"');
            if let Some(name) = name.take()
                && LEGACY
                    .iter()
                    .any(|(legacy, prefix)| *legacy == name && version.starts_with(prefix))
            {
                found.push(format!("{name} {version}"));
            }
        }
    }
    found.sort();
    found
}

#[test]
fn workspace_lockfile_carries_no_legacy_tls_stack() {
    let found = legacy_tls_packages();
    assert!(
        found.is_empty(),
        "legacy TLS stack back in Cargo.lock: {}. Some workspace member enables aws-sdk-s3's default \
         features; declare it with `default-features = false` as api/Cargo.toml does.",
        found.join(", ")
    );
}
