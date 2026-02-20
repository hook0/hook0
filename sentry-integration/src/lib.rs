// Force exposed items to be documented
#![deny(missing_docs)]

//! This is a collection of helpers related to Sentry.

use log::{info, warn};
use sentry::protocol::Value;
use sentry::{ClientInitGuard, Level, User, configure_scope};
use std::collections::BTreeMap;

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
pub fn init(
    crate_name: &'static str,
    sentry_dsn: &Option<String>,
    traces_sample_rate: &Option<f32>,
) -> Option<ClientInitGuard> {
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
                    traces_sample_rate: traces_sample_rate.unwrap_or(0.0),
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

const AUTH_TYPE_PROPERTY: &str = "auth_type";

/// Use JWT claims to set the user to be used in reports
pub fn set_user_from_jwt(id: &str) {
    configure_scope(|scope| {
        scope.set_user(Some(User {
            id: Some(id.to_owned()),
            other: BTreeMap::from_iter([(
                AUTH_TYPE_PROPERTY.to_owned(),
                Value::String("jwt".to_owned()),
            )]),
            ..Default::default()
        }));
    });
}

/// Use an application secret to set the user to be used in reports
pub fn set_user_from_application_secret(application_id: &str) {
    configure_scope(|scope| {
        scope.set_user(Some(User {
            id: Some(application_id.to_owned()),
            other: BTreeMap::from_iter([(
                AUTH_TYPE_PROPERTY.to_owned(),
                Value::String("application_secret".to_owned()),
            )]),
            ..Default::default()
        }));
    });
}

/// Use a token ID to set the user to be used in reports
pub fn set_user_from_token(token_id: &str) {
    configure_scope(|scope| {
        scope.set_user(Some(User {
            id: Some(token_id.to_owned()),
            other: BTreeMap::from_iter([(
                AUTH_TYPE_PROPERTY.to_owned(),
                Value::String("token".to_owned()),
            )]),
            ..Default::default()
        }));
    });
}

/// Logs an object storage error event with static message (for Sentry grouping) and attaches extra context (error chain, object key) to the Sentry event.
/// Also emits a warn-level log line with all details for stdout/log aggregation.
pub fn _log_object_storage_error_with_context(
    module_path: &str,
    file: &str,
    line: u32,
    static_msg: &str,
    error_chain: &str,
    object_key: Option<&str>,
    prefix: Option<&str>,
) {
    sentry::with_scope(
        |scope| {
            scope.set_extra("error_chain", Value::String(error_chain.to_owned()));
            if let Some(key) = object_key {
                scope.set_extra("object_key", Value::String(key.to_owned()));
            }
            if let Some(pfx) = prefix {
                scope.set_extra("prefix", Value::String(pfx.to_owned()));
            }
        },
        || {
            sentry::capture_message(static_msg, Level::Error);
        },
    );

    let mut detail_parts = Vec::new();
    if let Some(key) = object_key {
        detail_parts.push(format!("object_key={key}"));
    }
    if let Some(pfx) = prefix {
        detail_parts.push(format!("prefix={pfx}"));
    }
    detail_parts.push(format!("error_chain={error_chain}"));
    let detail = format!("{static_msg} [{}]", detail_parts.join(", "));
    log::logger().log(
        &log::Record::builder()
            .args(format_args!("{detail}"))
            .level(log::Level::Warn)
            .target(module_path)
            .module_path(Some(module_path))
            .file(Some(file))
            .line(Some(line))
            .build(),
    );
}

/// Logs an S3/object-storage error with a static message for Sentry grouping
/// and a detailed warn-level line for stdout/log aggregation.
#[macro_export]
macro_rules! log_object_storage_error_with_context {
    ($static_msg:literal, error_chain = $chain:expr, object_key = $key:expr, prefix = $prefix:expr $(,)?) => {{
        let __chain: String = $chain;
        let __key: &str = $key;
        let __prefix: &str = $prefix;
        $crate::_log_object_storage_error_with_context(
            module_path!(),
            file!(),
            line!(),
            $static_msg,
            &__chain,
            Some(__key),
            Some(__prefix),
        )
    }};
    ($static_msg:literal, error_chain = $chain:expr, object_key = $key:expr $(,)?) => {{
        let __chain: String = $chain;
        let __key: &str = $key;
        $crate::_log_object_storage_error_with_context(
            module_path!(),
            file!(),
            line!(),
            $static_msg,
            &__chain,
            Some(__key),
            None,
        )
    }};
    ($static_msg:literal, error_chain = $chain:expr, prefix = $prefix:expr $(,)?) => {{
        let __chain: String = $chain;
        let __prefix: &str = $prefix;
        $crate::_log_object_storage_error_with_context(
            module_path!(),
            file!(),
            line!(),
            $static_msg,
            &__chain,
            None,
            Some(__prefix),
        )
    }};
    ($static_msg:literal, error_chain = $chain:expr $(,)?) => {{
        let __chain: String = $chain;
        $crate::_log_object_storage_error_with_context(
            module_path!(),
            file!(),
            line!(),
            $static_msg,
            &__chain,
            None,
            None,
        )
    }};
}
