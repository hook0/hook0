use reqwest::header::HeaderMap;
use serde_json::Value;
use std::collections::HashMap;
use validator::{ValidateNonControlCharacter, ValidationError};

use crate::password::MAXIMUM_LENGTH as SECRET_MAX_LENGTH;

const METADATA_MAX_SIZE: usize = 50;
const METADATA_PROPERTY_MIN_LENGTH: usize = 1;
const METADATA_PROPERTY_MAX_LENGTH: usize = 50;
const LABELS_MIN_SIZE: usize = 1;
const LABELS_MAX_SIZE: usize = 10;
const LABELS_PROPERTY_MIN_LENGTH: usize = 1;
const LABELS_PROPERTY_MAX_LENGTH: usize = 50;
const EVENT_TYPES_MIN_SIZE: usize = 1;
const EVENT_TYPES_MAX_SIZE: usize = 100;
const EVENT_TYPES_NAME_MIN_LENGTH: usize = 1;
const EVENT_TYPES_NAME_MAX_LENGTH: usize = 200;
const SUBSCRIPTION_TARGET_HTTP_ALLOWED_METHODS: &[&str] =
    &["GET", "PATCH", "POST", "PUT", "DELETE", "OPTIONS", "HEAD"];
const SUBSCRIPTION_TARGET_HTTP_URL_MAX_LENGTH: usize = 1000;
const SUBSCRIPTION_TARGET_HTTP_HEADERS_MAX_SIZE: usize = 10;
const SUBSCRIPTION_TARGET_HTTP_HEADERS_PROPERTY_MAX_LENGTH: usize = 500;

const SECRET_MIN_LENGTH: usize = 1;

/// Marks a validation error as being about a value the caller must not get
/// back. `Hook0Problem::Validation` drops the refused value from every error
/// whose code starts with this, which is the only way to keep it out of the
/// response: the `validator` derive attaches the value to each error it
/// builds, whichever validator produced it.
pub const CODE_SECRET_PREFIX: &str = "secret-";

const CODE_SECRET_CHARACTERS: &str = "secret-characters";
const CODE_SECRET_LENGTH: &str = "secret-length";
const CODE_METADATA_SIZE: &str = "metadata-size";
const CODE_METADATA_PROPERTY_LENGTH: &str = "metadata-property-length";
const CODE_LABELS_SIZE: &str = "labels-size";
const CODE_LABELS_PROPERTY_LENGTH: &str = "labels-property-length";
const CODE_EVENT_TYPES_SIZE: &str = "event-types-size";
const CODE_EVENT_TYPES_NAME_LENGTH: &str = "event-types-name-length";
const CODE_SUBSCRIPTION_TARGET_HTTP_METHOD: &str = "subscription-target-http-method";
const CODE_SUBSCRIPTION_TARGET_HTTP_URL_LENGTH: &str = "subscription-target-http-url-length";
const CODE_SUBSCRIPTION_TARGET_HTTP_HEADERS_SIZE: &str = "subscription-target-http-headers-size";
const CODE_SUBSCRIPTION_TARGET_HTTP_HEADERS_PROPERTY_LENGTH: &str =
    "subscription-target-http-headers-property-length";

/// Reject control characters in a secret without putting the secret in the
/// error. Validation errors are serialized whole into the response body, and
/// the built-in `non_control_character` and `length` validators hand back the
/// value they refused as a `value` parameter — for a password, that returns
/// it to the caller, to the browser console, and to anything that logs
/// response bodies. Same rule as `validate_non_control_character`, without the
/// echo.
pub fn secret_characters(val: &str) -> Result<(), ValidationError> {
    if val.validate_non_control_character() {
        return Ok(());
    }
    Err(ValidationError {
        code: CODE_SECRET_CHARACTERS.into(),
        message: Some("Password must not contain control characters".into()),
        params: HashMap::new(),
    })
}

