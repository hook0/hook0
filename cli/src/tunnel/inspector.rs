use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::forwarder::ForwardResult;

/// Status of a request
#[derive(Debug, Clone, PartialEq)]
pub enum RequestStatus {
    /// Request received, waiting to be forwarded
    Pending,
    /// Request is being forwarded
    Forwarding,
    /// Request forwarded successfully
    Success {
        status_code: u16,
        elapsed_ms: i64,
    },
    /// Request forwarding failed
    Failed {
        error: String,
        elapsed_ms: i64,
    },
}

impl RequestStatus {
    pub fn display_status(&self) -> String {
        match self {
            RequestStatus::Pending => "Pending".to_string(),
            RequestStatus::Forwarding => "Forwarding".to_string(),
            RequestStatus::Success { status_code, .. } => format!("{} OK", status_code),
            RequestStatus::Failed { .. } => "Failed".to_string(),
        }
    }

    pub fn elapsed_ms(&self) -> Option<i64> {
        match self {
            RequestStatus::Success { elapsed_ms, .. } => Some(*elapsed_ms),
            RequestStatus::Failed { elapsed_ms, .. } => Some(*elapsed_ms),
            _ => None,
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, RequestStatus::Success { .. })
    }

    pub fn is_error(&self) -> bool {
        matches!(self, RequestStatus::Failed { .. })
    }
}

/// An inspected/logged request
#[derive(Debug, Clone)]
pub struct InspectedRequest {
    pub request_id: String,
    pub event_id: Uuid,
    pub event_type: String,
    pub payload: String,
    pub headers: HashMap<String, String>,
    pub received_at: DateTime<Utc>,
    pub status: RequestStatus,
    pub response_headers: Option<HashMap<String, String>>,
    pub response_body: Option<String>,
    pub labels: HashMap<String, String>,
}

impl InspectedRequest {
    pub fn new(
        request_id: String,
        event_id: Uuid,
        event_type: String,
        payload: String,
        headers: HashMap<String, String>,
        received_at: DateTime<Utc>,
    ) -> Self {
        Self {
            request_id,
            event_id,
            event_type,
            payload,
            headers,
            received_at,
            status: RequestStatus::Pending,
            response_headers: None,
            response_body: None,
            labels: HashMap::new(),
        }
    }

    pub fn update_from_result(&mut self, result: &ForwardResult) {
        if let Some(error) = &result.error {
            self.status = RequestStatus::Failed {
                error: error.clone(),
                elapsed_ms: result.elapsed_ms,
            };
        } else {
            self.status = RequestStatus::Success {
                status_code: result.status_code,
                elapsed_ms: result.elapsed_ms,
            };
        }
        self.response_headers = Some(result.headers.clone());
        self.response_body = result.body.clone();
    }
}

/// Inspector for storing and managing request history
pub struct Inspector {
    requests: Arc<RwLock<Vec<InspectedRequest>>>,
    max_requests: usize,
}

impl Inspector {
    /// Create a new inspector with the specified max history size
    pub fn new(max_requests: usize) -> Self {
        Self {
            requests: Arc::new(RwLock::new(Vec::with_capacity(max_requests))),
            max_requests,
        }
    }

    /// Create a new inspector with default max size (1000 requests)
    pub fn default_capacity() -> Self {
        Self::new(1000)
    }

    /// Add a new request
    pub fn add(&self, request: InspectedRequest) {
        let mut requests = self.requests.write().expect("lock poisoned");

        // Remove oldest if at capacity
        if requests.len() >= self.max_requests {
            requests.remove(0);
        }

        requests.push(request);
    }

    /// Update a request by ID
    pub fn update<F>(&self, request_id: &str, f: F)
    where
        F: FnOnce(&mut InspectedRequest),
    {
        let mut requests = self.requests.write().expect("lock poisoned");

        if let Some(req) = requests.iter_mut().find(|r| r.request_id == request_id) {
            f(req);
        }
    }

    /// Get a request by ID
    pub fn get(&self, request_id: &str) -> Option<InspectedRequest> {
        let requests = self.requests.read().expect("lock poisoned");
        requests.iter().find(|r| r.request_id == request_id).cloned()
    }

    /// Get all requests (newest first)
    pub fn list(&self) -> Vec<InspectedRequest> {
        let requests = self.requests.read().expect("lock poisoned");
        let mut result = requests.clone();
        result.reverse();
        result
    }

