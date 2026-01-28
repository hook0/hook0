use serde::Serialize;

/// JSON output handler
pub struct JsonOutput;

impl JsonOutput {
    /// Print a single item as JSON
    pub fn print_one<T: Serialize>(item: &T) {
        let json = serde_json::to_string_pretty(item).expect("Failed to serialize to JSON");
        println!("{}", json);
    }

    /// Print multiple items as a JSON array
    pub fn print_many<T: Serialize>(items: &[T]) {
        let json = serde_json::to_string_pretty(items).expect("Failed to serialize to JSON");
        println!("{}", json);
    }

    /// Print raw JSON value
    pub fn print_raw(value: &serde_json::Value) {
        let json = serde_json::to_string_pretty(value).expect("Failed to serialize to JSON");
        println!("{}", json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestItem {
        id: i32,
        name: String,
    }

    #[test]
    fn test_json_serialization() {
        let item = TestItem {
            id: 1,
            name: "test".to_string(),
        };

        // This just tests that serialization works without panicking
        let json = serde_json::to_string_pretty(&item).expect("serialization should work");
        assert!(json.contains("\"id\": 1"));
        assert!(json.contains("\"name\": \"test\""));
    }

    #[test]
    fn test_json_array_serialization() {
        let items = vec![
            TestItem { id: 1, name: "one".to_string() },
            TestItem { id: 2, name: "two".to_string() },
        ];

        let json = serde_json::to_string_pretty(&items).expect("serialization should work");
        assert!(json.contains("\"id\": 1"));
        assert!(json.contains("\"id\": 2"));
    }
}
