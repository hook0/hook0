
use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use dialoguer::{Confirm, Input, Password, Select};

use crate::api::models::EventTypePost;
use crate::api::ApiClient;
use crate::config::{config_file_path, Config, Profile};
use crate::output::{output_info, output_success, output_warning, OutputFormat, TableOutput};
use crate::Cli;

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// List all profiles
    List(ListArgs),

    /// Show current configuration
    Show(ShowArgs),

    /// Set default profile
    SetDefault(SetDefaultArgs),

    /// Remove a profile
    Remove(RemoveArgs),

    /// Show configuration file path
    Path(PathArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {}

#[derive(Args, Debug)]
pub struct ShowArgs {}

#[derive(Args, Debug)]
pub struct SetDefaultArgs {
    /// Profile name to set as default
    pub profile: String,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// Profile name to remove
    pub profile: String,

    /// Skip confirmation
    #[arg(long, short = 'y')]
    pub yes: bool,
}

#[derive(Args, Debug)]
pub struct PathArgs {}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Skip interactive prompts and use defaults
    #[arg(long)]
    pub non_interactive: bool,
}

#[derive(Args, Debug)]
pub struct QuickstartArgs {
    /// Local port to forward webhooks to
    #[arg(default_value = "3000")]
    pub port: u16,

    /// Event type to create (optional)
    #[arg(long)]
    pub event_type: Option<String>,
}

pub async fn execute(cli: &Cli, cmd: &ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::List(args) => list(cli, args).await,
        ConfigCommands::Show(args) => show(cli, args).await,
        ConfigCommands::SetDefault(args) => set_default(cli, args).await,
        ConfigCommands::Remove(args) => remove(cli, args).await,
        ConfigCommands::Path(args) => path(cli, args).await,
    }
}