    /// Get recent requests (newest first)
    pub fn recent(&self, limit: usize) -> Vec<InspectedRequest> {
        let requests = self.requests.read().expect("lock poisoned");
        let start = requests.len().saturating_sub(limit);
        let mut result: Vec<_> = requests[start..].to_vec();
        result.reverse();
        result
    }

    /// Get count of requests
    pub fn count(&self) -> usize {
        self.requests.read().expect("lock poisoned").len()
    }

    /// Get count by status
    pub fn count_by_status(&self) -> (usize, usize, usize) {
        let requests = self.requests.read().expect("lock poisoned");
        let success = requests.iter().filter(|r| r.status.is_success()).count();
        let failed = requests.iter().filter(|r| r.status.is_error()).count();
        let pending = requests.len() - success - failed;
        (success, failed, pending)
    }

    /// Clear all requests
    pub fn clear(&self) {
        let mut requests = self.requests.write().expect("lock poisoned");
        requests.clear();
    }

    /// Get a clone of the underlying Arc for sharing
    pub fn shared(&self) -> Arc<RwLock<Vec<InspectedRequest>>> {
        Arc::clone(&self.requests)
    }
}

impl Default for Inspector {
    fn default() -> Self {
        Self::default_capacity()
    }
}

impl Clone for Inspector {
    fn clone(&self) -> Self {
        Self {
            requests: Arc::clone(&self.requests),
            max_requests: self.max_requests,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_request(id: &str) -> InspectedRequest {
        InspectedRequest::new(
            id.to_string(),
            Uuid::new_v4(),
            "test.event".to_string(),
            "payload".to_string(),
            HashMap::new(),
            Utc::now(),
        )
    }

    #[test]
    fn test_inspector_add_and_get() {
        let inspector = Inspector::new(10);
        let request = create_test_request("req-1");

        inspector.add(request.clone());

        let retrieved = inspector.get("req-1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.expect("should exist").request_id, "req-1");
    }

    #[test]
    fn test_inspector_max_capacity() {
        let inspector = Inspector::new(3);

        inspector.add(create_test_request("req-1"));
        inspector.add(create_test_request("req-2"));
        inspector.add(create_test_request("req-3"));
        inspector.add(create_test_request("req-4"));

        assert_eq!(inspector.count(), 3);
        assert!(inspector.get("req-1").is_none());
        assert!(inspector.get("req-4").is_some());
    }

    #[test]
    fn test_inspector_update() {
        let inspector = Inspector::new(10);
        let request = create_test_request("req-1");

        inspector.add(request);

        inspector.update("req-1", |r| {
            r.status = RequestStatus::Success {
                status_code: 200,
                elapsed_ms: 50,
            };
        });

        let updated = inspector.get("req-1").expect("should exist");
        assert!(updated.status.is_success());
    }

    #[test]
    fn test_inspector_recent() {
        let inspector = Inspector::new(10);

        inspector.add(create_test_request("req-1"));
        inspector.add(create_test_request("req-2"));
        inspector.add(create_test_request("req-3"));

        let recent = inspector.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].request_id, "req-3");
        assert_eq!(recent[1].request_id, "req-2");
    }

    #[test]
    fn test_request_status_display() {
        let pending = RequestStatus::Pending;
        assert_eq!(pending.display_status(), "Pending");

        let success = RequestStatus::Success {
            status_code: 200,
            elapsed_ms: 50,
        };
        assert_eq!(success.display_status(), "200 OK");
        assert!(success.is_success());

        let failed = RequestStatus::Failed {
            error: "Connection refused".to_string(),
            elapsed_ms: 100,
        };
        assert_eq!(failed.display_status(), "Failed");
        assert!(failed.is_error());
    }

    #[test]
    fn test_count_by_status() {
        let inspector = Inspector::new(10);

        let mut req1 = create_test_request("req-1");
        req1.status = RequestStatus::Success { status_code: 200, elapsed_ms: 50 };
        inspector.add(req1);

        let mut req2 = create_test_request("req-2");
        req2.status = RequestStatus::Failed { error: "error".to_string(), elapsed_ms: 100 };
        inspector.add(req2);

        inspector.add(create_test_request("req-3")); // Pending

        let (success, failed, pending) = inspector.count_by_status();
        assert_eq!(success, 1);
        assert_eq!(failed, 1);
        assert_eq!(pending, 1);
    }
}
