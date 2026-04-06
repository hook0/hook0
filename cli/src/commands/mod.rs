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
use anyhow::{Result, anyhow};

/// Sentinel profile name used when credentials are provided via CLI flags/env vars.
pub const OVERRIDE_PROFILE: &str = "override";

/// Resolve override profile from CLI flags (--secret + --api-url + --application-id).
/// Returns None if not all three are provided.
pub fn resolve_override_profile(cli: &crate::Cli) -> Result<Option<(String, Profile)>> {
    let (Some(_), Some(api_url)) = (&cli.secret, &cli.api_url) else {
        return Ok(None);
    };
    let app_id = cli.application_id.ok_or_else(|| {
        anyhow!("--application-id (or HOOK0_APPLICATION_ID) is required when using --secret and --api-url overrides")
    })?;
    Ok(Some((
        OVERRIDE_PROFILE.to_string(),
        Profile::new(api_url.clone(), app_id),
    )))
}

/// Get API client from CLI args or config
pub fn get_api_client(
    cli: &crate::Cli,
    profile_name: Option<&str>,
) -> Result<(ApiClient, String, Profile)> {
    // Check for direct secret/api_url overrides
    if let Some((name, profile)) = resolve_override_profile(cli)? {
        let secret = cli.secret.as_ref().unwrap();
        let api_url = cli.api_url.as_ref().unwrap();
        let client = ApiClient::new(api_url, secret);
        return Ok((client, name, profile));
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
