use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};

use crate::api::models::{EventType, EventTypePost};
use crate::commands::require_auth;
use crate::output::{output_many, output_one, output_success};
use crate::Cli;

#[derive(Subcommand, Debug)]
pub enum EventTypeCommands {
    /// Create a new event type
    Create(CreateArgs),

    /// List event types
    List(ListArgs),

    /// Delete an event type
    Delete(DeleteArgs),
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Event type name (e.g., user.account.created) or individual components
    pub name: Option<String>,

    /// Service name (alternative to full name)
    #[arg(long, short = 's')]
    pub service: Option<String>,

    /// Resource type name (alternative to full name)
    #[arg(long, short = 'r')]
    pub resource: Option<String>,

    /// Verb name (alternative to full name)
    #[arg(long, short = 'b')]
    pub verb: Option<String>,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter by service name
    #[arg(long)]
    pub service: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// Event type name (e.g., user.account.created)
    pub name: String,

    /// Skip confirmation prompt
    #[arg(long, short = 'y')]
    pub yes: bool,
}

pub async fn execute(cli: &Cli, cmd: &EventTypeCommands) -> Result<()> {
    match cmd {
        EventTypeCommands::Create(args) => create(cli, args).await,
        EventTypeCommands::List(args) => list(cli, args).await,
        EventTypeCommands::Delete(args) => delete(cli, args).await,
    }
}

async fn create(cli: &Cli, args: &CreateArgs) -> Result<()> {
    let (client, _, profile) = require_auth(cli)?;

    // Determine service, resource, verb from arguments
    let (service, resource, verb) = match (&args.name, &args.service, &args.resource, &args.verb) {
        // Full name provided
        (Some(name), None, None, None) => {
            let parsed = EventType::parse(name).ok_or_else(|| {
                anyhow!(
                    "Invalid event type format. Expected 'service.resource.verb' (e.g., user.account.created)"
                )
            })?;
            parsed
        }
        // Individual components provided
        (None, Some(s), Some(r), Some(v)) => (s.clone(), r.clone(), v.clone()),
        // Mixed - error
        _ => {
            return Err(anyhow!(
                "Provide either a full event type name (user.account.created) or all three components (--service, --resource, --verb)"
            ));
        }
    };

    let event_type_post = EventTypePost {
        application_id: profile.application_id,
        service,
        resource_type: resource,
        verb,
    };

    let result = client.create_event_type(&event_type_post).await?;

    if cli.output == crate::output::OutputFormat::Json {
        output_one(&result, cli.output);
    } else {
        output_success(&format!("Event type '{}' created successfully!", result.full_name()));
    }

    Ok(())
}

async fn list(cli: &Cli, args: &ListArgs) -> Result<()> {
    let (client, _, profile) = require_auth(cli)?;

    let mut event_types = client.list_event_types(&profile.application_id).await?;

    // Filter by service if provided
    if let Some(service_filter) = &args.service {
        event_types.retain(|et| et.service_name == *service_filter);
    }

    output_many(&event_types, cli.output);

    Ok(())
}

async fn delete(cli: &Cli, args: &DeleteArgs) -> Result<()> {
    let (client, _, profile) = require_auth(cli)?;

    // Parse the event type name
    let (service, resource, verb) = EventType::parse(&args.name).ok_or_else(|| {
        anyhow!(
            "Invalid event type format. Expected 'service.resource.verb' (e.g., user.account.created)"
        )
    })?;

    // Confirm deletion
    if !args.yes {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!("Delete event type '{}'?", args.name))
            .default(false)
            .interact()?;

        if !confirmed {
            println!("Cancelled.");
            return Ok(());
        }
    }

    client
        .delete_event_type(&profile.application_id, &service, &resource, &verb)
        .await?;

    output_success(&format!("Event type '{}' deleted successfully!", args.name));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_parsing() {
        let parsed = EventType::parse("user.account.created");
        assert!(parsed.is_some());
        let (s, r, v) = parsed.expect("should parse");
        assert_eq!(s, "user");
        assert_eq!(r, "account");
        assert_eq!(v, "created");
    }

    #[test]
    fn test_event_type_parsing_invalid() {
        assert!(EventType::parse("invalid").is_none());
        assert!(EventType::parse("only.two").is_none());
        assert!(EventType::parse("too.many.parts.here").is_none()); // Requires exactly 3 parts
    }
}