/// A secret no policy will bound afterwards: logging in must accept whatever
/// the account's password happens to be, so nothing downstream rejects an
/// oversized one. Bounds it here, under the same no-echo rule as
/// `secret_characters`.
pub fn secret(val: &str) -> Result<(), ValidationError> {
    secret_characters(val)?;

    let length = val.chars().count();
    if (SECRET_MIN_LENGTH..=SECRET_MAX_LENGTH).contains(&length) {
        return Ok(());
    }
    Err(ValidationError {
        code: CODE_SECRET_LENGTH.into(),
        message: Some(
            format!(
                "Password must be between {SECRET_MIN_LENGTH} and {SECRET_MAX_LENGTH} characters"
            )
            .into(),
        ),
        params: HashMap::new(),
    })
}

pub fn metadata(val: &HashMap<String, String>) -> Result<(), ValidationError> {
    if val.len() > METADATA_MAX_SIZE {
        return Err(ValidationError {
            code: CODE_METADATA_SIZE.into(),
            message: Some(
                format!("Metadata object cannot have more than {METADATA_MAX_SIZE} properties",)
                    .into(),
            ),
            params: HashMap::new(),
        });
    }

    let mut invalid_length = vec![];

    for (k, v) in val {
        if !(METADATA_PROPERTY_MIN_LENGTH..=METADATA_PROPERTY_MAX_LENGTH).contains(&k.len())
            || !(METADATA_PROPERTY_MIN_LENGTH..=METADATA_PROPERTY_MAX_LENGTH).contains(&v.len())
        {
            invalid_length.push(k.to_owned());
        }
    }

    if !invalid_length.is_empty() {
        let invalid = invalid_length.join(", ");
        Err(ValidationError {
            code: CODE_METADATA_PROPERTY_LENGTH.into(),
            message: Some(format!("Metadata properties and values must have a length between {METADATA_PROPERTY_MIN_LENGTH} and {METADATA_PROPERTY_MAX_LENGTH} (the following properties are out of range: {invalid})").into()),
            params: HashMap::new(),
        })
    } else {
        Ok(())
    }
}

pub fn labels(val: &HashMap<String, String>) -> Result<(), ValidationError> {
    if val.len() < LABELS_MIN_SIZE {
        return Err(ValidationError {
            code: CODE_LABELS_SIZE.into(),
            message: Some(
                format!("Labels object must have at least {LABELS_MIN_SIZE} properties",).into(),
            ),
            params: HashMap::new(),
        });
    }
    if val.len() > LABELS_MAX_SIZE {
        return Err(ValidationError {
            code: CODE_LABELS_SIZE.into(),
            message: Some(
                format!("Labels object cannot have more than {LABELS_MAX_SIZE} properties",).into(),
            ),
            params: HashMap::new(),
        });
    }

    let mut invalid_length = vec![];

    for (k, v) in val {
        if !(LABELS_PROPERTY_MIN_LENGTH..=LABELS_PROPERTY_MAX_LENGTH).contains(&k.len())
            || !(LABELS_PROPERTY_MIN_LENGTH..=LABELS_PROPERTY_MAX_LENGTH).contains(&v.len())
        {
            invalid_length.push(k.to_owned());
        }
    }

    if !invalid_length.is_empty() {
        let invalid = invalid_length.join(", ");
        Err(ValidationError {
            code: CODE_LABELS_PROPERTY_LENGTH.into(),
            message: Some(format!("Labels properties and values must have a length between {LABELS_PROPERTY_MIN_LENGTH} and {LABELS_PROPERTY_MAX_LENGTH} (the following properties are out of range: {invalid})").into()),
            params: HashMap::new(),
        })
    } else {
        Ok(())
    }
}

