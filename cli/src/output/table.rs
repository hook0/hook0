use comfy_table::{presets::UTF8_FULL_CONDENSED, ContentArrangement, Table};

use super::Outputable;

/// Table output handler using comfy-table
pub struct TableOutput;

impl TableOutput {
    /// Print a single item as a table
    pub fn print_one<T: Outputable>(item: &T) {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL_CONDENSED)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(T::table_headers());
        table.add_row(item.table_row());

        println!("{}", table);
    }

    /// Print multiple items as a table
    pub fn print_many<T: Outputable>(items: &[T]) {
        if items.is_empty() {
            println!("No items found.");
            return;
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL_CONDENSED)
            .set_content_arrangement(ContentArrangement::Dynamic);

        // Add headers
        table.set_header(T::table_headers());

        // Add rows
        for item in items {
            table.add_row(item.table_row());
        }

        println!("{}", table);
    }

    /// Print a custom table with provided headers and rows
    pub fn print_custom(headers: Vec<&str>, rows: Vec<Vec<String>>) {
        if rows.is_empty() {
            println!("No items found.");
            return;
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL_CONDENSED)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(headers);

        for row in rows {
            table.add_row(row);
        }

        println!("{}", table);
    }

    /// Print a key-value table (for showing details)
    pub fn print_details(pairs: Vec<(&str, String)>) {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL_CONDENSED)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec!["Field", "Value"]);

        for (key, value) in pairs {
            table.add_row(vec![key.to_string(), value]);
        }

        println!("{}", table);
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
    fn test_table_creation() {
        let items = vec![
            TestItem { id: 1, name: "one".to_string() },
            TestItem { id: 2, name: "two".to_string() },
        ];

        // Just test that table creation doesn't panic
        let mut table = comfy_table::Table::new();
        table.set_header(TestItem::table_headers());
        for item in &items {
            table.add_row(item.table_row());
        }
        let output = table.to_string();
        assert!(output.contains("ID"));
        assert!(output.contains("Name"));
        assert!(output.contains("one"));
        assert!(output.contains("two"));
    }

    #[test]
    fn test_details_table() {
        let pairs = vec![
            ("Name", "Test".to_string()),
            ("ID", "123".to_string()),
        ];

        let mut table = comfy_table::Table::new();
        table.set_header(vec!["Field", "Value"]);
        for (key, value) in pairs {
            table.add_row(vec![key.to_string(), value]);
        }

        let output = table.to_string();
        assert!(output.contains("Field"));
        assert!(output.contains("Value"));
        assert!(output.contains("Name"));
        assert!(output.contains("Test"));
    }
}
