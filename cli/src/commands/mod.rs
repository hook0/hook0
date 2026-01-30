pub mod application;
pub mod auth;
pub mod completion;
pub mod config;
pub mod event;
pub mod event_type;
pub mod example;
pub mod listen;
pub mod replay;
pub mod subscription;

use crate::api::ApiClient;
use crate::config::{Config, Profile};
use anyhow::{anyhow, Result};

/// Get API client from CLI args or config
pub fn get_api_client(
    cli: &crate::Cli,
    profile_name: Option<&str>,
) -> Result<(ApiClient, String, Profile)> {
    // Check for direct secret/api_url overrides
    if let (Some(secret), Some(api_url)) = (&cli.secret, &cli.api_url) {
        // Use direct credentials
        let client = ApiClient::new(api_url, secret);
        let profile = Profile::new(api_url.clone(), uuid::Uuid::nil());
        return Ok((client, "override".to_string(), profile));
    }

    // Load from config
    let config = Config::load()?;
    let (name, profile) = config.get_profile(profile_name.or(cli.profile.as_deref()))?;

    // Get secret from keyring
    let secret = Config::get_secret(&name, &profile.application_id)?;

    let api_url = cli.api_url.as_deref().unwrap_or(&profile.api_url);
    let client = ApiClient::new(api_url, &secret);

    Ok((client, name, profile.clone()))
}

/// Get API client, requiring authentication
pub fn require_auth(cli: &crate::Cli) -> Result<(ApiClient, String, Profile)> {
    get_api_client(cli, None)
        .map_err(|e| anyhow!("{}\n\nHint: Run 'hook0 login' to authenticate first.", e))
}
