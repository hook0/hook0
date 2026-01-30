//! Example command — interactive TUI for composing and sending test webhooks.

use anyhow::Result;
use clap::Args;
use std::time::Duration;

use crate::tunnel::generate_token;
use crate::Cli;

#[derive(Args, Debug)]
pub struct ExampleArgs {
    /// Target URL to forward webhooks to (default: built-in echo server)
    #[arg(long)]
    pub target: Option<String>,

    /// Relay server WebSocket URL
    #[arg(long, default_value = "wss://play.hook0.com/ws")]
    pub relay_url: String,

    /// Token for the webhook URL (auto-generated if not specified)
    #[arg(long)]
    pub token: Option<String>,

    /// Ping interval in seconds
    #[arg(long, default_value = "30")]
    pub ping_interval: u64,

    /// Allow insecure TLS connections
    #[arg(long)]
    pub insecure: bool,
}

pub async fn execute(_cli: &Cli, args: &ExampleArgs) -> Result<()> {
    let token = args.token.clone().unwrap_or_else(generate_token);

    let ping_interval = Duration::from_secs(args.ping_interval);

    #[cfg(feature = "tui")]
    {
        crate::tui::run_example_tui(
            &args.relay_url,
            token,
            args.target.as_deref(),
            ping_interval,
        )
        .await
    }

    #[cfg(not(feature = "tui"))]
    {
        anyhow::bail!("The example TUI requires the `tui` feature. Rebuild with `--features tui`.");
    }
}
