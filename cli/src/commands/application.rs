use anyhow::Result;
use clap::{Args, Subcommand};
use uuid::Uuid;

use crate::commands::require_auth;
use crate::config::Config;
use crate::output::{output_many, output_one, output_success, OutputFormat, TableOutput};
use crate::Cli;

#[derive(Subcommand, Debug)]
pub enum ApplicationCommands {
    /// List applications
    List(ListArgs),

    /// Get application details
    Get(GetArgs),

    /// Switch to a different application
    Switch(SwitchArgs),

    /// Show current application
    Current(CurrentArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Organization ID (uses default if not specified)
    #[arg(long)]
    pub organization_id: Option<Uuid>,
}

#[derive(Args, Debug)]
pub struct GetArgs {
    /// Application ID (uses default if not specified)
    pub application_id: Option<Uuid>,
}

#[derive(Args, Debug)]
pub struct SwitchArgs {
    /// Application ID to switch to
    pub application_id: Uuid,
}

#[derive(Args, Debug)]
pub struct CurrentArgs {}

pub async fn execute(cli: &Cli, cmd: &ApplicationCommands) -> Result<()> {
    match cmd {
        ApplicationCommands::List(args) => list(cli, args).await,
        ApplicationCommands::Get(args) => get(cli, args).await,
        ApplicationCommands::Switch(args) => switch(cli, args).await,
        ApplicationCommands::Current(args) => current(cli, args).await,
    }
}

async fn list(cli: &Cli, args: &ListArgs) -> Result<()> {
    let (client, _, profile) = require_auth(cli)?;

    let org_id = args
        .organization_id
        .or(profile.organization_id)
        .ok_or_else(|| {
            anyhow::anyhow!("No organization ID available. Specify with --organization-id")
        })?;

    let applications = client.list_applications(&org_id).await?;

    output_many(&applications, cli.output);

    Ok(())
}

async fn get(cli: &Cli, args: &GetArgs) -> Result<()> {
    let (client, _, profile) = require_auth(cli)?;

    let app_id = args.application_id.unwrap_or(profile.application_id);
    let application = client.get_application(&app_id).await?;

    if cli.output == OutputFormat::Json {
        output_one(&application, cli.output);
    } else {
        let mut details = vec![
            ("Application ID", application.application_id.to_string()),
            ("Name", application.name.clone()),
            ("Organization ID", application.organization_id.to_string()),
        ];

        if let Some(created_at) = application.created_at {
            details.push(("Created At", created_at.to_rfc3339()));
        }

        if let Some(quotas) = &application.quotas {
            details.push(("Events/Day Limit", quotas.events_per_day_limit.to_string()));
            details.push((
                "Retention Days",
                quotas.days_of_events_retention_limit.to_string(),
            ));
        }

        TableOutput::print_details(details);
    }

    Ok(())
}

async fn switch(cli: &Cli, args: &SwitchArgs) -> Result<()> {
    let (client, profile_name, _) = require_auth(cli)?;

    // Verify the application exists and we have access
    let application = client.get_application(&args.application_id).await?;

    // Update the profile with the new application ID
    let mut config = Config::load()?;

    if let Ok((_, profile)) = config.get_profile(Some(&profile_name)) {
        // Get the current secret
        let secret = Config::get_secret(&profile_name, &profile.application_id)?;

        // Create updated profile
        let new_profile = crate::config::Profile::with_details(
            profile.api_url.clone(),
            args.application_id,
            Some(application.organization_id),
            Some(format!("Application: {}", application.name)),
        );

        // Update config
        config.set_profile(&profile_name, new_profile);
        config.save()?;

        // Re-store secret with new application ID
        Config::store_secret(&profile_name, &args.application_id, &secret)?;
    }

    if cli.output == OutputFormat::Json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "application_id": args.application_id,
                "application_name": application.name,
            })
        );
    } else {
        output_success(&format!(
            "Switched to application '{}' ({})",
            application.name, args.application_id
        ));
    }

    Ok(())
}

async fn current(cli: &Cli, _args: &CurrentArgs) -> Result<()> {
    let (client, profile_name, profile) = require_auth(cli)?;

    let application = client.get_application(&profile.application_id).await?;

    if cli.output == OutputFormat::Json {
        println!(
            "{}",
            serde_json::json!({
                "profile": profile_name,
                "application_id": application.application_id,
                "application_name": application.name,
                "organization_id": application.organization_id,
            })
        );
    } else {
        println!("Profile: {}", profile_name);
        println!(
            "Application: {} ({})",
            application.name, application.application_id
        );
        println!("Organization: {}", application.organization_id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_parsing() {
        let valid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(Uuid::parse_str(valid).is_ok());

        let invalid = "not-a-uuid";
        assert!(Uuid::parse_str(invalid).is_err());
    }
}
