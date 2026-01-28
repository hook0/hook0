use std::sync::Arc;
use std::time::Instant;

use anyhow::{anyhow, Result};
use clap::{Args, ValueEnum};
use tokio::sync::mpsc;

use crate::api::ApiClient;
use crate::commands::require_auth;
use crate::output::{output_error, output_info, output_success, output_warning};
use crate::tui::TuiApp;
use crate::tunnel::{Forwarder, InspectedRequest, Inspector, StreamClient, StreamEvent};
use crate::Cli;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum ListenMode {
    /// Full-screen TUI with event list and navigation
    #[default]
    Interactive,
    /// One line per event (compact output)
    Compact,
    /// Minimal output, only errors
    Quiet,
    /// JSON streaming output
    Json,
}

#[derive(Args, Debug)]
pub struct ListenArgs {
    /// Target to forward webhooks to (port number or URL)
    /// Examples: 3000, http://localhost:3000/webhooks
    pub target: String,

    /// Filter by event types (can be repeated)
    #[arg(long, short = 'e')]
    pub events: Vec<String>,

    /// Filter by labels (key=value, can be repeated)
    #[arg(long, short = 'l', value_parser = parse_label)]
    pub label: Vec<(String, String)>,

    /// Output mode
    #[arg(long, short = 'm', value_enum, default_value = "interactive")]
    pub mode: ListenMode,

    /// Replay recent events from the last N hours
    #[arg(long)]
    pub since: Option<String>,

    /// Accept invalid SSL certificates from local server
    #[arg(long)]
    pub insecure: bool,
}

