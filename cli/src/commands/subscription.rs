use std::collections::HashMap;

use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use uuid::Uuid;

use crate::api::models::{SubscriptionPost, SubscriptionPut, Target};
use crate::commands::require_auth;
use crate::output::{output_many, output_one, output_success, OutputFormat, TableOutput};
use crate::Cli;

#[derive(Subcommand, Debug)]
pub enum SubscriptionCommands {
    /// Create a new subscription
    Create(CreateArgs),

    /// List subscriptions
    List(ListArgs),

    /// Get subscription details
    Get(GetArgs),

    /// Update a subscription
    Update(UpdateArgs),

    /// Delete a subscription
    Delete(DeleteArgs),

    /// Enable a subscription
    Enable(EnableArgs),

    /// Disable a subscription
    Disable(DisableArgs),
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Webhook endpoint URL
    #[arg(long, short = 'u')]
    pub url: String,

    /// Event types to subscribe to (comma-separated or repeated)
    #[arg(long, short = 'e', value_delimiter = ',')]
    pub events: Vec<String>,

    /// Labels in key=value format (can be repeated)
    #[arg(long, short = 'l', value_parser = parse_label)]
    pub label: Vec<(String, String)>,

    /// HTTP method (default: POST)
    #[arg(long, default_value = "POST")]
    pub method: String,

    /// Custom headers in key=value format (can be repeated)
    #[arg(long, short = 'H', value_parser = parse_label)]
    pub header: Vec<(String, String)>,

    /// Description
    #[arg(long, short = 'd')]
    pub description: Option<String>,

    /// Create disabled
    #[arg(long)]
    pub disabled: bool,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter by label (key=value, can be repeated)
    #[arg(long, short = 'l', value_parser = parse_label)]
    pub label: Vec<(String, String)>,

    /// Show only enabled subscriptions
    #[arg(long)]
    pub enabled: bool,

    /// Show only disabled subscriptions
    #[arg(long)]
    pub disabled: bool,
}

#[derive(Args, Debug)]
pub struct GetArgs {
    /// Subscription ID
    pub subscription_id: Uuid,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// Subscription ID
    pub subscription_id: Uuid,

    /// Webhook endpoint URL
    #[arg(long, short = 'u')]
    pub url: Option<String>,

    /// Event types to subscribe to (replaces existing)
    #[arg(long, short = 'e', value_delimiter = ',')]
    pub events: Option<Vec<String>>,

    /// Labels in key=value format (replaces existing)
    #[arg(long, short = 'l', value_parser = parse_label)]
    pub label: Vec<(String, String)>,

    /// HTTP method
    #[arg(long)]
    pub method: Option<String>,

    /// Custom headers (replaces existing)
    #[arg(long, short = 'H', value_parser = parse_label)]
    pub header: Vec<(String, String)>,

    /// Description
    #[arg(long, short = 'd')]
    pub description: Option<String>,

    /// Enable the subscription
    #[arg(long)]
    pub enable: bool,

    /// Disable the subscription
    #[arg(long)]
    pub disable: bool,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// Subscription ID
    pub subscription_id: Uuid,

    /// Skip confirmation prompt
    #[arg(long, short = 'y')]
    pub yes: bool,
}

#[derive(Args, Debug)]
pub struct EnableArgs {
    /// Subscription ID
    pub subscription_id: Uuid,
}

#[derive(Args, Debug)]
pub struct DisableArgs {
    /// Subscription ID
    pub subscription_id: Uuid,
}

