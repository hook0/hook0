use log::{info, warn};

// Initialize Sentry integration
pub fn init() {
    let dsn = std::env::var("SENTRY_DSN").unwrap_or_else(|_| "".to_string());

    let client = sentry::init((
        dsn,
        sentry::ClientOptions {
            send_default_pii: true,
            attach_stacktrace: true,
            ..Default::default()
        },
    ));

    sentry::integrations::panic::register_panic_handler();
    sentry::integrations::env_logger::init(Some(env_logger::builder().build()), Default::default());
    if client.is_enabled() {
        std::env::set_var("RUST_BACKTRACE", "1");
        info!("Sentry integration initialized");
    } else {
        warn!("Could not initialize Sentry integration");
    }
}