pub fn event_types(val: &[String]) -> Result<(), ValidationError> {
    let size = val.len();
    if !(EVENT_TYPES_MIN_SIZE..=EVENT_TYPES_MAX_SIZE).contains(&size) {
        return Err(ValidationError {
            code: CODE_EVENT_TYPES_SIZE.into(),
            message: Some(
                format!(
                    "There must be between {EVENT_TYPES_MIN_SIZE} and {EVENT_TYPES_MAX_SIZE} event types (found {size})"
                )
                .into(),
            ),
            params: HashMap::new(),
        });
    }

    let mut invalid_names = vec![];

    for (index, name) in val.iter().enumerate() {
        if !(EVENT_TYPES_NAME_MIN_LENGTH..=EVENT_TYPES_NAME_MAX_LENGTH).contains(&name.len()) {
            invalid_names.push(index);
        }
    }

    if !invalid_names.is_empty() {
        let invalid = invalid_names
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        Err(ValidationError {
                code: CODE_EVENT_TYPES_NAME_LENGTH.into(),
                message: Some(format!("Event types must have a length between {EVENT_TYPES_NAME_MIN_LENGTH} and {EVENT_TYPES_NAME_MAX_LENGTH} (invalid event types were spotted at the following indexes: {invalid})").into()),
                params: HashMap::new(),
            })
    } else {
        Ok(())
    }
}

pub fn subscription_target_http_method(val: &String) -> Result<(), ValidationError> {
    if !SUBSCRIPTION_TARGET_HTTP_ALLOWED_METHODS.contains(&val.as_str()) {
        Err(ValidationError {
            code: CODE_SUBSCRIPTION_TARGET_HTTP_METHOD.into(),
            message: Some(
                format!(
                    "HTTP method must be one of: {}",
                    SUBSCRIPTION_TARGET_HTTP_ALLOWED_METHODS.to_vec().join(", ")
                )
                .into(),
            ),
            params: HashMap::from_iter([
                ("value".into(), Value::String(val.to_owned())),
                (
                    "options".into(),
                    Value::Array(
                        SUBSCRIPTION_TARGET_HTTP_ALLOWED_METHODS
                            .iter()
                            .map(|m| Value::String(m.to_owned().to_owned()))
                            .collect::<Vec<_>>(),
                    ),
                ),
            ]),
        })
    } else {
        Ok(())
    }
}

pub fn subscription_target_http_url(val: &str) -> Result<(), ValidationError> {
    if val.len() > SUBSCRIPTION_TARGET_HTTP_URL_MAX_LENGTH {
        Err(ValidationError {
            code: CODE_SUBSCRIPTION_TARGET_HTTP_URL_LENGTH.into(),
            message: Some(
                format!("HTTP URL must be smaller than {SUBSCRIPTION_TARGET_HTTP_URL_MAX_LENGTH} characters")
                .into(),
            ),
            params: HashMap::from_iter([
                ("length".into(), Value::Number(val.len().into())),
                (
                    "max".into(),
                    Value::Number(SUBSCRIPTION_TARGET_HTTP_URL_MAX_LENGTH.into())
                ),
            ]),
        })
    } else {
        Ok(())
    }
}

