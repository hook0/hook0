// Force exposed items to be documented
#![deny(missing_docs)]

//! This is a collection of helpers related to Sentry.

use sentry::protocol::Value;
use sentry::{ClientInitGuard, Level, User, configure_scope};
use std::collections::BTreeMap;
use tracing::{info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

/// Initialize Sentry integration
pub fn init(
    sentry_dsn: &Option<String>,
    traces_sample_rate: &Option<f32>,
    debug: bool,
) -> Option<ClientInitGuard> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let client = sentry_dsn.as_deref().map(|dsn| {
        sentry::init((
            dsn,
            sentry::ClientOptions {
                send_default_pii: true,
                attach_stacktrace: true,
                debug,
                traces_sample_rate: traces_sample_rate.unwrap_or(0.0),
                ..Default::default()
            },
        ))
    });

    let sentry_layer = client
        .as_ref()
        .map(|_| sentry::integrations::tracing::layer());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .with(sentry_layer)
        .init();

    if client.as_ref().map(|c| c.is_enabled()).unwrap_or(false) {
        info!("Sentry integration initialized");
    } else {
        warn!("Could not initialize Sentry integration");
    }

    client
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
/// Also emits a warn-level tracing event with all details for stdout/log aggregation.
pub fn _log_object_storage_error_with_context(
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

    warn!(
        error_chain = %error_chain,
        object_key = object_key.unwrap_or(""),
        prefix = prefix.unwrap_or(""),
        "{static_msg}",
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
            $static_msg,
            &__chain,
            Some(__key),
            Some(__prefix),
        )
    }};
    ($static_msg:literal, error_chain = $chain:expr, object_key = $key:expr $(,)?) => {{
        let __chain: String = $chain;
        let __key: &str = $key;
        $crate::_log_object_storage_error_with_context($static_msg, &__chain, Some(__key), None)
    }};
    ($static_msg:literal, error_chain = $chain:expr, prefix = $prefix:expr $(,)?) => {{
        let __chain: String = $chain;
        let __prefix: &str = $prefix;
        $crate::_log_object_storage_error_with_context($static_msg, &__chain, None, Some(__prefix))
    }};
    ($static_msg:literal, error_chain = $chain:expr $(,)?) => {{
        let __chain: String = $chain;
        $crate::_log_object_storage_error_with_context($static_msg, &__chain, None, None)
    }};
}
