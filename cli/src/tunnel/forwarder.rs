use std::collections::HashMap;
use std::time::Instant;

use reqwest::Client;
use thiserror::Error;

use crate::api::models::base64_decode;

#[derive(Error, Debug)]
pub enum ForwardError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Connection refused: {0}")]
    ConnectionRefused(String),
}

/// Result of forwarding a webhook
#[derive(Debug, Clone)]
pub struct ForwardResult {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub elapsed_ms: i64,
    pub error: Option<String>,
}

/// Forwards webhooks to a local server
pub struct Forwarder {
    client: Client,
    target_url: String,
    #[allow(dead_code)]
    insecure: bool,
}

impl Forwarder {
    /// Create a new forwarder
    pub fn new(target_url: String, insecure: bool) -> Self {
        let client = Client::builder()
            .danger_accept_invalid_certs(insecure)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            target_url,
            insecure,
        }
    }

    /// Forward a webhook to the local server
    pub async fn forward(
        &self,
        payload: &str,
        headers: &HashMap<String, String>,
        event_type: &str,
    ) -> Result<ForwardResult, ForwardError> {
        let start = Instant::now();

        // Decode payload if base64 encoded
        let body = base64_decode(payload).unwrap_or_else(|_| payload.to_string());

        // Build request
        let mut req = self.client.post(&self.target_url);

        // Add headers
        for (key, value) in headers {
            req = req.header(key, value);
        }

        // Add event type header
        req = req.header("X-Hook0-Event-Type", event_type);

        // Add body
        req = req.body(body);

        // Send request
        let response = match req.send().await {
            Ok(resp) => resp,
            Err(e) => {
                let elapsed = start.elapsed().as_millis() as i64;

                if e.is_connect() {
                    return Ok(ForwardResult {
                        status_code: 0,
                        headers: HashMap::new(),
                        body: None,
                        elapsed_ms: elapsed,
                        error: Some(format!("Connection refused: {}", self.target_url)),
                    });
                }

                if e.is_timeout() {
                    return Ok(ForwardResult {
                        status_code: 0,
                        headers: HashMap::new(),
                        body: None,
                        elapsed_ms: elapsed,
                        error: Some("Request timeout".to_string()),
                    });
                }

                return Err(ForwardError::RequestError(e));
            }
        };

        let elapsed = start.elapsed().as_millis() as i64;
        let status_code = response.status().as_u16();

        // Extract response headers
        let response_headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        // Read response body
        let body = response.text().await.ok();

        Ok(ForwardResult {
            status_code,
            headers: response_headers,
            body,
            elapsed_ms: elapsed,
            error: None,
        })
    }

    /// Check if the target URL is reachable
    pub async fn health_check(&self) -> bool {
        match self.client.get(&self.target_url).send().await {
            Ok(resp) => {
                // Consider success if we got any response (even 4xx/5xx means server is up)
                resp.status().is_success() || resp.status().is_client_error() || resp.status().is_server_error()
            }
            Err(_) => false,
        }
    }

    /// Get the target URL
    pub fn target_url(&self) -> &str {
        &self.target_url
    }
}

/// Parse a target specification (port number or URL)
pub fn parse_target(target: &str) -> Result<String, ForwardError> {
    // Check if it's just a port number
    if let Ok(port) = target.parse::<u16>() {
        return Ok(format!("http://localhost:{}", port));
    }

    // Check if it's a valid URL
    if target.starts_with("http://") || target.starts_with("https://") {
        url::Url::parse(target)
            .map_err(|e| ForwardError::InvalidUrl(e.to_string()))?;
        return Ok(target.to_string());
    }

    // Assume it's a hostname:port or just a hostname
    // Check for host:port pattern (but not IPv6 literal like ::1)
    if target.contains(':') && !target.contains("::") && !target.starts_with('[') {
        Ok(format!("http://{}", target))
    } else if target.starts_with('[') {
        // Bracketed IPv6 like [::1]:8080
        Ok(format!("http://{}", target))
    } else {
        Ok(format!("http://{}:3000", target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_target_port() {
        assert_eq!(
            parse_target("3000").expect("should parse"),
            "http://localhost:3000"
        );
        assert_eq!(
            parse_target("8080").expect("should parse"),
            "http://localhost:8080"
        );
    }

    #[test]
    fn test_parse_target_url() {
        assert_eq!(
            parse_target("http://localhost:3000/webhook").expect("should parse"),
            "http://localhost:3000/webhook"
        );
        assert_eq!(
            parse_target("https://example.com/hook").expect("should parse"),
            "https://example.com/hook"
        );
    }

    #[test]
    fn test_parse_target_hostname() {
        assert_eq!(
            parse_target("localhost:3000").expect("should parse"),
            "http://localhost:3000"
        );
        assert_eq!(
            parse_target("myservice").expect("should parse"),
            "http://myservice:3000"
        );
    }

    #[test]
    fn test_forwarder_creation() {
        let forwarder = Forwarder::new("http://localhost:3000".to_string(), false);
        assert_eq!(forwarder.target_url(), "http://localhost:3000");
    }
}
