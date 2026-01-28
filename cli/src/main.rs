use anyhow::{anyhow, Result};
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use hook0_cli::commands::{
    application, auth, completion, config, event, event_type, listen, replay, subscription,
};
use hook0_cli::{Cli, Commands};

fn setup_logging(verbosity: u8) {
    let filter = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    setup_logging(cli.verbose);

    let result = match &cli.command {
        Commands::Login(args) => auth::login(&cli, args).await,
        Commands::Logout(args) => auth::logout(&cli, args).await,
        Commands::Whoami(args) => auth::whoami(&cli, args).await,
        Commands::Event(cmd) => event::execute(&cli, cmd).await,
        Commands::EventType(cmd) => event_type::execute(&cli, cmd).await,
        Commands::Subscription(cmd) => subscription::execute(&cli, cmd).await,
        Commands::Application(cmd) => application::execute(&cli, cmd).await,
        Commands::Listen(args) => listen::execute(&cli, args).await,
        Commands::Replay(args) => replay::execute(&cli, args).await,
        Commands::Config(cmd) => config::execute(&cli, cmd).await,
        Commands::Completion(args) => completion::execute(args),
        Commands::Init(args) => config::init(&cli, args).await,
        Commands::Quickstart(args) => config::quickstart(&cli, args).await,
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            // Provide contextual hints based on error type
            if let Some(api_err) = e.downcast_ref::<hook0_cli::ApiError>() {
                match api_err {
                    hook0_cli::ApiError::Unauthorized => {
                        Err(anyhow!("{e}\n\nHint: Run 'hook0 login' to authenticate."))
                    }
                    hook0_cli::ApiError::NotFound(resource) => {
                        Err(anyhow!("{e}\n\nHint: The {resource} was not found. Check the ID and try again."))
                    }
                    hook0_cli::ApiError::ValidationError(msg) => {
                        Err(anyhow!("{e}\n\nHint: {msg}"))
                    }
                    hook0_cli::ApiError::NetworkError(_) => {
                        Err(anyhow!("{e}\n\nHint: Check your internet connection and API URL."))
                    }
                    _ => Err(e),
                }
            } else {
                Err(e)
            }
        }
    }
}
