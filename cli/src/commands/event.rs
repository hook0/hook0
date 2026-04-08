use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use clap::{Args, Subcommand};
use uuid::Uuid;

use crate::Cli;
use crate::api::models::{EventFilters, EventPost, PaginationParams, base64_decode};
use crate::commands::require_auth;
use crate::output::{OutputFormat, TableOutput, output_many, output_one, output_success};

#[derive(Subcommand, Debug)]
pub enum EventCommands {
    /// Send a new event
    Send(SendArgs),

    /// List events
    List(ListArgs),

    /// Get event details
    Get(GetArgs),
}

#[derive(Args, Debug)]
pub struct SendArgs {
    /// Event type (e.g., user.account.created)
    pub event_type: String,

    /// JSON payload
    #[arg(long, short = 'd')]
    pub payload: Option<String>,

    /// Read payload from file
    #[arg(long, short = 'f')]
    pub payload_file: Option<PathBuf>,

    /// Labels in key=value format (required, can be repeated)
    #[arg(long, short = 'l', required = true, value_parser = parse_label)]
    pub label: Vec<(String, String)>,

    /// Custom event ID (UUID, auto-generated if not provided)
    #[arg(long)]
    pub event_id: Option<Uuid>,

    /// Content type (default: application/json)
    #[arg(long, default_value = crate::api::models::CONTENT_TYPE_JSON)]
    pub content_type: String,
}

#[derive(Args, Debug)]
pub struct ListArgs {}

#[derive(Args, Debug)]
pub struct GetArgs {
    /// Event ID
    pub event_id: Uuid,

    /// Show request attempts for this event
    #[arg(long)]
    pub attempts: bool,
}

/// Parse a label in key=value format
fn parse_label(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err("Label must be in key=value format".to_string());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

pub async fn execute(cli: &Cli, cmd: &EventCommands) -> Result<()> {
    match cmd {
        EventCommands::Send(args) => send(cli, args).await,
        EventCommands::List(args) => list(cli, args).await,
        EventCommands::Get(args) => get(cli, args).await,
    }
}

async fn send(cli: &Cli, args: &SendArgs) -> Result<()> {
    let (client, _, profile) = require_auth(cli)?;

    // Get payload
    let payload_str = match (&args.payload, &args.payload_file) {
        (Some(p), None) => p.clone(),
        (None, Some(f)) => fs::read_to_string(f)?,
        (Some(_), Some(_)) => {
            return Err(anyhow!("Cannot specify both --payload and --payload-file"));
        }
        (None, None) => "{}".to_string(),
    };

    let is_json = args.content_type.contains("json");
    let labels: HashMap<String, String> = args.label.iter().cloned().collect();

    let event = if is_json {
        let payload_value: serde_json::Value = serde_json::from_str(&payload_str)
            .map_err(|e| anyhow!("Invalid JSON payload: {}", e))?;
        EventPost::new_json(
            profile.application_id,
            args.event_type.clone(),
            payload_value,
            labels,
        )
    } else {
        EventPost::new_with_content_type(
            profile.application_id,
            args.event_type.clone(),
            payload_str,
            args.content_type.clone(),
            labels,
        )
    };

    // Override event ID if provided
    let event = EventPost {
        event_id: args.event_id,
        ..event
    };

    let result = client.send_event(&event).await?;

    if cli.output == OutputFormat::Json {
        output_one(&result, cli.output);
    } else {
        output_success(&format!(
            "Event sent successfully!\n  Event ID: {}\n  Type: {}",
            result.event_id,
            result
                .event_type_name
                .as_deref()
                .unwrap_or(&args.event_type)
        ));
    }

    Ok(())
}

async fn list(cli: &Cli, _args: &ListArgs) -> Result<()> {
    let (client, _, profile) = require_auth(cli)?;

    let filters = EventFilters::default();
    let pagination = PaginationParams::default();

    let events = client
        .list_events(&profile.application_id, &filters, &pagination)
        .await?;

    output_many(&events, cli.output);

    Ok(())
}

async fn get(cli: &Cli, args: &GetArgs) -> Result<()> {
    let (client, _, profile) = require_auth(cli)?;

    let event = client
        .get_event(&args.event_id, &profile.application_id)
        .await?;

    if args.attempts {
        // Show event and its request attempts
        let attempts = client
            .list_request_attempts(&profile.application_id, Some(&args.event_id))
            .await?;

        if cli.output == OutputFormat::Json {
            println!(
                "{}",
                serde_json::json!({
                    "event": event,
                    "attempts": attempts,
                })
            );
        } else {
            // Show event details
            let raw_payload = event.payload.clone().unwrap_or_default();
            let payload_decoded = base64_decode(&raw_payload).unwrap_or(raw_payload);

            TableOutput::print_details(vec![
                ("Event ID", event.event_id.to_string()),
                ("Type", event.event_type_name.clone().unwrap_or_default()),
                (
                    "Content Type",
                    event.payload_content_type.clone().unwrap_or_default(),
                ),
                (
                    "Occurred At",
                    event
                        .occurred_at
                        .map(|t| t.to_rfc3339())
                        .unwrap_or_default(),
                ),
                ("Received At", event.received_at.to_rfc3339()),
                (
                    "Labels",
                    event
                        .labels
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
            ]);

            println!("\nPayload:\n{}", payload_decoded);

            if !attempts.is_empty() {
                println!("\nRequest Attempts:");
                output_many(&attempts, cli.output);
            } else {
                println!("\nNo request attempts yet.");
            }
        }
    } else if cli.output == OutputFormat::Json {
        output_one(&event, cli.output);
    } else {
        let raw_payload = event.payload.clone().unwrap_or_default();
        let payload_decoded = base64_decode(&raw_payload).unwrap_or(raw_payload);

        TableOutput::print_details(vec![
            ("Event ID", event.event_id.to_string()),
            ("Type", event.event_type_name.clone().unwrap_or_default()),
            (
                "Content Type",
                event.payload_content_type.clone().unwrap_or_default(),
            ),
            (
                "Occurred At",
                event
                    .occurred_at
                    .map(|t| t.to_rfc3339())
                    .unwrap_or_default(),
            ),
            ("Received At", event.received_at.to_rfc3339()),
            (
                "Labels",
                event
                    .labels
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        ]);

        println!("\nPayload:\n{}", payload_decoded);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_label() {
        let result = parse_label("key=value");
        assert!(result.is_ok());
        let (k, v) = result.expect("should parse");
        assert_eq!(k, "key");
        assert_eq!(v, "value");
    }

    #[test]
    fn test_parse_label_with_equals_in_value() {
        let result = parse_label("key=value=with=equals");
        assert!(result.is_ok());
        let (k, v) = result.expect("should parse");
        assert_eq!(k, "key");
        assert_eq!(v, "value=with=equals");
    }

    #[test]
    fn test_parse_label_invalid() {
        let result = parse_label("invalid");
        assert!(result.is_err());
    }
}
