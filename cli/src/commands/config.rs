use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use console::style;
use dialoguer::{Confirm, Input, Password, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::api::models::EventTypePost;
use crate::api::ApiClient;
use crate::config::{config_file_path, Config, Profile};
use crate::output::{output_success, output_warning, OutputFormat, TableOutput};
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

    /// Event type to create
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
        println!(
            "{}",
            serde_json::to_string_pretty(&profiles).expect("json serialization")
        );
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
        println!(
            "{}",
            serde_json::to_string_pretty(&json).expect("json serialization")
        );
    } else {
        println!("Configuration file: {}", config_file_path().display());
        println!(
            "Default profile: {}",
            config.default_profile.as_deref().unwrap_or("(none)")
        );
        println!("Profiles: {}", config.profiles.len());

        for (name, profile) in &config.profiles {
            let is_default = config.default_profile.as_deref() == Some(name.as_str());
            println!("\n[{}]{}", name, if is_default { " (default)" } else { "" });
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

/// Print a numbered step header
fn step(current: u8, total: u8, label: &str) {
    println!(
        "  {} {}",
        style(format!("[{current}/{total}]")).dim(),
        style(label).bold()
    );
}

/// Print a completed step with a value
fn step_done(label: &str, value: &str) {
    println!("  {} {} {}", style("●").green(), style(label).dim(), value);
}

/// Create a spinner with a message
fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("  {spinner:.cyan} {msg}")
            .expect("valid template")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "●"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Interactive initialization wizard
pub async fn init(_cli: &Cli, args: &InitArgs) -> Result<()> {
    if args.non_interactive {
        return Err(anyhow!(
            "Non-interactive init requires --secret and --api-url. Use 'hook0 login' instead."
        ));
    }

    let total_steps = if args.event_type.is_some() { 5 } else { 4 };

    // Banner
    println!();
    println!("  {}", style("Hook0 — Setup Wizard").bold().cyan());
    println!("  {}", style("─".repeat(40)).dim());
    println!(
        "  {}",
        style("Let's connect your application to Hook0.").dim()
    );
    println!();

    // ── Step 1: API URL ──────────────────────────────────────────────────
    step(1, total_steps, "API Endpoint");
    println!();
    let api_url: String = Input::new()
        .with_prompt("  API URL")
        .default("https://app.hook0.com/api/v1".to_string())
        .interact_text()?;
    step_done("API", &api_url);
    println!();

    // ── Step 2: Secret ───────────────────────────────────────────────────
    step(2, total_steps, "Authentication");
    println!(
        "  {}",
        style("Find your secret in Dashboard > Application > Settings > Secrets").dim()
    );
    println!();
    let secret: String = Password::new()
        .with_prompt("  Application Secret")
        .interact()?;
    step_done("Secret", &style("••••••••").dim().to_string());
    println!();

    // ── Step 3: Validate credentials ─────────────────────────────────────
    step(3, total_steps, "Connect & Verify");

    let sp = spinner("Validating credentials...");
    let client = ApiClient::new(&api_url, &secret);

    let orgs = client.list_organizations().await.inspect_err(|_| {
        sp.finish_with_message(format!("{} {}", style("✗").red(), style("Failed").red()));
    })?;
    if orgs.is_empty() {
        sp.finish_with_message(format!(
            "{} {}",
            style("✗").red(),
            style("No organizations found").red()
        ));
        return Err(anyhow!("No organizations found for this secret."));
    }

    sp.set_message("Fetching applications...");

    let mut apps = Vec::new();
    for org in &orgs {
        let org_apps = client
            .list_applications(&org.organization_id)
            .await
            .inspect_err(|_| {
                sp.finish_with_message(format!("{} {}", style("✗").red(), style("Failed").red()));
            })?;
        apps.extend(org_apps);
    }

    if apps.is_empty() {
        sp.finish_with_message(format!(
            "{} {}",
            style("✗").red(),
            style("No applications found").red()
        ));
        return Err(anyhow!("No applications found."));
    }

    sp.finish_and_clear();

    // Select application
    let app = if apps.len() == 1 {
        let app = apps.remove(0);
        step_done("Application", &style(&app.name).cyan().to_string());
        app
    } else {
        let app_names: Vec<String> = apps.iter().map(|a| a.name.clone()).collect();
        let selection = Select::new()
            .with_prompt("  Select application")
            .items(&app_names)
            .interact()?;
        let app = apps.remove(selection);
        step_done("Application", &style(&app.name).cyan().to_string());
        app
    };

    step_done(
        "Organization",
        &format!(
            "{} ({})",
            orgs.first().map(|o| o.name.as_str()).unwrap_or("—"),
            style(app.organization_id).dim()
        ),
    );
    println!();

    // ── Step 4: Save profile ─────────────────────────────────────────────
    step(4, total_steps, "Profile");
    println!();
    let profile_name: String = Input::new()
        .with_prompt("  Profile name")
        .default("default".to_string())
        .interact_text()?;

    let sp = spinner("Saving configuration...");

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

    sp.finish_and_clear();
    step_done("Profile", &format!("{} saved", style(&profile_name).cyan()));
    println!();

    // ── Step 5 (optional): Event type ────────────────────────────────────
    if let Some(ref event_type_name) = args.event_type {
        step(5, total_steps, "Event Type");

        let sp = spinner(&format!("Creating event type '{}'...", event_type_name));

        let (service, resource, verb) = crate::api::models::EventType::parse(event_type_name)
            .ok_or_else(|| anyhow!("Invalid event type format"))?;

        let event_type_post = EventTypePost {
            application_id: app.application_id,
            service,
            resource_type: resource,
            verb,
        };

        client.create_event_type(&event_type_post).await?;
        sp.finish_and_clear();
        step_done(
            "Event type",
            &format!("{} created", style(event_type_name).cyan()),
        );
        println!();
    } else if !args.non_interactive {
        let create_event_type = Confirm::new()
            .with_prompt(format!(
                "  {} Create your first event type?",
                style("→").dim()
            ))
            .default(true)
            .interact()?;

        if create_event_type {
            let sp = spinner("Fetching existing event types...");
            let existing_types = client
                .list_event_types(&app.application_id)
                .await
                .unwrap_or_default();
            sp.finish_and_clear();

            let mut suggestions: Vec<String> =
                existing_types.iter().map(|et| et.full_name()).collect();

            if suggestions.is_empty() {
                let common_services = ["user", "order", "invoice", "payment"];
                let common_verbs = ["created", "updated", "deleted"];
                suggestions = common_services
                    .iter()
                    .flat_map(|s| {
                        common_verbs
                            .iter()
                            .map(move |v| format!("{}.account.{}", s, v))
                    })
                    .take(4)
                    .collect();
            }

            suggestions.push("Custom...".to_string());

            let display_items: Vec<&str> = suggestions.iter().map(|s| s.as_str()).collect();
            let selection = Select::new()
                .with_prompt("  Select event type")
                .items(&display_items)
                .interact()?;

            let event_type_name = if selection == suggestions.len() - 1 {
                Input::<String>::new()
                    .with_prompt("  Event type (format: service.resource.verb)")
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

            let sp = spinner(&format!("Creating '{}'...", event_type_name));
            client.create_event_type(&event_type_post).await?;
            sp.finish_and_clear();
            step_done(
                "Event type",
                &format!("{} created", style(&event_type_name).cyan()),
            );
            println!();
        }
    }

    // ── Done ─────────────────────────────────────────────────────────────
    println!("  {}", style("─".repeat(40)).dim());
    println!(
        "  {} {}",
        style("●").green().bold(),
        style("Setup complete!").green().bold()
    );
    println!("  {}", style("─".repeat(40)).dim());
    println!();
    println!("  {} Next steps:", style("→").cyan());
    println!();
    println!(
        "    {}  Forward webhooks to localhost",
        style("hook0 listen").cyan()
    );
    println!(
        "    {}  Send a test webhook",
        style("hook0 example http://localhost:3000").cyan()
    );
    println!(
        "    {}  List your event types",
        style("hook0 event-type list").cyan()
    );
    println!();

    Ok(())
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