fn parse_label(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err("Must be in key=value format".to_string());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

pub async fn execute(cli: &Cli, args: &ListenArgs) -> Result<()> {
    let (client, _profile_name, profile) = require_auth(cli)?;

    // Parse target URL
    let target_url = crate::tunnel::forwarder::parse_target(&args.target)?;

    output_info(&format!("Forwarding webhooks to: {}", target_url));

    // Create forwarder
    let forwarder = Arc::new(Forwarder::new(target_url.clone(), args.insecure));

    // Create inspector for request history
    let inspector = Inspector::default_capacity();

    // Get stream URL
    let stream_url = client.get_stream_url(&profile.application_id);

    output_info(&format!("Connecting to stream server..."));

    // Connect to stream
    let mut stream_client = StreamClient::new(stream_url);
    let mut events = match stream_client.connect().await {
        Ok(rx) => rx,
        Err(e) => {
            // If streaming is not available, fall back to polling mode
            output_warning(&format!("Stream connection failed: {}. Falling back to polling mode.", e));
            return run_polling_mode(cli, &args, &client, &profile, &forwarder, &inspector).await;
        }
    };

    // Wait for connection
    let webhook_url = match events.recv().await {
        Some(StreamEvent::Connected { webhook_url, session_id }) => {
            output_success(&format!("Connected! Session: {}", session_id));
            webhook_url
        }
        Some(StreamEvent::Error(e)) => {
            return Err(anyhow!("Connection error: {}", e));
        }
        _ => {
            return Err(anyhow!("Unexpected connection state"));
        }
    };

    output_info(&format!("Webhook URL: {}", webhook_url));
    output_info("Paste this URL in your application settings to receive webhooks.");
    output_info("Press Ctrl+C to stop.\n");

    // Run based on mode
    match args.mode {
        ListenMode::Interactive => {
            run_interactive_mode(events, &forwarder, inspector, &webhook_url, &target_url).await
        }
        ListenMode::Compact => {
            run_compact_mode(events, &forwarder, inspector).await
        }
        ListenMode::Quiet => {
            run_quiet_mode(events, &forwarder, inspector).await
        }
        ListenMode::Json => {
            run_json_mode(events, &forwarder, inspector).await
        }
    }
}

async fn run_interactive_mode(
    mut events: mpsc::Receiver<StreamEvent>,
    forwarder: &Arc<Forwarder>,
    inspector: Inspector,
    webhook_url: &str,
    target_url: &str,
) -> Result<()> {
    // Create TUI app
    let mut app = TuiApp::new(inspector.clone(), webhook_url.to_string(), target_url.to_string())?;

    // Spawn task to receive stream events
    let inspector_clone = inspector.clone();
    let forwarder_clone = Arc::clone(forwarder);
    let (update_tx, mut update_rx) = mpsc::channel::<()>(100);

    tokio::spawn(async move {
        while let Some(event) = events.recv().await {
            match event {
                StreamEvent::WebhookReceived {
                    request_id,
                    event_id,
                    event_type,
                    payload,
                    headers,
                    received_at,
                } => {
                    // Add to inspector
                    let request = InspectedRequest::new(
                        request_id.clone(),
                        event_id,
                        event_type.clone(),
                        payload.clone(),
                        headers.clone(),
                        received_at,
                    );
                    inspector_clone.add(request);

                    // Notify TUI
                    let _ = update_tx.send(()).await;

                    // Forward the webhook
                    let result = forwarder_clone
                        .forward(&payload, &headers, &event_type)
                        .await;

                    // Update inspector with result
                    if let Ok(result) = result {
                        inspector_clone.update(&request_id, |r| {
                            r.update_from_result(&result);
                        });
                        let _ = update_tx.send(()).await;
                    }
                }
                StreamEvent::Disconnected => {
                    break;
                }
                StreamEvent::Error(e) => {
                    tracing::error!("Stream error: {}", e);
                }
                _ => {}
            }
        }
    });

    // Run TUI
    app.run(&mut update_rx).await
}

async fn run_compact_mode(
    mut events: mpsc::Receiver<StreamEvent>,
    forwarder: &Arc<Forwarder>,
    _inspector: Inspector,
) -> Result<()> {
    println!("{:<10} {:<30} {:<10} {:<10} {}", "TIME", "EVENT TYPE", "STATUS", "TIME(ms)", "EVENT ID");
    println!("{}", "-".repeat(80));

    while let Some(event) = events.recv().await {
        match event {
            StreamEvent::WebhookReceived {
                event_id,
                event_type,
                payload,
                headers,
                received_at,
                ..
            } => {
                let start = Instant::now();

                // Forward
                let result = forwarder.forward(&payload, &headers, &event_type).await;

                let (status, elapsed) = match result {
                    Ok(r) => {
                        if r.error.is_some() {
                            ("ERR".to_string(), r.elapsed_ms)
                        } else {
                            (format!("{}", r.status_code), r.elapsed_ms)
                        }
                    }
                    Err(_) => ("ERR".to_string(), start.elapsed().as_millis() as i64),
                };

                println!(
                    "{:<10} {:<30} {:<10} {:<10} {}",
                    received_at.format("%H:%M:%S"),
                    truncate(&event_type, 28),
                    status,
                    format!("{}ms", elapsed),
                    event_id
                );
            }
            StreamEvent::Disconnected => {
                println!("\nDisconnected from stream.");
                break;
            }
            StreamEvent::Error(e) => {
                eprintln!("Error: {}", e);
            }
            _ => {}
        }
    }

    Ok(())
}

async fn run_quiet_mode(
    mut events: mpsc::Receiver<StreamEvent>,
    forwarder: &Arc<Forwarder>,
    _inspector: Inspector,
) -> Result<()> {
    while let Some(event) = events.recv().await {
        match event {
            StreamEvent::WebhookReceived {
                
                event_id,
                event_type,
                payload,
                headers,
                ..
            } => {
                // Forward
                let result = forwarder.forward(&payload, &headers, &event_type).await;

                // Only print errors
                match result {
                    Ok(r) => {
                        if let Some(err) = &r.error {
                            output_error(&format!("[{}] {} - {}", event_type, event_id, err));
                        } else if r.status_code >= 400 {
                            output_error(&format!(
                                "[{}] {} - HTTP {}",
                                event_type, event_id, r.status_code
                            ));
                        }
                    }
                    Err(e) => {
                        output_error(&format!("[{}] {} - {}", event_type, event_id, e));
                    }
                }
            }
            StreamEvent::Disconnected => {
                output_warning("Disconnected from stream.");
                break;
            }
            StreamEvent::Error(e) => {
                output_error(&format!("Stream error: {}", e));
            }
            _ => {}
        }
    }

    Ok(())
}

async fn run_json_mode(
    mut events: mpsc::Receiver<StreamEvent>,
    forwarder: &Arc<Forwarder>,
    _inspector: Inspector,
) -> Result<()> {
    while let Some(event) = events.recv().await {
        match event {
            StreamEvent::WebhookReceived {
                event_id,
                event_type,
                payload,
                headers,
                received_at,
                ..
            } => {
                // Forward
                let result = forwarder.forward(&payload, &headers, &event_type).await;

                let json = match result {
                    Ok(r) => {
                        serde_json::json!({
                            "event_id": event_id,
                            "event_type": event_type,
                            "received_at": received_at,
                            "status_code": r.status_code,
                            "elapsed_ms": r.elapsed_ms,
                            "error": r.error,
                        })
                    }
                    Err(e) => {
                        serde_json::json!({
                            "event_id": event_id,
                            "event_type": event_type,
                            "received_at": received_at,
                            "error": e.to_string(),
                        })
                    }
                };

                println!("{}", json);
            }
            StreamEvent::Disconnected => {
                println!(r#"{{"type": "disconnected"}}"#);
                break;
            }
            StreamEvent::Error(e) => {
                println!(r#"{{"type": "error", "message": "{}"}}"#, e);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Fallback polling mode when WebSocket streaming is not available
async fn run_polling_mode(
    _cli: &Cli,
    _args: &ListenArgs,
    _client: &ApiClient,
    _profile: &crate::config::Profile,
    _forwarder: &Arc<Forwarder>,
    _inspector: &Inspector,
) -> Result<()> {
    output_warning("Polling mode is not fully implemented. Please ensure the Hook0 server supports WebSocket streaming.");
    output_info("Waiting for webhooks... (Press Ctrl+C to stop)");

    // In a real implementation, this would poll the API for new events
    // For now, we just wait
    tokio::signal::ctrl_c().await?;

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_label() {
        let result = parse_label("tenant_id=org123");
        assert!(result.is_ok());
        let (k, v) = result.expect("should parse");
        assert_eq!(k, "tenant_id");
        assert_eq!(v, "org123");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a long string", 10), "this is...");
    }
}
