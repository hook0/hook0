mod file;
mod profiles;

pub use file::{Config, ConfigError};
pub use profiles::Profile;

use std::path::PathBuf;

/// Get the configuration directory path
pub fn config_dir() -> PathBuf {
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
