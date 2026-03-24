use std::collections::HashMap;
use std::fs;
use std::path::Path;

use keyring::Entry;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use super::profiles::Profile;
use super::{config_file_path, ensure_config_dir};

const KEYRING_SERVICE: &str = "hook0-cli";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found. Run 'hook0 login' first.")]
    NotFound,

    #[error("Failed to read configuration: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse configuration: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Failed to serialize configuration: {0}")]
    SerializeError(#[from] toml::ser::Error),

    #[error("Profile '{0}' not found")]
    ProfileNotFound(String),

    #[error("Keyring error: {0}")]
    KeyringError(String),

    #[error("No default profile configured")]
    NoDefaultProfile,
}

/// Main configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Default profile name
    #[serde(default)]
    pub default_profile: Option<String>,

    /// Map of profile name to profile configuration
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

impl Config {
    /// Load configuration from the default path
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from(&config_file_path())
    }

    /// Load configuration from a specific path
    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::NotFound);
        }

        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to the default path
    pub fn save(&self) -> Result<(), ConfigError> {
        self.save_to(&config_file_path())
    }

    /// Save configuration to a specific path
    pub fn save_to(&self, path: &Path) -> Result<(), ConfigError> {
        ensure_config_dir()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Get a profile by name, or the default profile if name is None
    pub fn get_profile(&self, name: Option<&str>) -> Result<(String, &Profile), ConfigError> {
        let profile_name = match name {
            Some(n) => n.to_string(),
            None => self
                .default_profile
                .clone()
                .ok_or(ConfigError::NoDefaultProfile)?,
        };

        self.profiles
            .get(&profile_name)
            .map(|p| (profile_name.clone(), p))
            .ok_or(ConfigError::ProfileNotFound(profile_name))
    }

    /// Add or update a profile
    pub fn set_profile(&mut self, name: &str, profile: Profile) {
        self.profiles.insert(name.to_string(), profile);

        // Set as default if it's the first profile
        if self.default_profile.is_none() {
            self.default_profile = Some(name.to_string());
        }
    }

    /// Remove a profile
    pub fn remove_profile(&mut self, name: &str) -> Option<Profile> {
        let profile = self.profiles.remove(name);

        // If we removed the default profile, set a new default
        if self.default_profile.as_deref() == Some(name) {
            self.default_profile = self.profiles.keys().next().cloned();
        }

        profile
    }

    /// Set the default profile
    pub fn set_default_profile(&mut self, name: &str) -> Result<(), ConfigError> {
        if !self.profiles.contains_key(name) {
            return Err(ConfigError::ProfileNotFound(name.to_string()));
        }
        self.default_profile = Some(name.to_string());
        Ok(())
    }

    /// List all profile names
    pub fn list_profiles(&self) -> Vec<&str> {
        self.profiles.keys().map(|s| s.as_str()).collect()
    }

    /// Store a secret in the OS keyring
    pub fn store_secret(
        profile_name: &str,
        application_id: &Uuid,
        secret: &str,
    ) -> Result<(), ConfigError> {
        let key = format!("{}-{}", profile_name, application_id);
        let entry = Entry::new(KEYRING_SERVICE, &key)
            .map_err(|e| ConfigError::KeyringError(e.to_string()))?;
        entry
            .set_password(secret)
            .map_err(|e| ConfigError::KeyringError(e.to_string()))?;
        Ok(())
    }

    /// Retrieve a secret from the OS keyring
    pub fn get_secret(profile_name: &str, application_id: &Uuid) -> Result<String, ConfigError> {
        let key = format!("{}-{}", profile_name, application_id);
        let entry = Entry::new(KEYRING_SERVICE, &key)
            .map_err(|e| ConfigError::KeyringError(e.to_string()))?;
        entry
            .get_password()
            .map_err(|e| ConfigError::KeyringError(e.to_string()))
    }

    /// Delete a secret from the OS keyring
    pub fn delete_secret(profile_name: &str, application_id: &Uuid) -> Result<(), ConfigError> {
        let key = format!("{}-{}", profile_name, application_id);
        let entry = Entry::new(KEYRING_SERVICE, &key)
            .map_err(|e| ConfigError::KeyringError(e.to_string()))?;
        entry
            .delete_credential()
            .map_err(|e| ConfigError::KeyringError(e.to_string()))?;
        Ok(())
    }

    /// Check if a secret exists in the OS keyring
    pub fn has_secret(profile_name: &str, application_id: &Uuid) -> bool {
        Self::get_secret(profile_name, application_id).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");
        (temp_dir, config_path)
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.default_profile.is_none());
        assert!(config.profiles.is_empty());
    }

    #[test]
    fn test_config_save_and_load() {
        let (_temp_dir, config_path) = create_test_config();

        let mut config = Config::default();
        let app_id = Uuid::new_v4();
        config.set_profile(
            "dev",
            Profile::new("https://api.dev.com".to_string(), app_id),
        );

        config.save_to(&config_path).expect("save should work");

        let loaded = Config::load_from(&config_path).expect("load should work");
        assert_eq!(loaded.default_profile, Some("dev".to_string()));
        assert!(loaded.profiles.contains_key("dev"));
    }

    #[test]
    fn test_config_not_found() {
        let result = Config::load_from(Path::new("/nonexistent/path/config.toml"));
        assert!(matches!(result, Err(ConfigError::NotFound)));
    }

    #[test]
    fn test_get_profile() {
        let mut config = Config::default();
        let app_id = Uuid::new_v4();
        config.set_profile(
            "dev",
            Profile::new("https://api.dev.com".to_string(), app_id),
        );
        config.set_profile(
            "prod",
            Profile::new("https://api.prod.com".to_string(), app_id),
        );

        // Get specific profile
        let (name, profile) = config
            .get_profile(Some("prod"))
            .expect("profile should exist");
        assert_eq!(name, "prod");
        assert_eq!(profile.api_url, "https://api.prod.com");

        // Get default profile
        let (name, _) = config
            .get_profile(None)
            .expect("default profile should exist");
        assert_eq!(name, "dev");
    }

    #[test]
    fn test_get_profile_not_found() {
        let config = Config::default();
        let result = config.get_profile(Some("nonexistent"));
        assert!(matches!(result, Err(ConfigError::ProfileNotFound(_))));
    }

    #[test]
    fn test_remove_profile() {
        let mut config = Config::default();
        let app_id = Uuid::new_v4();
        config.set_profile(
            "dev",
            Profile::new("https://api.dev.com".to_string(), app_id),
        );
        config.set_profile(
            "prod",
            Profile::new("https://api.prod.com".to_string(), app_id),
        );

        assert_eq!(config.default_profile, Some("dev".to_string()));

        config.remove_profile("dev");
        assert!(!config.profiles.contains_key("dev"));
        assert_eq!(config.default_profile, Some("prod".to_string()));
    }

    #[test]
    fn test_set_default_profile() {
        let mut config = Config::default();
        let app_id = Uuid::new_v4();
        config.set_profile(
            "dev",
            Profile::new("https://api.dev.com".to_string(), app_id),
        );
        config.set_profile(
            "prod",
            Profile::new("https://api.prod.com".to_string(), app_id),
        );

        config
            .set_default_profile("prod")
            .expect("should set default");
        assert_eq!(config.default_profile, Some("prod".to_string()));
    }

    #[test]
    fn test_set_default_profile_not_found() {
        let mut config = Config::default();
        let result = config.set_default_profile("nonexistent");
        assert!(matches!(result, Err(ConfigError::ProfileNotFound(_))));
    }

    #[test]
    fn test_list_profiles() {
        let mut config = Config::default();
        let app_id = Uuid::new_v4();
        config.set_profile(
            "dev",
            Profile::new("https://api.dev.com".to_string(), app_id),
        );
        config.set_profile(
            "prod",
            Profile::new("https://api.prod.com".to_string(), app_id),
        );

        let profiles = config.list_profiles();
        assert_eq!(profiles.len(), 2);
        assert!(profiles.contains(&"dev"));
        assert!(profiles.contains(&"prod"));
    }

    #[test]
    fn test_config_serialization_format() {
        let mut config = Config::default();
        let app_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("valid uuid");
        config.set_profile(
            "dev",
            Profile::with_details(
                "https://app.hook0.com/api/v1".to_string(),
                app_id,
                None,
                Some("Development environment".to_string()),
            ),
        );

        let toml_str = toml::to_string_pretty(&config).expect("serialization should work");
        assert!(toml_str.contains("default_profile"));
        assert!(toml_str.contains("[profiles.dev]"));
        assert!(toml_str.contains("api_url"));
    }
}
