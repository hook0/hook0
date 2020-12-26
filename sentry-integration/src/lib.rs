use log::{info, warn};
use sentry::ClientInitGuard;

/// Initialise a logger with default level at INFO
fn mk_log_builder() -> env_logger::Builder {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
}

/// Register Sentry logger as the global logger
fn init_sentry_logger(crate_name: &'static str) {
    let logger = sentry::integrations::log::SentryLogger::with_dest(mk_log_builder().build())
        .filter(move |md| match (md.target(), md.level()) {
            (_, log::Level::Error) => sentry::integrations::log::LogFilter::Event,
            (target, _) if target == crate_name => sentry::integrations::log::LogFilter::Breadcrumb,
            (_, log::Level::Warn) | (_, log::Level::Info) => {
                sentry::integrations::log::LogFilter::Breadcrumb
            }
            (_, log::Level::Debug) | (_, log::Level::Trace) => {
                sentry::integrations::log::LogFilter::Ignore
            }
        });

    log::set_boxed_logger(Box::new(logger)).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
}

/// Initialize Sentry integration
pub fn init(crate_name: &'static str, sentry_dsn: &Option<String>) -> Option<ClientInitGuard> {
    let client;
    match sentry_dsn {
        Some(dsn) => {
            init_sentry_logger(crate_name);

            client = sentry::init((
                dsn.as_str(),
                sentry::ClientOptions {
                    send_default_pii: true,
                    attach_stacktrace: true,
                    debug: true,
                    ..Default::default()
                },
            ));

            if client.is_enabled() {
                info!("Sentry integration initialized");
            } else {
                unreachable!();
            }

            Some(client)
        }
        None => {
            mk_log_builder().init();
            warn!("Could not initialize Sentry integration");
            None
        }
    }
}
