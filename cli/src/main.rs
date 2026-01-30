use clap::{CommandFactory, Parser};
use std::process::ExitCode;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use hook0_cli::commands::{
    application, auth, completion, config, event, event_type, example, listen, replay, subscription,
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
async fn main() -> ExitCode {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls CryptoProvider");

    let cli = Cli::parse();

    setup_logging(cli.verbose);

    let command = match &cli.command {
        Some(cmd) => cmd,
        None => {
            Cli::command().print_help().expect("failed to print help");
            println!();
            return ExitCode::SUCCESS;
        }
    };

    let result = match command {
        // Getting Started
        Commands::Init(args) => config::init(&cli, args).await,
        Commands::Login(args) => auth::login(&cli, args).await,
        Commands::Logout(args) => auth::logout(&cli, args).await,
        Commands::Whoami(args) => auth::whoami(&cli, args).await,
        // Local Development
        Commands::Listen(args) => listen::execute(&cli, args).await,
        Commands::Example(args) => example::execute(&cli, args).await,
        // Webhook Management
        Commands::Event(cmd) => event::execute(&cli, cmd).await,
        Commands::EventType(cmd) => event_type::execute(&cli, cmd).await,
        Commands::Subscription(cmd) => subscription::execute(&cli, cmd).await,
        Commands::Application(cmd) => application::execute(&cli, cmd).await,
        Commands::Replay(args) => replay::execute(&cli, args).await,
        // Configuration
        Commands::Config(cmd) => config::execute(&cli, cmd).await,
        Commands::Completion(args) => completion::execute(args),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let code = if let Some(api_err) = e.downcast_ref::<hook0_cli::ApiError>() {
                match api_err {
                    hook0_cli::ApiError::Unauthorized => {
                        eprintln!("Error: {e}");
                        eprintln!("\nHint: Run 'hook0 login' to authenticate.");
                        2
                    }
                    hook0_cli::ApiError::NotFound(resource) => {
                        eprintln!("Error: {e}");
                        eprintln!(
                            "\nHint: The {resource} was not found. Check the ID and try again."
                        );
                        3
                    }
                    hook0_cli::ApiError::ValidationError(msg) => {
                        eprintln!("Error: {e}");
                        eprintln!("\nHint: {msg}");
                        4
                    }
                    hook0_cli::ApiError::NetworkError(_) => {
                        eprintln!("Error: {e}");
                        eprintln!("\nHint: Check your internet connection and API URL.");
                        5
                    }
                    _ => {
                        eprintln!("Error: {e}");
                        1
                    }
                }
            } else {
                eprintln!("Error: {e}");
                1
            };

            ExitCode::from(code)
        }
    }
}
