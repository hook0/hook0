use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// =============================================================================
// Organization
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub organization_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub plan: Option<Plan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub name: String,
    #[serde(default)]
    pub label: Option<String>,
}

// =============================================================================
// Application
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub application_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub quotas: Option<Quotas>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quotas {
    pub events_per_day_limit: i32,
    pub days_of_events_retention_limit: i32,
    #[serde(default)]
    pub applications_per_organization_limit: Option<i32>,
    #[serde(default)]
    pub members_per_organization_limit: Option<i32>,
    #[serde(default)]
    pub subscriptions_per_application_limit: Option<i32>,
    #[serde(default)]
    pub event_types_per_application_limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationPost {
    pub organization_id: Uuid,
    pub name: String,
}

// =============================================================================
// Application Secret (API Token)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationSecret {
    pub token: Uuid,
    #[serde(default)]
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationSecretPost {
    pub application_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// =============================================================================
// Service Token
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceToken {
    pub token_id: Uuid,
    pub name: String,
    pub biscuit: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceTokenPost {
    pub organization_id: Uuid,
    pub name: String,
}

// =============================================================================
// Event Type
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventType {
    pub service_name: String,
    pub resource_type_name: String,
    pub verb_name: String,
    #[serde(default)]
    pub event_type_name: Option<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
}

impl EventType {
    /// Generate the full event type name from components
    pub fn full_name(&self) -> String {
        self.event_type_name.clone().unwrap_or_else(|| {
            format!(
                "{}.{}.{}",
                self.service_name, self.resource_type_name, self.verb_name
            )
        })
    }

    /// Parse an event type name into components
    pub fn parse(event_type_name: &str) -> Option<(String, String, String)> {
        let parts: Vec<&str> = event_type_name.split('.').collect();
        if parts.len() == 3 {
            Some((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTypePost {
    pub application_id: Uuid,
    pub service: String,
    pub resource_type: String,
    pub verb: String,
}

impl EventTypePost {
    /// Create from a full event type name (e.g., "user.account.created")
    pub fn from_name(application_id: Uuid, name: &str) -> Option<Self> {
        EventType::parse(name).map(|(service, resource_type, verb)| Self {
            application_id,
            service,
            resource_type,
            verb,
        })
    }
}

// =============================================================================
// Event
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: Uuid,
    pub application_id: Uuid,
    pub event_type_name: String,
    pub payload: String,
    pub payload_content_type: String,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    pub occurred_at: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    pub event_id: Uuid,
    pub event_type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPost {
    pub application_id: Uuid,
    pub event_id: Uuid,
    pub event_type: String,
    pub payload: String,
    pub payload_content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    pub occurred_at: DateTime<Utc>,
}

impl EventPost {
    /// Create a new event with JSON payload
    pub fn new_json(
        application_id: Uuid,
        event_type: String,
        payload: serde_json::Value,
        labels: HashMap<String, String>,
    ) -> Self {
        Self {
            application_id,
            event_id: Uuid::new_v4(),
            event_type,
            payload: base64_encode(&payload.to_string()),
            payload_content_type: "application/json".to_string(),
            metadata: None,
            labels,
            occurred_at: Utc::now(),
        }
    }

    /// Create a new event with plain text payload
    pub fn new_text(
        application_id: Uuid,
        event_type: String,
        payload: String,
        labels: HashMap<String, String>,
    ) -> Self {
        Self {
            application_id,
            event_id: Uuid::new_v4(),
            event_type,
            payload: base64_encode(&payload),
            payload_content_type: "text/plain".to_string(),
            metadata: None,
            labels,
            occurred_at: Utc::now(),
        }
    }
}

// =============================================================================
// Subscription
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub subscription_id: Uuid,
    pub application_id: Uuid,
    pub is_enabled: bool,
    pub event_types: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub secret: Uuid,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    pub target: Target,
    #[serde(default)]
    pub dedicated_workers: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionSummary {
    pub subscription_id: Uuid,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Target {
    Http {
        method: String,
        url: String,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
}

impl Target {
    pub fn http(url: String) -> Self {
        Self::Http {
            method: "POST".to_string(),
            url,
            headers: HashMap::new(),
        }
    }

    pub fn http_with_headers(
        url: String,
        method: String,
        headers: HashMap<String, String>,
    ) -> Self {
        Self::Http {
            method,
            url,
            headers,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPost {
    pub application_id: Uuid,
    pub event_types: Vec<String>,
    pub is_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    pub target: Target,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedicated_workers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPut {
    pub event_types: Vec<String>,
    pub is_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    pub target: Target,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedicated_workers: Option<Vec<String>>,
}

// =============================================================================
// Request Attempt (Webhook Delivery)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestAttempt {
    pub request_attempt_id: Uuid,
    pub event_id: Uuid,
    #[serde(default)]
    pub event: Option<EventSummary>,
    #[serde(default)]
    pub subscription: Option<SubscriptionSummary>,
    pub status: RequestAttemptStatus,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub picked_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub failed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub succeeded_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub delay_until: Option<DateTime<Utc>>,
    #[serde(default)]
    pub response_id: Option<Uuid>,
    #[serde(default)]
    pub retry_count: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RequestAttemptStatus {
    Waiting {
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    },
    Pending {
        since: DateTime<Utc>,
    },
    InProgress {
        since: DateTime<Utc>,
    },
    Successful {
        at: DateTime<Utc>,
        full_processing_ms: i64,
    },
    Failed {
        at: DateTime<Utc>,
        full_processing_ms: i64,
    },
}

impl RequestAttemptStatus {
    pub fn is_successful(&self) -> bool {
        matches!(self, Self::Successful { .. })
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Waiting { .. } => "Waiting",
            Self::Pending { .. } => "Pending",
            Self::InProgress { .. } => "In Progress",
            Self::Successful { .. } => "Successful",
            Self::Failed { .. } => "Failed",
        }
    }
}

// =============================================================================
// Response (Webhook Response)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub response_id: Uuid,
    #[serde(default)]
    pub http_code: Option<i32>,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub elapsed_time_ms: Option<i32>,
    #[serde(default)]
    pub response_error_name: Option<String>,
}

// =============================================================================
// Pagination
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub total: Option<i64>,
    #[serde(default)]
    pub page: Option<i32>,
    #[serde(default)]
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct PaginationParams {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

impl PaginationParams {
    pub fn new(page: Option<i32>, per_page: Option<i32>) -> Self {
        Self { page, per_page }
    }

    pub fn to_query_params(&self) -> Vec<(&str, String)> {
        let mut params = Vec::new();
        if let Some(page) = self.page {
            params.push(("page", page.to_string()));
        }
        if let Some(per_page) = self.per_page {
            params.push(("per_page", per_page.to_string()));
        }
        params
    }
}

// =============================================================================
// Filter Parameters
// =============================================================================

#[derive(Debug, Clone, Default)]
pub struct EventFilters {
    pub event_type: Option<String>,
    pub status: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub labels: HashMap<String, String>,
}

impl EventFilters {
    pub fn to_query_params(&self) -> Vec<(&str, String)> {
        let mut params = Vec::new();
        if let Some(ref event_type) = self.event_type {
            params.push(("event_type", event_type.clone()));
        }
        if let Some(ref status) = self.status {
            params.push(("status", status.clone()));
        }
        if let Some(since) = self.since {
            params.push(("since", since.to_rfc3339()));
        }
        if let Some(until) = self.until {
            params.push(("until", until.to_rfc3339()));
        }
        for (key, value) in &self.labels {
            params.push(("label", format!("{}={}", key, value)));
        }
        params
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Base64 encode a string
pub fn base64_encode(input: &str) -> String {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    STANDARD.encode(input.as_bytes())
}

/// Error type for base64 decoding that preserves UTF-8 error details
#[derive(Debug, thiserror::Error)]
pub enum Base64DecodeError {
    #[error("Base64 decode error: {0}")]
    DecodeError(#[from] base64::DecodeError),
    #[error("Invalid UTF-8: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

/// Base64 decode a string
pub fn base64_decode(input: &str) -> Result<String, Base64DecodeError> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    let bytes = STANDARD.decode(input)?;
    Ok(String::from_utf8(bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_parse() {
        let result = EventType::parse("user.account.created");
        assert!(result.is_some());
        let (service, resource, verb) = result.expect("should parse");
        assert_eq!(service, "user");
        assert_eq!(resource, "account");
        assert_eq!(verb, "created");
    }

    #[test]
    fn test_event_type_parse_invalid() {
        assert!(EventType::parse("invalid").is_none());
        assert!(EventType::parse("only.two").is_none());
        assert!(EventType::parse("").is_none());
    }

    #[test]
    fn test_event_type_full_name() {
        let et = EventType {
            service_name: "user".to_string(),
            resource_type_name: "account".to_string(),
            verb_name: "created".to_string(),
            event_type_name: None,
            created_at: None,
        };
        assert_eq!(et.full_name(), "user.account.created");
    }

    #[test]
    fn test_target_http() {
        let target = Target::http("https://example.com/webhook".to_string());
        let Target::Http {
            method,
            url,
            headers,
        } = target;
        assert_eq!(method, "POST");
        assert_eq!(url, "https://example.com/webhook");
        assert!(headers.is_empty());
    }

    #[test]
    fn test_request_attempt_status_display() {
        let status = RequestAttemptStatus::Successful {
            at: Utc::now(),
            full_processing_ms: 100,
        };
        assert_eq!(status.display_name(), "Successful");
        assert!(status.is_successful());
        assert!(!status.is_failed());
    }

    #[test]
    fn test_base64_encode_decode() {
        let original = "Hello, World!";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).expect("decode should work");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_event_post_new_json() {
        let app_id = Uuid::new_v4();
        let payload = serde_json::json!({"user_id": 123});
        let event = EventPost::new_json(
            app_id,
            "user.account.created".to_string(),
            payload,
            HashMap::new(),
        );

        assert_eq!(event.application_id, app_id);
        assert_eq!(event.event_type, "user.account.created");
        assert_eq!(event.payload_content_type, "application/json");
    }

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams::new(Some(2), Some(50));
        let query = params.to_query_params();
        assert_eq!(query.len(), 2);
    }

    #[test]
    fn test_event_filters() {
        let mut filters = EventFilters::default();
        filters.event_type = Some("user.account.created".to_string());
        filters
            .labels
            .insert("tenant_id".to_string(), "org123".to_string());

        let query = filters.to_query_params();
        assert!(query.len() >= 2);
    }
}
