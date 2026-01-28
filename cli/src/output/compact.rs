use super::Outputable;

/// Compact output handler (one line per item)
pub struct CompactOutput;

impl CompactOutput {
    /// Print a single item in compact format
    pub fn print_one<T: Outputable>(item: &T) {
        println!("{}", item.compact_line());
    }

    /// Print multiple items in compact format (one per line)
    pub fn print_many<T: Outputable>(items: &[T]) {
        for item in items {
            println!("{}", item.compact_line());
        }
    }

    /// Print a raw line
    pub fn print_line(line: &str) {
        println!("{}", line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    struct TestItem {
        id: i32,
        name: String,
    }

    impl Outputable for TestItem {
        fn table_headers() -> Vec<&'static str> {
            vec!["ID", "Name"]
        }

        fn table_row(&self) -> Vec<String> {
            vec![self.id.to_string(), self.name.clone()]
        }

        fn compact_line(&self) -> String {
            format!("{}\t{}", self.id, self.name)
        }
    }

    #[test]
    fn test_compact_line() {
        let item = TestItem {
            id: 42,
            name: "test".to_string(),
        };
        assert_eq!(item.compact_line(), "42\ttest");
    }
}
