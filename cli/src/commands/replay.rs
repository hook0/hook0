use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use clap::Args;
use uuid::Uuid;

use crate::api::models::{EventFilters, PaginationParams};
use crate::commands::require_auth;
use crate::output::{output_many, output_success, output_warning, OutputFormat};
use crate::Cli;

#[derive(Args, Debug)]
pub struct ReplayArgs {
    /// Event ID to replay
    pub event_id: Option<Uuid>,

    /// Replay all events matching criteria (requires --confirm)
    #[arg(long)]
    pub all: bool,

    /// Filter by status (failed, successful, etc.)
    #[arg(long)]
    pub status: Option<String>,

    /// Filter events since (e.g., 1h, 24h, 7d)
    #[arg(long)]
    pub since: Option<String>,

    /// Filter events until (e.g., 1h, 24h, 7d)
    #[arg(long)]
    pub until: Option<String>,

    /// Filter by event type
    #[arg(long)]
    pub event_type: Option<String>,

    /// Dry run - show what would be replayed without actually replaying
    #[arg(long)]
    pub dry_run: bool,

    /// Confirm bulk replay operation
    #[arg(long)]
    pub confirm: bool,

    /// Maximum number of events to replay
    #[arg(long, default_value = "100")]
    pub limit: i32,
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

pub async fn execute(cli: &Cli, args: &ReplayArgs) -> Result<()> {
    let (client, _, profile) = require_auth(cli)?;

    // Single event replay
    if let Some(event_id) = args.event_id {
        if args.dry_run {
            println!("Would replay event: {}", event_id);
            return Ok(());
        }

        let attempts = client.replay_event(&event_id).await?;

        if cli.output == OutputFormat::Json {
            output_many(&attempts, cli.output);
        } else {
            output_success(&format!(
                "Event {} replayed successfully!\n  {} new request attempt(s) created",
                event_id,
                attempts.len()
            ));
        }

        return Ok(());
    }

    // Bulk replay
    if !args.all {
        return Err(anyhow!(
            "Specify either an event ID or use --all for bulk replay"
        ));
    }

    if !args.confirm && !args.dry_run {
        return Err(anyhow!(
            "Bulk replay requires --confirm flag or --dry-run to preview"
        ));
    }

    // Build filters
    let mut filters = EventFilters {
        event_type: args.event_type.clone(),
        status: args.status.clone(),
        since: None,
        until: None,
        labels: std::collections::HashMap::new(),
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

    let pagination = PaginationParams::new(Some(1), Some(args.limit));

    // Get events matching criteria
    let events = client
        .list_events(&profile.application_id, &filters, &pagination)
        .await?;

    if events.is_empty() {
        output_warning("No events found matching the specified criteria.");
        return Ok(());
    }

    if args.dry_run {
        println!("Would replay {} event(s):", events.len());
        for event in &events {
            println!("  - {} ({})", event.event_id, event.event_type_name);
        }
        return Ok(());
    }

    // Confirm and replay
    println!("Replaying {} event(s)...", events.len());

    let mut total_attempts = 0;
    let mut failed = 0;

    for event in &events {
        match client.replay_event(&event.event_id).await {
            Ok(attempts) => {
                total_attempts += attempts.len();
                if cli.output != OutputFormat::Json {
                    println!("  Replayed {} ({} attempts)", event.event_id, attempts.len());
                }
            }
            Err(e) => {
                failed += 1;
                if cli.output != OutputFormat::Json {
                    eprintln!("  Failed to replay {}: {}", event.event_id, e);
                }
            }
        }
    }

    if cli.output == OutputFormat::Json {
        println!(
            "{}",
            serde_json::json!({
                "replayed": events.len() - failed,
                "failed": failed,
                "total_attempts": total_attempts,
            })
        );
    } else {
        output_success(&format!(
            "Replay complete!\n  Events: {} replayed, {} failed\n  Total request attempts: {}",
            events.len() - failed,
            failed,
            total_attempts
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("1h").expect("should parse").num_hours(), 1);
        assert_eq!(parse_duration("24h").expect("should parse").num_hours(), 24);
        assert_eq!(parse_duration("7d").expect("should parse").num_days(), 7);
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("invalid").is_err());
        assert!(parse_duration("").is_err());
    }
}
