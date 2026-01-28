use anyhow::{anyhow, Result};
use clap::Args;
use dialoguer::Password;
use uuid::Uuid;

use crate::api::ApiClient;
use crate::config::{Config, Profile};
use crate::output::{output_info, output_success, OutputFormat};
use crate::Cli;

#[derive(Args, Debug)]
pub struct LoginArgs {
    /// Application Secret (UUID token)
    #[arg(long, env = "HOOK0_SECRET")]
    pub secret: Option<String>,

    /// API URL
    #[arg(long, env = "HOOK0_API_URL", default_value = "https://app.hook0.com/api/v1")]
    pub api_url: String,

    /// Profile name to save credentials
    #[arg(long, short = 'n', default_value = "default")]
    pub profile_name: String,

    /// Application ID (required - the Application Secret is tied to this application)
    #[arg(long, env = "HOOK0_APPLICATION_ID")]
    pub application_id: Uuid,
}

#[derive(Args, Debug)]
pub struct LogoutArgs {
    /// Profile name to remove credentials for
    #[arg(long, short = 'n')]
    pub profile_name: Option<String>,

    /// Remove all stored credentials
    #[arg(long)]
    pub all: bool,
}

#[derive(Args, Debug)]
pub struct WhoamiArgs {}

/// Login command - authenticate with an Application Secret
pub async fn login(cli: &Cli, args: &LoginArgs) -> Result<()> {
    // Get secret interactively if not provided
    let secret = match &args.secret {
        Some(s) => s.clone(),
        None => {
            output_info("Enter your Application Secret to authenticate.");
            output_info("You can find this in the Hook0 dashboard under Application > Settings > Secrets.");

            Password::new()
                .with_prompt("Application Secret")
                .interact()?
        }
    };

    // Validate the secret format (should be a UUID)
    let _secret_uuid = Uuid::parse_str(&secret)
        .map_err(|_| anyhow!("Invalid secret format. Expected a UUID."))?;

    // Create API client to validate credentials
    let client = ApiClient::new(&args.api_url, &secret);

    // Validate credentials by fetching the application
    // Application Secrets are tied to a specific application, so we use get_current_application
    output_info("Validating credentials...");

    let app = client
        .get_current_application(&args.application_id)
        .await
        .map_err(|e| match e {
            crate::ApiError::Unauthorized => {
                anyhow!("Authentication failed: invalid secret or application ID mismatch.")
            }
            crate::ApiError::NotFound(_) => {
                anyhow!("Application not found with ID: {}", args.application_id)
            }
            _ => anyhow!("Failed to validate credentials: {}", e),
        })?;

    output_success(&format!(
        "Authenticated successfully!\n  Application: {} ({})\n  Organization: {}",
        app.name,
        app.application_id,
        app.organization_id
    ));

    // Save to config
    let mut config = Config::load().unwrap_or_default();

    let profile = Profile::with_details(
        args.api_url.clone(),
        app.application_id,
        Some(app.organization_id),
        Some(format!("Application: {}", app.name)),
    );

    config.set_profile(&args.profile_name, profile);
    config.save()?;

    // Store secret in keyring
    Config::store_secret(&args.profile_name, &app.application_id, &secret)?;

    output_success(&format!(
        "Credentials saved to profile '{}'",
        args.profile_name
    ));

    if cli.output == OutputFormat::Json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "profile": args.profile_name,
                "application_id": app.application_id,
                "application_name": app.name,
                "organization_id": app.organization_id,
                "api_url": args.api_url,
            })
        );
    }

    Ok(())
}

/// Logout command - remove stored credentials
pub async fn logout(_cli: &Cli, args: &LogoutArgs) -> Result<()> {
    let mut config = Config::load()?;

    if args.all {
        // Remove all profiles and their secrets
        let profiles: Vec<String> = config.list_profiles().iter().map(|s| s.to_string()).collect();

        for name in &profiles {
            if let Ok((_, profile)) = config.get_profile(Some(name)) {
                let _ = Config::delete_secret(name, &profile.application_id);
            }
            config.remove_profile(name);
        }

        config.save()?;
        output_success("All credentials have been removed.");
    } else {
        // Get profile name first to avoid borrowing issues
        let profile_name = args.profile_name
            .clone()
            .or_else(|| config.default_profile.clone())
            .unwrap_or_else(|| "default".to_string());

        let application_id = {
            let (_, profile) = config.get_profile(Some(&profile_name))?;
            profile.application_id
        };

        // Delete secret from keyring
        Config::delete_secret(&profile_name, &application_id)?;

        // Remove profile from config
        config.remove_profile(&profile_name);
        config.save()?;

        output_success(&format!("Credentials for profile '{}' have been removed.", profile_name));
    }

    Ok(())
}

/// Whoami command - display current authentication info
pub async fn whoami(cli: &Cli, _args: &WhoamiArgs) -> Result<()> {
    let config = Config::load()?;
    let profile_name = cli.profile.as_deref();
    let (name, profile) = config.get_profile(profile_name)?;

    // Check if we have a valid secret
    let has_secret = Config::has_secret(&name, &profile.application_id);

    if cli.output == OutputFormat::Json {
        println!(
            "{}",
            serde_json::json!({
                "profile": name,
                "application_id": profile.application_id,
                "organization_id": profile.organization_id,
                "api_url": profile.api_url,
                "authenticated": has_secret,
                "description": profile.description,
            })
        );
    } else {
        println!("Profile: {}", name);
        println!("Application ID: {}", profile.application_id);
        if let Some(org_id) = &profile.organization_id {
            println!("Organization ID: {}", org_id);
        }
        println!("API URL: {}", profile.api_url);
        println!("Authenticated: {}", if has_secret { "Yes" } else { "No" });
        if let Some(desc) = &profile.description {
            println!("Description: {}", desc);
        }

        if config.default_profile.as_deref() == Some(&name) {
            println!("\n(This is the default profile)");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_args_defaults() {
        let app_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let args = LoginArgs {
            secret: None,
            api_url: "https://app.hook0.com/api/v1".to_string(),
            profile_name: "default".to_string(),
            application_id: app_id,
        };

        assert!(args.secret.is_none());
        assert_eq!(args.profile_name, "default");
        assert_eq!(args.application_id, app_id);
    }

    #[test]
    fn test_secret_uuid_parsing() {
        let valid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(Uuid::parse_str(valid).is_ok());

        let invalid = "not-a-uuid";
        assert!(Uuid::parse_str(invalid).is_err());
    }
}
