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

const CODE_METADATA_SIZE: &str = "metadata-size";
const CODE_METADATA_PROPERTY_TYPE: &str = "metadata-property-type";
const CODE_METADATA_PROPERTY_LENGTH: &str = "metadata-property-length";
const CODE_LABELS_SIZE: &str = "labels-size";
const CODE_LABELS_PROPERTY_TYPE: &str = "labels-property-type";
const CODE_LABELS_PROPERTY_LENGTH: &str = "labels-property-length";
const CODE_EVENT_TYPES_SIZE: &str = "event-types-size";
const CODE_EVENT_TYPES_NAME_LENGTH: &str = "event-types-name-length";

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
                format!(
                    "Metadata object cannot have more than {} properties",
                    METADATA_MAX_SIZE
                )
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
            .map(|(k, t)| format!("'{}' → {}", k, t))
            .collect::<Vec<_>>()
            .join(", ");
        Err(ValidationError {
            code: CODE_METADATA_PROPERTY_TYPE.into(),
            message: Some(format!("Metadata values must be of type string (found the following invalid properties: {})", &invalid).into()),
            params: HashMap::new(),
        })
    } else if !invalid_length.is_empty() {
        let invalid = invalid_length.join(", ");
        Err(ValidationError {
            code: CODE_METADATA_PROPERTY_LENGTH.into(),
            message: Some(format!("Metadata properties and values must have a length between {} and {} (the following properties are out of range: {})", METADATA_PROPERTY_MIN_LENGTH, METADATA_PROPERTY_MAX_LENGTH, &invalid).into()),
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
                format!(
                    "Labels object cannot have more than {} properties",
                    LABELS_MAX_SIZE
                )
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
            .map(|(k, t)| format!("'{}' → {}", k, t))
            .collect::<Vec<_>>()
            .join(", ");
        Err(ValidationError {
            code: CODE_LABELS_PROPERTY_TYPE.into(),
            message: Some(format!("Labels values must be of type string (found the following invalid properties: {})", &invalid).into()),
            params: HashMap::new(),
        })
    } else if !invalid_length.is_empty() {
        let invalid = invalid_length.join(", ");
        Err(ValidationError {
            code: CODE_LABELS_PROPERTY_LENGTH.into(),
            message: Some(format!("Labels properties and values must have a length between {} and {} (the following properties are out of range: {})", LABELS_PROPERTY_MIN_LENGTH, LABELS_PROPERTY_MAX_LENGTH, &invalid).into()),
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
                    "There must be between {} and {} event types (found {})",
                    EVENT_TYPES_MIN_SIZE, EVENT_TYPES_MAX_SIZE, size
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
                message: Some(format!("Event types must have a length between {} and {} (invalid event types were spotted at the following indexes: {})", EVENT_TYPES_NAME_MIN_LENGTH, EVENT_TYPES_NAME_MAX_LENGTH, &invalid).into()),
                params: HashMap::new(),
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
            val.insert(format!("test-{}", i), json!("test"));
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
            val.insert(format!("test-{}", i), json!("test"));
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
            val.push(format!("test-{}", i));
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
