mod compact;
mod json;
mod table;

pub use compact::CompactOutput;
pub use json::JsonOutput;
pub use table::TableOutput;

use clap::ValueEnum;
use serde::Serialize;

/// Output format for CLI commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum OutputFormat {
    /// JSON output (machine-readable)
    Json,
    /// Table output (human-readable)
    #[default]
    Table,
    /// Compact output (one line per item)
    Compact,
}

/// Trait for types that can be output in multiple formats
pub trait Outputable: Serialize {
    /// Get table headers for this type
    fn table_headers() -> Vec<&'static str>;

    /// Get table row values for this instance
    fn table_row(&self) -> Vec<String>;

    /// Get compact single-line representation
    fn compact_line(&self) -> String;
}

/// Output a single item in the specified format
pub fn output_one<T: Outputable>(item: &T, format: OutputFormat) {
    match format {
        OutputFormat::Json => JsonOutput::print_one(item),
        OutputFormat::Table => TableOutput::print_one(item),
        OutputFormat::Compact => CompactOutput::print_one(item),
    }
}

/// Output multiple items in the specified format
pub fn output_many<T: Outputable>(items: &[T], format: OutputFormat) {
    match format {
        OutputFormat::Json => JsonOutput::print_many(items),
        OutputFormat::Table => TableOutput::print_many(items),
        OutputFormat::Compact => CompactOutput::print_many(items),
    }
}

/// Output a success message
pub fn output_success(message: &str) {
    use console::style;
    println!("{} {}", style("✓").green().bold(), message);
}

/// Output an error message
pub fn output_error(message: &str) {
    use console::style;
    eprintln!("{} {}", style("✗").red().bold(), message);
}

/// Output a warning message
pub fn output_warning(message: &str) {
    use console::style;
    eprintln!("{} {}", style("!").yellow().bold(), message);
}

/// Output an info message
pub fn output_info(message: &str) {
    use console::style;
    println!("{} {}", style("ℹ").blue().bold(), message);
}

// =============================================================================
// Implement Outputable for API types
// =============================================================================

use crate::api::models::*;

impl Outputable for Application {
    fn table_headers() -> Vec<&'static str> {
        vec!["ID", "Name", "Organization ID"]
    }

    fn table_row(&self) -> Vec<String> {
        vec![
            self.application_id.to_string(),
            self.name.clone(),
            self.organization_id.to_string(),
        ]
    }

    fn compact_line(&self) -> String {
        format!("{}\t{}", self.application_id, self.name)
    }
}

impl Outputable for Organization {
    fn table_headers() -> Vec<&'static str> {
        vec!["ID", "Name", "Role", "Plan"]
    }

    fn table_row(&self) -> Vec<String> {
        vec![
            self.organization_id.to_string(),
            self.name.clone(),
            self.role.clone().unwrap_or_else(|| "-".to_string()),
            self.plan.as_ref().map(|p| p.name.clone()).unwrap_or_else(|| "-".to_string()),
        ]
    }

    fn compact_line(&self) -> String {
        format!("{}\t{}", self.organization_id, self.name)
    }
}

impl Outputable for EventType {
    fn table_headers() -> Vec<&'static str> {
        vec!["Event Type", "Service", "Resource", "Verb"]
    }

    fn table_row(&self) -> Vec<String> {
        vec![
            self.full_name(),
            self.service_name.clone(),
            self.resource_type_name.clone(),
            self.verb_name.clone(),
        ]
    }

    fn compact_line(&self) -> String {
        self.full_name()
    }
}

impl Outputable for Event {
    fn table_headers() -> Vec<&'static str> {
        vec!["ID", "Type", "Occurred At", "Labels"]
    }

    fn table_row(&self) -> Vec<String> {
        let labels = self
            .labels
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(", ");

        vec![
            self.event_id.to_string(),
            self.event_type_name.clone(),
            self.occurred_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            if labels.is_empty() { "-".to_string() } else { labels },
        ]
    }

    fn compact_line(&self) -> String {
        format!(
            "{}\t{}\t{}",
            self.event_id,
            self.event_type_name,
            self.occurred_at.format("%Y-%m-%d %H:%M:%S")
        )
    }
}

impl Outputable for Subscription {
    fn table_headers() -> Vec<&'static str> {
        vec!["ID", "Enabled", "Events", "Target URL", "Labels"]
    }

    fn table_row(&self) -> Vec<String> {
        let target_url = match &self.target {
            Target::Http { url, .. } => url.clone(),
        };

        let labels = self
            .labels
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(", ");

        vec![
            self.subscription_id.to_string(),
            if self.is_enabled { "Yes" } else { "No" }.to_string(),
            self.event_types.join(", "),
            target_url,
            if labels.is_empty() { "-".to_string() } else { labels },
        ]
    }

    fn compact_line(&self) -> String {
        let target_url = match &self.target {
            Target::Http { url, .. } => url.clone(),
        };
        format!(
            "{}\t{}\t{}",
            self.subscription_id,
            if self.is_enabled { "enabled" } else { "disabled" },
            target_url
        )
    }
}

impl Outputable for RequestAttempt {
    fn table_headers() -> Vec<&'static str> {
        vec!["ID", "Event ID", "Status", "Retry", "Created At"]
    }

    fn table_row(&self) -> Vec<String> {
        vec![
            self.request_attempt_id.to_string(),
            self.event_id.to_string(),
            self.status.display_name().to_string(),
            self.retry_count.to_string(),
            self.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        ]
    }

    fn compact_line(&self) -> String {
        format!(
            "{}\t{}\t{}",
            self.request_attempt_id,
            self.status.display_name(),
            self.created_at.format("%H:%M:%S")
        )
    }
}

impl Outputable for ApplicationSecret {
    fn table_headers() -> Vec<&'static str> {
        vec!["Token", "Name", "Created At"]
    }

    fn table_row(&self) -> Vec<String> {
        vec![
            self.token.to_string(),
            self.name.clone().unwrap_or_else(|| "-".to_string()),
            self.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        ]
    }

    fn compact_line(&self) -> String {
        format!(
            "{}\t{}",
            self.token,
            self.name.as_deref().unwrap_or("-")
        )
    }
}

impl Outputable for ServiceToken {
    fn table_headers() -> Vec<&'static str> {
        vec!["ID", "Name", "Created At"]
    }

    fn table_row(&self) -> Vec<String> {
        vec![
            self.token_id.to_string(),
            self.name.clone(),
            self.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        ]
    }

    fn compact_line(&self) -> String {
        format!("{}\t{}", self.token_id, self.name)
    }
}