pub fn subscription_target_http_method_headers(val: &HeaderMap) -> Result<(), ValidationError> {
    if val.len() > SUBSCRIPTION_TARGET_HTTP_HEADERS_MAX_SIZE {
        return Err(ValidationError {
            code: CODE_SUBSCRIPTION_TARGET_HTTP_HEADERS_SIZE.into(),
            message: Some(
                format!("Headers object cannot have more than {SUBSCRIPTION_TARGET_HTTP_HEADERS_MAX_SIZE} properties",)
                    .into(),
            ),
            params: HashMap::from_iter([
                ("max".into(), Value::Number(SUBSCRIPTION_TARGET_HTTP_HEADERS_MAX_SIZE.into()))
            ]),
        });
    }

    let mut invalid_length = vec![];

    for (k, v) in val {
        if k.as_str().len() > SUBSCRIPTION_TARGET_HTTP_HEADERS_PROPERTY_MAX_LENGTH
            || v.len() > SUBSCRIPTION_TARGET_HTTP_HEADERS_PROPERTY_MAX_LENGTH
        {
            invalid_length.push(k.to_owned());
        }
    }

    if !invalid_length.is_empty() {
        let invalid = invalid_length.join(", ");
        Err(ValidationError {
            code: CODE_SUBSCRIPTION_TARGET_HTTP_HEADERS_PROPERTY_LENGTH.into(),
            message: Some(format!("Headers properties and values must contains less than {METADATA_PROPERTY_MAX_LENGTH} characters (the following properties are out of range: {invalid})").into()),
            params: HashMap::from_iter([
                ("max".into(), Value::Number(SUBSCRIPTION_TARGET_HTTP_HEADERS_PROPERTY_MAX_LENGTH.into())),
                ("invalid_properties".into(), Value::Array(invalid_length.into_iter().map(|v| Value::String(v.as_str().to_owned())).collect::<Vec<_>>())),
            ]),
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The prefix is the whole mechanism: `Hook0Problem::Validation` decides
    /// what to strip from a code alone, so a secret validator whose code drops
    /// the prefix would start returning the password again, silently.
    #[test]
    fn every_secret_validator_code_carries_the_prefix() {
        for code in [CODE_SECRET_CHARACTERS, CODE_SECRET_LENGTH] {
            assert!(
                code.starts_with(CODE_SECRET_PREFIX),
                "{code} would not be recognised as being about a secret"
            );
        }
    }

    #[test]
    fn a_secret_may_contain_anything_printable() {
        assert!(secret("quilt lantern harbour ✓ 𝐀").is_ok());
        assert!(secret_characters("quilt lantern harbour ✓ 𝐀").is_ok());
    }

    #[test]
    fn a_secret_is_refused_for_control_characters_and_for_length() {
        assert_eq!(
            secret_characters("quilt\u{7}lantern")
                .err()
                .map(|e| e.code)
                .unwrap_or_else(|| "".into()),
            CODE_SECRET_CHARACTERS
        );
        assert_eq!(
            secret("")
                .err()
                .map(|e| e.code)
                .unwrap_or_else(|| "".into()),
            CODE_SECRET_LENGTH
        );
        assert_eq!(
            secret(&"x".repeat(SECRET_MAX_LENGTH + 1))
                .err()
                .map(|e| e.code)
                .unwrap_or_else(|| "".into()),
            CODE_SECRET_LENGTH
        );
        // Counted in characters, not bytes: a long password made of multi-byte
        // characters must not be refused for a length it does not have.
        assert!(secret(&"é".repeat(SECRET_MAX_LENGTH)).is_ok());
    }

    #[test]
    fn metadata_valid() {
        let val = HashMap::from_iter([
            ("key1".to_owned(), "val1".to_owned()),
            ("key2".to_owned(), "val2".to_owned()),
            ("key3".to_owned(), "val3".to_owned()),
        ]);
        assert!(metadata(&val).is_ok())
    }

    #[test]
    fn metadata_empty() {
        let val = HashMap::new();
        assert!(metadata(&val).is_ok())
    }

    #[test]
    fn metadata_invalid_size() {
        let length = METADATA_PROPERTY_MAX_LENGTH + 1;
        let mut val = HashMap::with_capacity(length);
        for i in 0..length {
            val.insert(format!("test-{i}"), "test".to_owned());
        }
        let output = metadata(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_METADATA_SIZE
        );
    }

    #[test]
    fn metadata_invalid_property_length1() {
        let val = HashMap::from_iter([("".to_owned(), "val".to_owned())]);
        let output = metadata(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_METADATA_PROPERTY_LENGTH
        );
    }

    #[test]
    fn metadata_invalid_property_length2() {
        let val = HashMap::from_iter([("key".to_owned(), "".to_owned())]);
        let output = metadata(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_METADATA_PROPERTY_LENGTH
        );
    }

    #[test]
    fn metadata_invalid_property_length3() {
        let mut str = String::new();
        for _ in 0..=METADATA_PROPERTY_MAX_LENGTH {
            str.push('_');
        }
        let val = HashMap::from_iter([(str, "val".to_owned())]);
        let output = metadata(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_METADATA_PROPERTY_LENGTH
        );
    }

    #[test]
    fn metadata_invalid_property_length4() {
        let mut str = String::new();
        for _ in 0..=METADATA_PROPERTY_MAX_LENGTH {
            str.push('_');
        }
        let val = HashMap::from_iter([("key".to_owned(), str)]);
        let output = metadata(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_METADATA_PROPERTY_LENGTH
        );
    }

    #[test]
    fn labels_valid() {
        let val = HashMap::from_iter([
            ("key1".to_owned(), "val1".to_owned()),
            ("key2".to_owned(), "val2".to_owned()),
            ("key3".to_owned(), "val3".to_owned()),
        ]);
        assert!(labels(&val).is_ok())
    }

    #[test]
    fn labels_empty() {
        let val = HashMap::new();
        let output = labels(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_LABELS_SIZE
        )
    }

    #[test]
    fn labels_invalid_size() {
        let length = LABELS_PROPERTY_MAX_LENGTH + 1;
        let mut val = HashMap::with_capacity(length);
        for i in 0..length {
            val.insert(format!("test-{i}"), "test".to_owned());
        }
        let output = labels(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_LABELS_SIZE
        );
    }

    #[test]
    fn labels_invalid_property_length1() {
        let val = HashMap::from_iter([("".to_owned(), "val".to_owned())]);
        let output = labels(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_LABELS_PROPERTY_LENGTH
        );
    }

    #[test]
    fn labels_invalid_property_length2() {
        let val = HashMap::from_iter([("key".to_owned(), "".to_owned())]);
        let output = labels(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_LABELS_PROPERTY_LENGTH
        );
    }

    #[test]
    fn labels_invalid_property_length3() {
        let mut str = String::new();
        for _ in 0..=LABELS_PROPERTY_MAX_LENGTH {
            str.push('_');
        }
        let val = HashMap::from_iter([(str, "val".to_owned())]);
        let output = labels(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_LABELS_PROPERTY_LENGTH
        );
    }

    #[test]
    fn labels_invalid_property_length4() {
        let mut str = String::new();
        for _ in 0..=LABELS_PROPERTY_MAX_LENGTH {
            str.push('_');
        }
        let val = HashMap::from_iter([("key".to_owned(), str)]);
        let output = labels(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_LABELS_PROPERTY_LENGTH
        );
    }

    #[test]
    fn event_types_valid() {
        let val = vec!["type1".to_owned(), "type2".to_owned(), "type3".to_owned()];
        assert!(event_types(&val).is_ok())
    }

    #[test]
    fn event_types_empty() {
        let val = vec![];
        let output = event_types(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_EVENT_TYPES_SIZE
        );
    }

    #[test]
    fn event_types_invalid_size() {
        let length = EVENT_TYPES_MAX_SIZE + 1;
        let mut val = Vec::with_capacity(length);
        for i in 0..length {
            val.push(format!("test-{i}"));
        }
        let output = event_types(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_EVENT_TYPES_SIZE
        );
    }

    #[test]
    fn event_types_invalid_name_length1() {
        let val = vec!["".to_owned()];
        let output = event_types(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_EVENT_TYPES_NAME_LENGTH
        );
    }

    #[test]
    fn event_types_invalid_name_length2() {
        let mut str = String::new();
        for _ in 0..=EVENT_TYPES_NAME_MAX_LENGTH {
            str.push('_');
        }
        let val = vec![str];
        let output = event_types(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_EVENT_TYPES_NAME_LENGTH
        );
    }
}
