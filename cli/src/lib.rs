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
"#;

const HELP_TEMPLATE: &str = "\
{about}
 Webhooks as a Service - https://hook0.com

{usage-heading} {usage}

Getting Started:
  init          Set up your first profile
  login         Authenticate with an Application Secret
  logout        Remove stored credentials
  whoami        Display current application and profile

Local Development:
  listen        Receive webhooks locally via tunnel
  example       Send a sample webhook to test your setup

Webhook Management:
  event         Manage webhook events
  event-type    Manage event types
  subscription  Manage subscriptions
  application   Manage applications
  replay        Replay failed events

Configuration:
  config        Manage configuration and profiles
  completion    Generate shell completion scripts

Options:
{options}
Get started:
  $ hook0 init               Set up your first profile
  $ hook0 listen             Forward webhooks to localhost
  $ hook0 example <URL>      Send a test webhook
";

#[derive(Parser)]
#[command(name = "hook0")]
#[command(author, version, about = BANNER, long_about = None)]
#[command(propagate_version = true)]
#[command(help_template = HELP_TEMPLATE)]
#[command(subcommand_help_heading = "Commands")]
#[command(disable_help_subcommand = true)]
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
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Set up your first profile
    #[command(display_order = 0)]
    Init(commands::config::InitArgs),

    /// Authenticate with an Application Secret
    #[command(display_order = 1)]
    Login(commands::auth::LoginArgs),

    /// Remove stored credentials
    #[command(display_order = 2)]
    Logout(commands::auth::LogoutArgs),

    /// Display current application and profile
    #[command(display_order = 3)]
    Whoami(commands::auth::WhoamiArgs),

    /// Receive webhooks locally via tunnel
    #[command(display_order = 10)]
    Listen(commands::listen::ListenArgs),

    /// Send a sample webhook to test your setup
    #[command(display_order = 11)]
    Example(commands::example::ExampleArgs),

    /// Manage webhook events
    #[command(subcommand, display_order = 20)]
    Event(commands::event::EventCommands),

    /// Manage event types
    #[command(subcommand, name = "event-type", display_order = 21)]
    EventType(commands::event_type::EventTypeCommands),

    /// Manage subscriptions
    #[command(subcommand, display_order = 22)]
    Subscription(commands::subscription::SubscriptionCommands),

    /// Manage applications
    #[command(subcommand, display_order = 23)]
    Application(commands::application::ApplicationCommands),

    /// Replay failed events
    #[command(display_order = 24)]
    Replay(commands::replay::ReplayArgs),

    /// Manage configuration and profiles
    #[command(subcommand, display_order = 30)]
    Config(commands::config::ConfigCommands),

    /// Generate shell completion scripts
    #[command(display_order = 31)]
    Completion(commands::completion::CompletionArgs),
}
