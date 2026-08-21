//! How a generated client presents its credentials.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::error::{Error, preview};
use crate::snapshot::{Operation, Snapshot};

/// What precedes the credential in an `Authorization` header.
///
/// This is the one thing about the API surface written here rather than read from the document,
/// and it is written here so that no target ever writes it again. paperclip 0.9.7, which produces
/// the snapshot, cannot say it: its `#[api_v2_security]` macro accepts `apiKey` and `oauth2` only,
/// and the v2 `SecurityScheme` struct it builds has no field able to carry `scheme: bearer`. The
/// API therefore declares its bearer scheme as an `apiKey` travelling in `Authorization`, and the
/// prefix survives only in the human-readable description. An `apiKey` in that header is a
/// credential scheme followed by a token, so the prefix is restored here, once.
const AUTHORIZATION_PREFIX: &str = "Bearer";

/// Header whose value is a credential scheme followed by the credential itself.
const AUTHORIZATION_HEADER: &str = "Authorization";

/// How a credential travels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scheme {
    /// A header, the credential preceded by `prefix` and a space when there is one.
    Header {
        name: String,
        prefix: Option<String>,
    },
    Query {
        name: String,
    },
    Http {
        scheme: String,
        bearer_format: Option<String>,
    },
}

/// The credentials the API accepts, under the names operations require them by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityModel {
    pub schemes: BTreeMap<String, Scheme>,
}

impl SecurityModel {
    /// Reads the declared schemes, then checks that every operation requires one of them.
    pub(crate) fn read(snapshot: &Snapshot) -> Result<Self, Error> {
        let mut schemes = BTreeMap::new();
        for (name, declared) in snapshot.security_schemes() {
            schemes.insert(name.clone(), scheme(name, declared)?);
        }

        for operation in snapshot.operations() {
            for required in &operation.security {
                if !schemes.contains_key(required) {
                    return Err(Error::UnknownSecurityScheme {
                        operation: preview(&subject(operation)),
                        scheme: preview(required),
                    });
                }
            }
        }

        Ok(Self { schemes })
    }
}

fn scheme(name: &str, declared: &Value) -> Result<Scheme, Error> {
    let unsupported = |declared: &str| Error::UnsupportedSecurityScheme {
        scheme: preview(name),
        declared: declared.to_owned(),
    };

    match declared.get("type").and_then(Value::as_str) {
        Some("apiKey") => {
            let carried = declared
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| unsupported("an API key naming nothing to travel under"))?
                .to_owned();

            match declared.get("in").and_then(Value::as_str) {
                Some("header") => {
                    let prefix =
                        (carried == AUTHORIZATION_HEADER).then(|| AUTHORIZATION_PREFIX.to_owned());
                    Ok(Scheme::Header {
                        name: carried,
                        prefix,
                    })
                }
                Some("query") => Ok(Scheme::Query { name: carried }),
                Some("cookie") => Err(unsupported("an API key travelling in a cookie")),
                _ => Err(unsupported("an API key travelling nowhere named")),
            }
        }
        Some("http") => {
            let carried = declared
                .get("scheme")
                .and_then(Value::as_str)
                .ok_or_else(|| unsupported("an HTTP credential naming no scheme"))?
                .to_owned();

            Ok(Scheme::Http {
                scheme: carried,
                bearer_format: declared
                    .get("bearerFormat")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            })
        }
        Some(declared) => Err(unsupported(&format!("a {} credential", preview(declared)))),
        None => Err(unsupported("a credential of no stated type")),
    }
}

/// How an operation is named in a message about its credentials.
fn subject(operation: &Operation) -> String {
    operation
        .operation_id
        .clone()
        .unwrap_or_else(|| operation.location())
}
