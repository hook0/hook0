use serde_json::Value;
use std::collections::HashMap;
use validator::ValidationError;

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
    let mut invalid_properties = vec![];
    for (k, v) in val {
        if !v.is_string() {
            invalid_properties.push((k, json_type(v)));
        }
    }

    if invalid_properties.is_empty() {
        Ok(())
    } else {
        let invalid = invalid_properties
            .iter()
            .map(|(k, t)| format!("'{}' → {}", k, t))
            .collect::<Vec<_>>()
            .join(", ");
        Err(ValidationError {
            code: "metadata".into(),
            message: Some(format!("Metadata values must be of type string (found the following invalid properties: {})", &invalid).into()),
            params: HashMap::new(),
        })
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
    fn metadata_invalid() {
        let val = HashMap::from_iter([
            ("key1".to_owned(), json!(1)),
            ("key2".to_owned(), json!("val2")),
            ("key3".to_owned(), json!(true)),
        ]);
        assert!(metadata(&val).is_err())
    }
}