/// Parse a label in key=value format
fn parse_label(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err("Must be in key=value format".to_string());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

pub async fn execute(cli: &Cli, cmd: &SubscriptionCommands) -> Result<()> {
    match cmd {
        SubscriptionCommands::Create(args) => create(cli, args).await,
        SubscriptionCommands::List(args) => list(cli, args).await,
        SubscriptionCommands::Get(args) => get(cli, args).await,
        SubscriptionCommands::Update(args) => update(cli, args).await,
        SubscriptionCommands::Delete(args) => delete(cli, args).await,
        SubscriptionCommands::Enable(args) => enable(cli, args).await,
        SubscriptionCommands::Disable(args) => disable(cli, args).await,
    }
}

async fn create(cli: &Cli, args: &CreateArgs) -> Result<()> {
    let (client, _, profile) = require_auth(cli)?;

    if args.events.is_empty() {
        return Err(anyhow!("At least one event type is required (--events)"));
    }

    // Validate URL
    url::Url::parse(&args.url).map_err(|e| anyhow!("Invalid URL: {}", e))?;

    let headers: HashMap<String, String> = args.header.iter().cloned().collect();
    let labels: HashMap<String, String> = args.label.iter().cloned().collect();

    let subscription = SubscriptionPost {
        application_id: profile.application_id,
        event_types: args.events.clone(),
        is_enabled: !args.disabled,
        description: args.description.clone(),
        labels: if labels.is_empty() { None } else { Some(labels) },
        metadata: None,
        target: Target::http_with_headers(args.url.clone(), args.method.clone(), headers),
        dedicated_workers: None,
    };

    let result = client.create_subscription(&subscription).await?;

    if cli.output == OutputFormat::Json {
        output_one(&result, cli.output);
    } else {
        output_success(&format!(
            "Subscription created successfully!\n  ID: {}\n  Status: {}",
            result.subscription_id,
            if result.is_enabled { "Enabled" } else { "Disabled" }
        ));
    }

    Ok(())
}

async fn list(cli: &Cli, args: &ListArgs) -> Result<()> {
    let (client, _, profile) = require_auth(cli)?;

    let labels: HashMap<String, String> = args.label.iter().cloned().collect();
    let mut subscriptions = client
        .list_subscriptions(&profile.application_id, &labels)
        .await?;

    // Filter by enabled/disabled
    if args.enabled {
        subscriptions.retain(|s| s.is_enabled);
    }
    if args.disabled {
        subscriptions.retain(|s| !s.is_enabled);
    }

    output_many(&subscriptions, cli.output);

    Ok(())
}

async fn get(cli: &Cli, args: &GetArgs) -> Result<()> {
    let (client, _, _) = require_auth(cli)?;

    let subscription = client.get_subscription(&args.subscription_id).await?;

    if cli.output == OutputFormat::Json {
        output_one(&subscription, cli.output);
    } else {
        let target_info = match &subscription.target {
            Target::Http { method, url, headers } => {
                let header_str = headers
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n    ");
                format!(
                    "{} {}\n  Headers:\n    {}",
                    method,
                    url,
                    if header_str.is_empty() { "(none)" } else { &header_str }
                )
            }
        };

        let labels_str = subscription
            .labels
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(", ");

        TableOutput::print_details(vec![
            ("Subscription ID", subscription.subscription_id.to_string()),
            ("Enabled", if subscription.is_enabled { "Yes" } else { "No" }.to_string()),
            ("Description", subscription.description.clone().unwrap_or_else(|| "-".to_string())),
            ("Event Types", subscription.event_types.join(", ")),
            ("Target", target_info),
            ("Labels", if labels_str.is_empty() { "-".to_string() } else { labels_str }),
            ("Secret", subscription.secret.to_string()),
            ("Created At", subscription.created_at.to_rfc3339()),
        ]);
    }

    Ok(())
}

async fn update(cli: &Cli, args: &UpdateArgs) -> Result<()> {
    let (client, _, _) = require_auth(cli)?;

    // Get current subscription
    let current = client.get_subscription(&args.subscription_id).await?;

    // Determine new enabled state
    let is_enabled = if args.enable {
        true
    } else if args.disable {
        false
    } else {
        current.is_enabled
    };

    // Build updated target
    let (current_method, current_url, current_headers) = match current.target {
        Target::Http { method, url, headers } => (method, url, headers),
    };

    let new_url = args.url.clone().unwrap_or(current_url);
    let new_method = args.method.clone().unwrap_or(current_method);
    let new_headers: HashMap<String, String> = if args.header.is_empty() {
        current_headers
    } else {
        args.header.iter().cloned().collect()
    };

    // Validate URL
    url::Url::parse(&new_url).map_err(|e| anyhow!("Invalid URL: {}", e))?;

    let new_labels: HashMap<String, String> = if args.label.is_empty() {
        current.labels
    } else {
        args.label.iter().cloned().collect()
    };

    let update = SubscriptionPut {
        event_types: args.events.clone().unwrap_or(current.event_types),
        is_enabled,
        description: args.description.clone().or(current.description),
        labels: Some(new_labels),
        metadata: Some(current.metadata),
        target: Target::http_with_headers(new_url, new_method, new_headers),
        dedicated_workers: Some(current.dedicated_workers),
    };

    let result = client.update_subscription(&args.subscription_id, &update).await?;

    if cli.output == OutputFormat::Json {
        output_one(&result, cli.output);
    } else {
        output_success(&format!(
            "Subscription {} updated successfully!",
            result.subscription_id
        ));
    }

    Ok(())
}

async fn delete(cli: &Cli, args: &DeleteArgs) -> Result<()> {
    let (client, _, _) = require_auth(cli)?;

    // Confirm deletion
    if !args.yes {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!("Delete subscription '{}'?", args.subscription_id))
            .default(false)
            .interact()?;

        if !confirmed {
            println!("Cancelled.");
            return Ok(());
        }
    }

    client.delete_subscription(&args.subscription_id).await?;

    output_success(&format!(
        "Subscription {} deleted successfully!",
        args.subscription_id
    ));

    Ok(())
}

async fn enable(cli: &Cli, args: &EnableArgs) -> Result<()> {
    let (client, _, _) = require_auth(cli)?;

    let result = client.enable_subscription(&args.subscription_id).await?;

    if cli.output == OutputFormat::Json {
        output_one(&result, cli.output);
    } else {
        output_success(&format!(
            "Subscription {} enabled successfully!",
            result.subscription_id
        ));
    }

    Ok(())
}

async fn disable(cli: &Cli, args: &DisableArgs) -> Result<()> {
    let (client, _, _) = require_auth(cli)?;

    let result = client.disable_subscription(&args.subscription_id).await?;

    if cli.output == OutputFormat::Json {
        output_one(&result, cli.output);
    } else {
        output_success(&format!(
            "Subscription {} disabled successfully!",
            result.subscription_id
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_label() {
        let result = parse_label("Content-Type=application/json");
        assert!(result.is_ok());
        let (k, v) = result.expect("should parse");
        assert_eq!(k, "Content-Type");
        assert_eq!(v, "application/json");
    }

    #[test]
    fn test_parse_label_invalid() {
        assert!(parse_label("no-equals").is_err());
    }

    #[test]
    fn test_url_validation() {
        assert!(url::Url::parse("https://example.com/webhook").is_ok());
        assert!(url::Url::parse("http://localhost:3000/webhook").is_ok());
        assert!(url::Url::parse("not-a-url").is_err());
    }
}
