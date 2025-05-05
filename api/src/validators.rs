use reqwest::header::HeaderMap;
use serde_json::Value;
use std::collections::HashMap;
use validator::ValidationError;

const METADATA_MAX_SIZE: usize = 50;
const METADATA_PROPERTY_MIN_LENGTH: usize = 1;
const METADATA_PROPERTY_MAX_LENGTH: usize = 50;
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

const CODE_METADATA_SIZE: &str = "metadata-size";
const CODE_METADATA_PROPERTY_TYPE: &str = "metadata-property-type";
const CODE_METADATA_PROPERTY_LENGTH: &str = "metadata-property-length";
const CODE_LABELS_SIZE: &str = "labels-size";
const CODE_LABELS_PROPERTY_TYPE: &str = "labels-property-type";
const CODE_LABELS_PROPERTY_LENGTH: &str = "labels-property-length";
const CODE_EVENT_TYPES_SIZE: &str = "event-types-size";
const CODE_EVENT_TYPES_NAME_LENGTH: &str = "event-types-name-length";
const CODE_SUBSCRIPTION_TARGET_HTTP_METHOD: &str = "subscription-target-http-method";
const CODE_SUBSCRIPTION_TARGET_HTTP_URL_LENGTH: &str = "subscription-target-http-url-length";
const CODE_SUBSCRIPTION_TARGET_HTTP_HEADERS_SIZE: &str = "subscription-target-http-headers-size";
const CODE_SUBSCRIPTION_TARGET_HTTP_HEADERS_PROPERTY_LENGTH: &str =
    "subscription-target-http-headers-property-length";

fn json_type(val: &Value) -> &'static str {
    match val {
        Value::Array(_) => "array",
        Value::Bool(_) => "boolean",
        Value::Null => "null",
        Value::Number(_) => "number",
        Value::Object(_) => "object",
        Value::String(_) => "string",
    }
}

pub fn metadata(val: &HashMap<String, Value>) -> Result<(), ValidationError> {
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

    let mut invalid_properties = vec![];
    let mut invalid_length = vec![];

    for (k, v) in val {
        if !v.is_string() {
            invalid_properties.push((k, json_type(v)));
        } else if !(METADATA_PROPERTY_MIN_LENGTH..=METADATA_PROPERTY_MAX_LENGTH).contains(&k.len())
            || !(METADATA_PROPERTY_MIN_LENGTH..=METADATA_PROPERTY_MAX_LENGTH).contains(
                &v.as_str()
                    .map(|s| s.len())
                    .unwrap_or(METADATA_PROPERTY_MIN_LENGTH),
            )
        {
            invalid_length.push(k.to_owned());
        }
    }

    if !invalid_properties.is_empty() {
        let invalid = invalid_properties
            .iter()
            .map(|(k, t)| format!("'{k}' → {t}"))
            .collect::<Vec<_>>()
            .join(", ");
        Err(ValidationError {
            code: CODE_METADATA_PROPERTY_TYPE.into(),
            message: Some(format!("Metadata values must be of type string (found the following invalid properties: {invalid})").into()),
            params: HashMap::new(),
        })
    } else if !invalid_length.is_empty() {
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

pub fn labels(val: &HashMap<String, Value>) -> Result<(), ValidationError> {
    if val.len() > LABELS_MAX_SIZE {
        return Err(ValidationError {
            code: CODE_LABELS_SIZE.into(),
            message: Some(
                format!("Labels object cannot have more than {LABELS_MAX_SIZE} properties",).into(),
            ),
            params: HashMap::new(),
        });
    }

    let mut invalid_properties = vec![];
    let mut invalid_length = vec![];

    for (k, v) in val {
        if !v.is_string() {
            invalid_properties.push((k, json_type(v)));
        } else if !(LABELS_PROPERTY_MIN_LENGTH..=LABELS_PROPERTY_MAX_LENGTH).contains(&k.len())
            || !(LABELS_PROPERTY_MIN_LENGTH..=LABELS_PROPERTY_MAX_LENGTH).contains(
                &v.as_str()
                    .map(|s| s.len())
                    .unwrap_or(LABELS_PROPERTY_MIN_LENGTH),
            )
        {
            invalid_length.push(k.to_owned());
        }
    }

    if !invalid_properties.is_empty() {
        let invalid = invalid_properties
            .iter()
            .map(|(k, t)| format!("'{k}' → {t}"))
            .collect::<Vec<_>>()
            .join(", ");
        Err(ValidationError {
            code: CODE_LABELS_PROPERTY_TYPE.into(),
            message: Some(format!("Labels values must be of type string (found the following invalid properties: {invalid})").into()),
            params: HashMap::new(),
        })
    } else if !invalid_length.is_empty() {
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

    use serde_json::json;

    #[test]
    fn metadata_valid() {
        let val = HashMap::from_iter([
            ("key1".to_owned(), json!("val1")),
            ("key2".to_owned(), json!("val2")),
            ("key3".to_owned(), json!("val3")),
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
            val.insert(format!("test-{i}"), json!("test"));
        }
        let output = metadata(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_METADATA_SIZE
        );
    }

    #[test]
    fn metadata_invalid_property_types() {
        let val = HashMap::from_iter([
            ("key1".to_owned(), json!(1)),
            ("key2".to_owned(), json!("val2")),
            ("key3".to_owned(), json!(true)),
        ]);
        let output = metadata(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_METADATA_PROPERTY_TYPE
        );
    }

    #[test]
    fn metadata_invalid_property_length1() {
        let val = HashMap::from_iter([("".to_owned(), json!("val"))]);
        let output = metadata(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_METADATA_PROPERTY_LENGTH
        );
    }

    #[test]
    fn metadata_invalid_property_length2() {
        let val = HashMap::from_iter([("key".to_owned(), json!(""))]);
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
        let val = HashMap::from_iter([(str, json!("val"))]);
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
        let val = HashMap::from_iter([("key".to_owned(), Value::String(str))]);
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
            ("key1".to_owned(), json!("val1")),
            ("key2".to_owned(), json!("val2")),
            ("key3".to_owned(), json!("val3")),
        ]);
        assert!(labels(&val).is_ok())
    }

    #[test]
    fn labels_empty() {
        let val = HashMap::new();
        assert!(labels(&val).is_ok())
    }

    #[test]
    fn labels_invalid_size() {
        let length = LABELS_PROPERTY_MAX_LENGTH + 1;
        let mut val = HashMap::with_capacity(length);
        for i in 0..length {
            val.insert(format!("test-{i}"), json!("test"));
        }
        let output = labels(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_LABELS_SIZE
        );
    }

    #[test]
    fn labels_invalid_property_types() {
        let val = HashMap::from_iter([
            ("key1".to_owned(), json!(1)),
            ("key2".to_owned(), json!("val2")),
            ("key3".to_owned(), json!(true)),
        ]);
        let output = labels(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_LABELS_PROPERTY_TYPE
        );
    }

    #[test]
    fn labels_invalid_property_length1() {
        let val = HashMap::from_iter([("".to_owned(), json!("val"))]);
        let output = labels(&val);
        assert!(output.is_err());
        assert_eq!(
            output.err().map(|e| e.code).unwrap_or_else(|| "".into()),
            CODE_LABELS_PROPERTY_LENGTH
        );
    }

    #[test]
    fn labels_invalid_property_length2() {
        let val = HashMap::from_iter([("key".to_owned(), json!(""))]);
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
        let val = HashMap::from_iter([(str, json!("val"))]);
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
        let val = HashMap::from_iter([("key".to_owned(), Value::String(str))]);
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
