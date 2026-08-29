//! Rewrites the artefact the dashboard reads.
//!
//! The guard beside it does the same work and compares rather than writes, so this is what somebody
//! runs once they have changed an example and want the change to reach the screen.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use hook0_dashboard_examples::{ARTEFACT, emit, sdks};

fn main() -> ExitCode {
    let tree = repository();
    match write(&tree) {
        Ok(report) => {
            print!("{report}");
            ExitCode::SUCCESS
        }
        Err(reason) => {
            eprintln!("dashboard-examples: {reason}");
            ExitCode::FAILURE
        }
    }
}

fn write(tree: &Path) -> Result<String, String> {
    let found = sdks(tree).map_err(|cause| cause.to_string())?;
    let body = emit::artefact(&found);
    let path = tree.join(ARTEFACT);

    if let Some(directory) = path.parent() {
        std::fs::create_dir_all(directory)
            .map_err(|cause| format!("{}: cannot be created: {cause}", directory.display()))?;
    }
    std::fs::write(&path, &body)
        .map_err(|cause| format!("{}: cannot be written: {cause}", path.display()))?;

    Ok(format!(
        "{ARTEFACT}: {} SDKs, {} bytes\n{}",
        found.len(),
        body.len(),
        found
            .iter()
            .map(|sdk| format!("  {} ({})\n", sdk.target, sdk.registry))
            .collect::<String>()
    ))
}

/// The repository this crate sits in, which is two levels above it wherever it was built from.
fn repository() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}
