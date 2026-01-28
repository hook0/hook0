pub mod api;
pub mod commands;
pub mod config;
pub mod output;
pub mod tui;
pub mod tunnel;

pub use api::client::ApiClient;
pub use api::ApiError;
pub use config::{Config, Profile};
pub use output::OutputFormat;

use clap::{Parser, Subcommand};

const BANNER: &str = r#"
  _   _             _     ___
 | | | | ___   ___ | | __/ _ \
 | |_| |/ _ \ / _ \| |/ / | | |
 |  _  | (_) | (_) |   <| |_| |
 |_| |_|\___/ \___/|_|\_\\___/

 Webhooks as a Service - https://hook0.com
"#;

#[derive(Parser)]
#[command(name = "hook0")]
#[command(author, version, about = BANNER, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Profile to use
    #[arg(short, long, env = "HOOK0_PROFILE", global = true)]
    pub profile: Option<String>,

    /// Output format: json, table, compact
    #[arg(short, long, default_value = "table", global = true)]
    pub output: OutputFormat,

    /// Override API URL
    #[arg(long, env = "HOOK0_API_URL", global = true)]
    pub api_url: Option<String>,

    /// Override secret
    #[arg(long, env = "HOOK0_SECRET", global = true)]
    pub secret: Option<String>,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Authenticate with an Application Secret
    Login(commands::auth::LoginArgs),

    /// Remove stored credentials
    Logout(commands::auth::LogoutArgs),

    /// Display current application and profile
    Whoami(commands::auth::WhoamiArgs),

    /// Manage webhook events
    #[command(subcommand)]
    Event(commands::event::EventCommands),

    /// Manage event types
    #[command(subcommand, name = "event-type")]
    EventType(commands::event_type::EventTypeCommands),

    /// Manage subscriptions
    #[command(subcommand)]
    Subscription(commands::subscription::SubscriptionCommands),

    /// Manage applications
    #[command(subcommand)]
    Application(commands::application::ApplicationCommands),

    /// Receive webhooks locally (tunneling)
    Listen(commands::listen::ListenArgs),

    /// Replay failed events
    Replay(commands::replay::ReplayArgs),

    /// Manage configuration and profiles
    #[command(subcommand)]
    Config(commands::config::ConfigCommands),

    /// Generate shell completion scripts
    Completion(commands::completion::CompletionArgs),

    /// Quick start wizard for first-time setup
    Init(commands::config::InitArgs),

    /// Quick start with minimal configuration
    Quickstart(commands::config::QuickstartArgs),
}
