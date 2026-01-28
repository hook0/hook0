use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use clap::{Args, Subcommand};
use uuid::Uuid;

use crate::api::models::{base64_decode, EventFilters, EventPost, PaginationParams};
use crate::commands::require_auth;
use crate::output::{output_many, output_one, output_success, OutputFormat, TableOutput};
use crate::Cli;

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

    /// Labels in key=value format (can be repeated)
    #[arg(long, short = 'l', value_parser = parse_label)]
    pub label: Vec<(String, String)>,

    /// Custom event ID (UUID, auto-generated if not provided)
    #[arg(long)]
    pub event_id: Option<Uuid>,

    /// Content type (default: application/json)
    #[arg(long, default_value = "application/json")]
    pub content_type: String,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter by event type
    #[arg(long)]
    pub event_type: Option<String>,

    /// Filter by status (waiting, pending, in_progress, successful, failed)
    #[arg(long)]
    pub status: Option<String>,

    /// Filter events since (e.g., 1h, 24h, 7d)
    #[arg(long)]
    pub since: Option<String>,

    /// Filter events until
    #[arg(long)]
    pub until: Option<String>,

    /// Filter by label (key=value, can be repeated)
    #[arg(long, short = 'l', value_parser = parse_label)]
    pub label: Vec<(String, String)>,

    /// Maximum number of events to return
    #[arg(long, default_value = "50")]
    pub limit: i32,

    /// Page number
    #[arg(long, default_value = "1")]
    pub page: i32,
}

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

/// Parse a duration string (e.g., 1h, 24h, 7d)
fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return Err(anyhow!("Empty duration string"));
    }

    let (num_str, unit) = s.split_at(s.len() - 1);
    let num: i64 = num_str
        .parse()
        .map_err(|_| anyhow!("Invalid duration number: {}", num_str))?;

    match unit {
        "s" => Ok(Duration::seconds(num)),
        "m" => Ok(Duration::minutes(num)),
        "h" => Ok(Duration::hours(num)),
        "d" => Ok(Duration::days(num)),
        "w" => Ok(Duration::weeks(num)),
        _ => Err(anyhow!(
            "Invalid duration unit '{}'. Use s, m, h, d, or w",
            unit
        )),
    }
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

    // Parse and validate JSON if content type is JSON
    let payload_value: serde_json::Value = if args.content_type.contains("json") {
        serde_json::from_str(&payload_str)?
    } else {
        serde_json::Value::String(payload_str.clone())
    };

    // Build labels
    let labels: HashMap<String, String> = args.label.iter().cloned().collect();

    // Create event
    let event = if args.content_type.contains("json") {
        EventPost::new_json(
            profile.application_id,
            args.event_type.clone(),
            payload_value,
            labels,
        )
    } else {
        EventPost::new_text(
            profile.application_id,
            args.event_type.clone(),
            payload_str,
            labels,
        )
    };

    // Override event ID if provided
    let event = if let Some(id) = args.event_id {
        EventPost {
            event_id: id,
            ..event
        }
    } else {
        event
    };

    let result = client.send_event(&event).await?;

    if cli.output == OutputFormat::Json {
        output_one(&result, cli.output);
    } else {
        output_success(&format!(
            "Event sent successfully!\n  Event ID: {}\n  Type: {}",
            result.event_id, result.event_type_name
        ));
    }

    Ok(())
}

async fn list(cli: &Cli, args: &ListArgs) -> Result<()> {
    let (client, _, profile) = require_auth(cli)?;

    // Build filters
    let mut filters = EventFilters {
        event_type: args.event_type.clone(),
        status: args.status.clone(),
        since: None,
        until: None,
        labels: args.label.iter().cloned().collect(),
    };

    // Parse since/until
    if let Some(since_str) = &args.since {
        let duration = parse_duration(since_str)?;
        filters.since = Some(Utc::now() - duration);
    }

    if let Some(until_str) = &args.until {
        let duration = parse_duration(until_str)?;
        filters.until = Some(Utc::now() - duration);
    }

    let pagination = PaginationParams::new(Some(args.page), Some(args.limit));

    let events = client
        .list_events(&profile.application_id, &filters, &pagination)
        .await?;

    output_many(&events, cli.output);

    Ok(())
}

async fn get(cli: &Cli, args: &GetArgs) -> Result<()> {
    let (client, _, profile) = require_auth(cli)?;

    let event = client.get_event(&args.event_id).await?;

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
            let payload_decoded = base64_decode(&event.payload).unwrap_or_else(|_| event.payload.clone());

            TableOutput::print_details(vec![
                ("Event ID", event.event_id.to_string()),
                ("Type", event.event_type_name.clone()),
                ("Content Type", event.payload_content_type.clone()),
                ("Occurred At", event.occurred_at.to_rfc3339()),
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
    } else {
        if cli.output == OutputFormat::Json {
            output_one(&event, cli.output);
        } else {
            let payload_decoded = base64_decode(&event.payload).unwrap_or_else(|_| event.payload.clone());

            TableOutput::print_details(vec![
                ("Event ID", event.event_id.to_string()),
                ("Type", event.event_type_name.clone()),
                ("Content Type", event.payload_content_type.clone()),
                ("Occurred At", event.occurred_at.to_rfc3339()),
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

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("1h").expect("should parse").num_hours(), 1);
        assert_eq!(parse_duration("24h").expect("should parse").num_hours(), 24);
        assert_eq!(parse_duration("7d").expect("should parse").num_days(), 7);
        assert_eq!(parse_duration("1w").expect("should parse").num_weeks(), 1);
        assert_eq!(parse_duration("30m").expect("should parse").num_minutes(), 30);
        assert_eq!(parse_duration("60s").expect("should parse").num_seconds(), 60);
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("invalid").is_err());
        assert!(parse_duration("1x").is_err());
        assert!(parse_duration("").is_err());
    }
}