async fn list(cli: &Cli, _args: &ListArgs) -> Result<()> {
    let config = Config::load().unwrap_or_default();

    if config.profiles.is_empty() {
        output_warning("No profiles configured. Run 'hook0 login' to create one.");
        return Ok(());
    }

    if cli.output == OutputFormat::Json {
        let profiles: Vec<serde_json::Value> = config
            .profiles
            .iter()
            .map(|(name, profile)| {
                serde_json::json!({
                    "name": name,
                    "api_url": profile.api_url,
                    "application_id": profile.application_id,
                    "organization_id": profile.organization_id,
                    "description": profile.description,
                    "is_default": config.default_profile.as_deref() == Some(name.as_str()),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&profiles).expect("json serialization"));
    } else {
        let headers = vec!["Profile", "API URL", "Application ID", "Default"];
        let rows: Vec<Vec<String>> = config
            .profiles
            .iter()
            .map(|(name, profile)| {
                vec![
                    name.clone(),
                    profile.api_url.clone(),
                    profile.application_id.to_string(),
                    if config.default_profile.as_deref() == Some(name.as_str()) {
                        "Yes".to_string()
                    } else {
                        "No".to_string()
                    },
                ]
            })
            .collect();

        TableOutput::print_custom(headers, rows);
    }

    Ok(())
}

async fn show(cli: &Cli, _args: &ShowArgs) -> Result<()> {
    let config = Config::load().unwrap_or_default();

    if cli.output == OutputFormat::Json {
        let json = serde_json::json!({
            "config_path": config_file_path().to_string_lossy(),
            "default_profile": config.default_profile,
            "profiles": config.profiles.keys().collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&json).expect("json serialization"));
    } else {
        println!("Configuration file: {}", config_file_path().display());
        println!("Default profile: {}", config.default_profile.as_deref().unwrap_or("(none)"));
        println!("Profiles: {}", config.profiles.len());

        for (name, profile) in &config.profiles {
            let is_default = config.default_profile.as_deref() == Some(name.as_str());
            println!(
                "\n[{}]{}",
                name,
                if is_default { " (default)" } else { "" }
            );
            println!("  API URL: {}", profile.api_url);
            println!("  Application ID: {}", profile.application_id);
            if let Some(org_id) = &profile.organization_id {
                println!("  Organization ID: {}", org_id);
            }
            if let Some(desc) = &profile.description {
                println!("  Description: {}", desc);
            }
        }
    }

    Ok(())
}

async fn set_default(_cli: &Cli, args: &SetDefaultArgs) -> Result<()> {
    let mut config = Config::load()?;

    config.set_default_profile(&args.profile)?;
    config.save()?;

    output_success(&format!("Default profile set to '{}'", args.profile));

    Ok(())
}

async fn remove(_cli: &Cli, args: &RemoveArgs) -> Result<()> {
    let mut config = Config::load()?;

    // Check if profile exists
    let (_, profile) = config.get_profile(Some(&args.profile))?;

    if !args.yes {
        let confirmed = Confirm::new()
            .with_prompt(format!("Remove profile '{}'?", args.profile))
            .default(false)
            .interact()?;

        if !confirmed {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Delete secret from keyring
    let _ = Config::delete_secret(&args.profile, &profile.application_id);

    // Remove profile
    config.remove_profile(&args.profile);
    config.save()?;

    output_success(&format!("Profile '{}' removed", args.profile));

    Ok(())
}

async fn path(_cli: &Cli, _args: &PathArgs) -> Result<()> {
    println!("{}", config_file_path().display());
    Ok(())
}

/// Interactive initialization wizard
pub async fn init(_cli: &Cli, args: &InitArgs) -> Result<()> {
    println!("Welcome to Hook0 CLI! Let's set up your first webhook.\n");

    if args.non_interactive {
        return Err(anyhow!(
            "Non-interactive init requires --secret and --api-url. Use 'hook0 login' instead."
        ));
    }

    // Step 1: Get API URL
    let api_url: String = Input::new()
        .with_prompt("API URL")
        .default("https://app.hook0.com/api/v1".to_string())
        .interact_text()?;

    // Step 2: Get secret
    output_info("Enter your Application Secret (found in Dashboard > Application > Settings > Secrets)");
    let secret: String = Password::new()
        .with_prompt("Application Secret")
        .interact()?;

    // Step 3: Validate and get application info
    output_info("Validating credentials...");
    let client = ApiClient::new(&api_url, &secret);

    let orgs = client.list_organizations().await?;
    if orgs.is_empty() {
        return Err(anyhow!("No organizations found for this secret."));
    }

    let mut apps = Vec::new();
    for org in &orgs {
        let org_apps = client.list_applications(&org.organization_id).await?;
        apps.extend(org_apps);
    }

    if apps.is_empty() {
        return Err(anyhow!("No applications found."));
    }

    // Select application if multiple
    let app = if apps.len() == 1 {
        apps.remove(0)
    } else {
        let app_names: Vec<String> = apps.iter().map(|a| a.name.clone()).collect();
        let selection = Select::new()
            .with_prompt("Select application")
            .items(&app_names)
            .interact()?;
        apps.remove(selection)
    };

    output_success(&format!("Authenticated with application: {}", app.name));

    // Step 4: Profile name
    let profile_name: String = Input::new()
        .with_prompt("Profile name")
        .default("default".to_string())
        .interact_text()?;

    // Save configuration
    let mut config = Config::load().unwrap_or_default();
    let profile = Profile::with_details(
        api_url.clone(),
        app.application_id,
        Some(app.organization_id),
        Some(format!("Application: {}", app.name)),
    );
    config.set_profile(&profile_name, profile);
    config.save()?;
    Config::store_secret(&profile_name, &app.application_id, &secret)?;

    output_success(&format!("Configuration saved to profile '{}'", profile_name));

    // Step 5: Optional - Create event type
    let create_event_type = Confirm::new()
        .with_prompt("Create your first event type?")
        .default(true)
        .interact()?;

    if create_event_type {
        // Dynamically fetch existing event types from the API to suggest
        let existing_types = client.list_event_types(&app.application_id).await.unwrap_or_default();

        let mut suggestions: Vec<String> = existing_types
            .iter()
            .map(|et| et.full_name())
            .collect();

        // If no existing types, provide some common patterns as examples
        if suggestions.is_empty() {
            // Build suggestions from common webhook patterns dynamically
            let common_services = ["user", "order", "invoice", "payment"];
            let common_verbs = ["created", "updated", "deleted"];
            suggestions = common_services
                .iter()
                .flat_map(|s| common_verbs.iter().map(move |v| format!("{}.account.{}", s, v)))
                .take(4)
                .collect();
        }

        suggestions.push("Custom...".to_string());

        let display_items: Vec<&str> = suggestions.iter().map(|s| s.as_str()).collect();
        let selection = Select::new()
            .with_prompt("Select event type")
            .items(&display_items)
            .interact()?;

        let event_type_name = if selection == suggestions.len() - 1 {
            Input::<String>::new()
                .with_prompt("Event type (format: service.resource.verb)")
                .interact_text()?
        } else {
            suggestions[selection].clone()
        };

        let (service, resource, verb) = crate::api::models::EventType::parse(&event_type_name)
            .ok_or_else(|| anyhow!("Invalid event type format"))?;

        let event_type_post = EventTypePost {
            application_id: app.application_id,
            service,
            resource_type: resource,
            verb,
        };

        client.create_event_type(&event_type_post).await?;
        output_success(&format!("Event type '{}' created!", event_type_name));
    }

    // Step 6: Optional - Set up local listener
    let setup_listener = Confirm::new()
        .with_prompt("Set up local webhook listening?")
        .default(true)
        .interact()?;

    if setup_listener {
        let port: u16 = Input::new()
            .with_prompt("Local port")
            .default(3000)
            .interact_text()?;

        println!("\nTo start receiving webhooks locally, run:");
        println!("  hook0 listen {}", port);
    }

    println!("\nSetup complete! Here are some useful commands:");
    println!("  hook0 event send user.account.created --payload '{{\"user_id\": 123}}'");
    println!("  hook0 event-type list");
    println!("  hook0 subscription list");
    println!("  hook0 listen 3000");

    Ok(())
}

/// Quick start - minimal setup and start listening
pub async fn quickstart(cli: &Cli, args: &QuickstartArgs) -> Result<()> {
    // Check if already configured
    let config = Config::load().ok();

    let (client, _profile_name, profile) = if config.is_some() {
        // Try to use existing config
        match crate::commands::require_auth(cli) {
            Ok(result) => result,
            Err(_) => {
                output_warning("No valid credentials found. Running setup wizard...");
                init(cli, &InitArgs { non_interactive: false }).await?;
                crate::commands::require_auth(cli)?
            }
        }
    } else {
        output_info("No configuration found. Let's set up quickly!");

        let secret: String = Password::new()
            .with_prompt("Application Secret")
            .interact()?;

        let api_url = "https://app.hook0.com/api/v1";
        let client = ApiClient::new(api_url, &secret);

        // Get first application
        let orgs = client.list_organizations().await?;
        let org = orgs.first().ok_or_else(|| anyhow!("No organizations found"))?;
        let apps = client.list_applications(&org.organization_id).await?;
        let app = apps.first().ok_or_else(|| anyhow!("No applications found"))?;

        // Save config
        let mut config = Config::default();
        let profile = Profile::with_details(
            api_url.to_string(),
            app.application_id,
            Some(app.organization_id),
            None,
        );
        config.set_profile("default", profile.clone());
        config.save()?;
        Config::store_secret("default", &app.application_id, &secret)?;

        output_success(&format!("Configured with application: {}", app.name));

        (client, "default".to_string(), profile)
    };

    // Create event type if specified
    if let Some(event_type_name) = &args.event_type {
        let (service, resource, verb) = crate::api::models::EventType::parse(event_type_name)
            .ok_or_else(|| anyhow!("Invalid event type format"))?;

        let event_type_post = EventTypePost {
            application_id: profile.application_id,
            service,
            resource_type: resource,
            verb,
        };

        match client.create_event_type(&event_type_post).await {
            Ok(_) => output_success(&format!("Event type '{}' created!", event_type_name)),
            Err(e) => output_warning(&format!("Could not create event type: {}", e)),
        }
    }

    // Start listener
    println!("\nStarting local listener on port {}...", args.port);

    let listen_args = crate::commands::listen::ListenArgs {
        target: format!("http://localhost:{}", args.port),
        events: Vec::new(),
        label: Vec::new(),
        mode: crate::commands::listen::ListenMode::Interactive,
        since: None,
        insecure: false,
    };

    crate::commands::listen::execute(cli, &listen_args).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path() {
        let path = config_file_path();
        assert!(path.to_string_lossy().contains("hook0"));
    }
}
