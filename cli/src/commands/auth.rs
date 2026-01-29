use anyhow::{anyhow, Result};
use clap::Args;
use dialoguer::{Confirm, Input, Password};
use uuid::Uuid;

use crate::api::ApiClient;
use crate::config::{Config, Profile};
use crate::output::{output_info, output_success, OutputFormat};
use crate::Cli;

#[derive(Args, Debug)]
pub struct LoginArgs {
    /// Application Secret (UUID token)
    #[arg(long)]
    pub secret: Option<String>,

    /// API URL
    #[arg(long, default_value = "https://app.hook0.com/api/v1")]
    pub api_url: String,

    /// Profile name to save credentials
    #[arg(long, short = 'n', default_value = "default")]
    pub profile_name: String,

    /// Application ID (required - the Application Secret is tied to this application)
    #[arg(long)]
    pub application_id: Option<Uuid>,
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

/// Check if we're running in an interactive terminal
fn is_interactive() -> bool {
    atty::is(atty::Stream::Stdin)
}

/// Helper to get a value from CLI arg, env var (with confirmation), or interactive prompt
fn get_value_interactive(
    cli_value: Option<&str>,
    env_var_name: &str,
    prompt_message: &str,
    is_secret: bool,
) -> Result<String> {
    // If CLI arg provided, use it directly
    if let Some(value) = cli_value {
        return Ok(value.to_string());
    }

    // Check for environment variable
    if let Ok(env_value) = std::env::var(env_var_name) {
        // In non-interactive mode, use env var directly
        if !is_interactive() {
            return Ok(env_value);
        }

        let display_value = if is_secret {
            format!(
                "{}...{}",
                &env_value[..8.min(env_value.len())],
                &env_value[env_value.len().saturating_sub(4)..]
            )
        } else {
            env_value.clone()
        };

        let use_env = Confirm::new()
            .with_prompt(format!(
                "Found {} in environment: {}. Use this value?",
                env_var_name, display_value
            ))
            .default(true)
            .interact()?;

        if use_env {
            return Ok(env_value);
        }
    }

    // Non-interactive mode without value available
    if !is_interactive() {
        return Err(anyhow!(
            "{} is required. Provide via --{} or {} environment variable.",
            prompt_message,
            prompt_message.to_lowercase().replace(' ', "-"),
            env_var_name
        ));
    }

    // Interactive prompt
    if is_secret {
        output_info(
            "You can find this in the Hook0 dashboard under Application > Settings > Secrets.",
        );
        Ok(Password::new().with_prompt(prompt_message).interact()?)
    } else {
        Ok(Input::new().with_prompt(prompt_message).interact_text()?)
    }
}

/// Helper to get application_id from CLI arg, env var (with confirmation), or interactive prompt
fn get_application_id_interactive(cli_value: Option<Uuid>) -> Result<Uuid> {
    // If CLI arg provided, use it directly
    if let Some(value) = cli_value {
        return Ok(value);
    }

    // Check for environment variable
    if let Ok(env_value) = std::env::var("HOOK0_APPLICATION_ID") {
        // In non-interactive mode, use env var directly
        if !is_interactive() {
            return Uuid::parse_str(&env_value)
                .map_err(|_| anyhow!("Invalid HOOK0_APPLICATION_ID format. Expected a UUID."));
        }

        let use_env = Confirm::new()
            .with_prompt(format!(
                "Found HOOK0_APPLICATION_ID in environment: {}. Use this value?",
                env_value
            ))
            .default(true)
            .interact()?;

        if use_env {
            return Uuid::parse_str(&env_value)
                .map_err(|_| anyhow!("Invalid HOOK0_APPLICATION_ID format. Expected a UUID."));
        }
    }

    // Non-interactive mode without value available
    if !is_interactive() {
        return Err(anyhow!(
            "Application ID is required. Provide via --application-id or HOOK0_APPLICATION_ID environment variable."
        ));
    }

    // Interactive prompt
    output_info(
        "You can find the Application ID in the Hook0 dashboard under Application > Settings.",
    );
    let input: String = Input::new().with_prompt("Application ID").interact_text()?;

    Uuid::parse_str(&input).map_err(|_| anyhow!("Invalid Application ID format. Expected a UUID."))
}

/// Login command - authenticate with an Application Secret
pub async fn login(cli: &Cli, args: &LoginArgs) -> Result<()> {
    output_info("Hook0 CLI Login");
    output_info("===============");

    // Get secret (CLI arg > env var with confirmation > interactive prompt)
    let secret = get_value_interactive(
        args.secret.as_deref(),
        "HOOK0_SECRET",
        "Application Secret",
        true,
    )?;

    // Validate the secret format (should be a UUID)
    let _secret_uuid =
        Uuid::parse_str(&secret).map_err(|_| anyhow!("Invalid secret format. Expected a UUID."))?;

    // Get application_id (CLI arg > env var with confirmation > interactive prompt)
    let application_id = get_application_id_interactive(args.application_id)?;

    // Get API URL (check env var if default value is used)
    let api_url = if args.api_url == "https://app.hook0.com/api/v1" {
        // Default value, check if env var is set
        if let Ok(env_value) = std::env::var("HOOK0_API_URL") {
            // In non-interactive mode, use env var directly
            if !is_interactive() {
                env_value
            } else {
                let use_env = Confirm::new()
                    .with_prompt(format!(
                        "Found HOOK0_API_URL in environment: {}. Use this value?",
                        env_value
                    ))
                    .default(true)
                    .interact()?;

                if use_env {
                    env_value
                } else {
                    args.api_url.clone()
                }
            }
        } else {
            args.api_url.clone()
        }
    } else {
        args.api_url.clone()
    };

    // Create API client to validate credentials
    let client = ApiClient::new(&api_url, &secret);

    // Validate credentials by fetching the application
    // Application Secrets are tied to a specific application, so we use get_current_application
    output_info("Validating credentials...");

    let app = client
        .get_current_application(&application_id)
        .await
        .map_err(|e| match e {
            crate::ApiError::Unauthorized => {
                anyhow!("Authentication failed: invalid secret or application ID mismatch.")
            }
            crate::ApiError::NotFound(_) => {
                anyhow!("Application not found with ID: {}", application_id)
            }
            _ => anyhow!("Failed to validate credentials: {}", e),
        })?;

    if cli.output != OutputFormat::Json {
        output_success(&format!(
            "Authenticated successfully!\n  Application: {} ({})\n  Organization: {}",
            app.name, app.application_id, app.organization_id
        ));
    }

    // Save to config
    let mut config = Config::load().unwrap_or_default();

    let profile = Profile::with_details(
        api_url.clone(),
        app.application_id,
        Some(app.organization_id),
        Some(format!("Application: {}", app.name)),
    );

    config.set_profile(&args.profile_name, profile);

    // Store secret in keyring first, then save config (avoid inconsistent state)
    Config::store_secret(&args.profile_name, &app.application_id, &secret)?;
    config.save()?;

    if cli.output == OutputFormat::Json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "profile": args.profile_name,
                "application_id": app.application_id,
                "application_name": app.name,
                "organization_id": app.organization_id,
                "api_url": api_url,
            })
        );
    } else {
        output_success(&format!(
            "Credentials saved to profile '{}'",
            args.profile_name
        ));
    }

    Ok(())
}

/// Logout command - remove stored credentials
pub async fn logout(_cli: &Cli, args: &LogoutArgs) -> Result<()> {
    let mut config = Config::load()?;

    if args.all {
        // Remove all profiles and their secrets
        let profiles: Vec<String> = config
            .list_profiles()
            .iter()
            .map(|s| s.to_string())
            .collect();

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
        let profile_name = args
            .profile_name
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

        output_success(&format!(
            "Credentials for profile '{}' have been removed.",
            profile_name
        ));
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
            application_id: Some(app_id),
        };

        assert!(args.secret.is_none());
        assert_eq!(args.profile_name, "default");
        assert_eq!(args.application_id, Some(app_id));
    }

    #[test]
    fn test_secret_uuid_parsing() {
        let valid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(Uuid::parse_str(valid).is_ok());

        let invalid = "not-a-uuid";
        assert!(Uuid::parse_str(invalid).is_err());
    }
}
