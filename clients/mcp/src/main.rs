//! Hook0 MCP Server binary
//!
//! Run with:
//! ```bash
//! export HOOK0_API_TOKEN="your-api-token"
//! hook0-mcp
//! ```

use hook0_mcp::{Config, Hook0Client, Hook0McpServer, Transport};
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing (logs to stderr to not interfere with stdio transport)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(false),
        )
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load configuration
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            error!("Configuration error: {}", e);
            eprintln!("Error: {}", e);
            eprintln!();
            eprintln!("Required environment variables:");
            eprintln!("  HOOK0_API_TOKEN  - Your Hook0 API token");
            eprintln!();
            eprintln!("Optional environment variables:");
            eprintln!("  HOOK0_API_URL    - API base URL (default: https://app.hook0.com)");
            eprintln!(
                "  HOOK0_READ_ONLY  - Set to 'true' to only expose read operations (default: false)"
            );
            eprintln!("  MCP_TRANSPORT    - Transport type: stdio or sse (default: stdio)");
            eprintln!("  MCP_SSE_PORT     - Port for SSE server (default: 3000)");
            std::process::exit(1);
        }
    };

    info!(
        "Starting Hook0 MCP server v{} with {:?} transport{}",
        env!("CARGO_PKG_VERSION"),
        config.transport,
        if config.read_only {
            " (read-only mode)"
        } else {
            ""
        }
    );

    // Create HTTP client
    let client = match Hook0Client::new(&config) {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to create HTTP client: {}", e);
            std::process::exit(1);
        }
    };

    // Create MCP server
    let server = Hook0McpServer::new(client, config.read_only);

    // Serve based on transport type
    match config.transport {
        Transport::Stdio => {
            info!("Serving over stdio transport");
            match server.serve(stdio()).await {
                Ok(service) => {
                    if let Err(e) = service.waiting().await {
                        error!("Server error: {}", e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    error!("Failed to start server: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Transport::Sse { port } => {
            info!("SSE transport requested on port {}", port);
            eprintln!("SSE transport is not yet implemented. Using stdio instead.");
            // For now, fall back to stdio
            // TODO: Implement SSE transport when rmcp supports it properly
            match server.serve(stdio()).await {
                Ok(service) => {
                    if let Err(e) = service.waiting().await {
                        error!("Server error: {}", e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    error!("Failed to start server: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
