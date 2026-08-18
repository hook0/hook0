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
use std::process::ExitCode;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// What this binary is called where a reader will type it.
const PROGRAM: &str = "hook0-mcp";

/// How this server is told what to do.
///
/// It takes no options of its own beyond the two flags below. Everything else is an environment
/// variable, which is what an MCP client can set in the block it launches a server from, and is
/// why there is nothing here worth an argument-parsing crate.
const USAGE: &str = "\
Usage: hook0-mcp [--version | --help]

An MCP server for the Hook0 API, speaking the Model Context Protocol over stdio.

Required environment variables:
  HOOK0_API_TOKEN  Your Hook0 API token

Optional environment variables:
  HOOK0_API_URL    API base URL (default: https://app.hook0.com)
  HOOK0_READ_ONLY  Set to 'true' to only expose read operations (default: false)
  MCP_TRANSPORT    Transport type: stdio (default; sse is not implemented)
  MCP_SSE_PORT     Port for SSE server (reserved, not implemented)";

/// What a command line asked this binary to do.
#[derive(Debug, PartialEq, Eq)]
enum Asked {
    /// Serve, which is what no argument at all means.
    Serve,
    /// Say which version this is.
    Version,
    /// Say how it is run.
    Help,
    /// Something this binary has no meaning for, reported rather than ignored: an argument that
    /// silently starts the server is a typo that looks like it worked.
    Unknown(String),
}

/// What a command line asked for, from the arguments after the program name.
///
/// The first argument decides. Nothing here takes a value, so a second argument is one this binary
/// has no meaning for and is reported as such.
fn asked<A: IntoIterator<Item = String>>(arguments: A) -> Asked {
    let mut arguments = arguments.into_iter();
    let Some(first) = arguments.next() else {
        return Asked::Serve;
    };

    match first.as_str() {
        "--version" | "-V" => Asked::Version,
        "--help" | "-h" => Asked::Help,
        other => Asked::Unknown(other.to_owned()),
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    // Read before anything else is set up, so that `--version` answers on a machine holding no
    // token and prints the version rather than a configuration error.
    match asked(std::env::args().skip(1)) {
        Asked::Serve => {}
        Asked::Version => {
            println!("{PROGRAM} {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Asked::Help => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Asked::Unknown(argument) => {
            eprintln!("{PROGRAM}: unknown argument `{argument}`");
            eprintln!();
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    }

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
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
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
            return ExitCode::FAILURE;
        }
    };

    // Create MCP server
    let server = Hook0McpServer::new(client, config.read_only);

    // Serve based on transport type
    match config.transport {
        Transport::Stdio => {
            info!("Serving over stdio transport");
            match server.serve(stdio()).await {
                Ok(service) => match service.waiting().await {
                    Ok(_) => ExitCode::SUCCESS,
                    Err(e) => {
                        error!("Server error: {}", e);
                        ExitCode::FAILURE
                    }
                },
                Err(e) => {
                    error!("Failed to start server: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        Transport::Sse { port } => {
            // Silently serving stdio here would give a server that works, but not on
            // the transport that was asked for. Refuse instead of pretending.
            error!(
                "SSE transport (requested on port {}) is not implemented; only MCP_TRANSPORT=stdio is supported",
                port
            );
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A command line of no arguments is what an MCP client sends, and it means serve.
    #[test]
    fn no_argument_serves() {
        assert_eq!(asked(Vec::<String>::new()), Asked::Serve);
    }

    /// Both spellings of each flag answer, because both are what a reader tries first.
    #[test]
    fn the_two_flags_answer_under_either_spelling() {
        for spelling in ["--version", "-V"] {
            assert_eq!(asked([spelling.to_owned()]), Asked::Version);
        }
        for spelling in ["--help", "-h"] {
            assert_eq!(asked([spelling.to_owned()]), Asked::Help);
        }
    }

    /// An argument with no meaning here is reported rather than dropped.
    ///
    /// Dropping it is what this binary used to do with every argument, so `hook0-mcp --version`
    /// started a server, and a typed flag looked like it had worked.
    #[test]
    fn an_argument_with_no_meaning_is_reported() {
        assert_eq!(
            asked(["--verison".to_owned()]),
            Asked::Unknown("--verison".to_owned())
        );
        assert_eq!(
            asked(["serve".to_owned()]),
            Asked::Unknown("serve".to_owned())
        );
    }
}
