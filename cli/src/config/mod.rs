mod file;
mod profiles;

pub use file::{Config, ConfigError};
pub use profiles::Profile;

use std::path::PathBuf;

/// Get the configuration directory path.
///
/// Respects `HOOK0_CONFIG_DIR` env var to override the default platform path.
pub fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("HOOK0_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    dirs::config_dir()
        .expect("Could not determine config directory")
        .join("hook0")
}

/// Get the configuration file path
pub fn config_file_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// Ensure the configuration directory exists
pub fn ensure_config_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(config_dir())
}
