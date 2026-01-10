//! HTTP client for Hook0 API

use crate::config::Config;
use crate::error::Hook0McpError;
use reqwest::{header, Client, StatusCode};
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, warn};
use url::Url;

/// HTTP client for Hook0 API
#[derive(Debug, Clone)]
pub struct Hook0Client {
    client: Client,
    base_url: Url,
    token: String,
}

impl Hook0Client {
    /// Create a new Hook0 client from configuration
    pub fn new(config: &Config) -> Result<Self, Hook0McpError> {
        let mut headers = header::HeaderMap::new();

        // Set default headers
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        // Create HTTP client
        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(Hook0McpError::Http)?;

        Ok(Self {
            client,
            base_url: config.api_url.clone(),
            token: config.api_token.clone(),
        })
    }

    /// Build a URL for an API path
    fn url(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        let base_path = url.path().trim_end_matches('/');
        let clean_path = path.trim_start_matches('/');

        // Ensure we're using the API v1 prefix
        let full_path = if clean_path.starts_with("api/v1/") {
            format!("{}/{}", base_path, clean_path)
        } else {
            format!("{}/api/v1/{}", base_path, clean_path)
        };

        url.set_path(&full_path);
        url
    }

    /// Execute a GET request
    pub async fn get(&self, path: &str) -> Result<Value, Hook0McpError> {
        let url = self.url(path);
        debug!("GET {}", url);

        let response = self
            .client
            .get(url.clone())
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(Hook0McpError::Http)?;

        self.handle_response(response).await
    }

    /// Execute a POST request
    pub async fn post(&self, path: &str, body: Option<Value>) -> Result<Value, Hook0McpError> {
        let url = self.url(path);
        debug!("POST {}", url);

        let mut request = self.client.post(url.clone()).bearer_auth(&self.token);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await.map_err(Hook0McpError::Http)?;

        self.handle_response(response).await
    }

    /// Execute a PUT request
    pub async fn put(&self, path: &str, body: Option<Value>) -> Result<Value, Hook0McpError> {
        let url = self.url(path);
        debug!("PUT {}", url);

        let mut request = self.client.put(url.clone()).bearer_auth(&self.token);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await.map_err(Hook0McpError::Http)?;

        self.handle_response(response).await
    }

    /// Execute a PATCH request
    pub async fn patch(&self, path: &str, body: Option<Value>) -> Result<Value, Hook0McpError> {
        let url = self.url(path);
        debug!("PATCH {}", url);

        let mut request = self.client.patch(url.clone()).bearer_auth(&self.token);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await.map_err(Hook0McpError::Http)?;

        self.handle_response(response).await
    }

    /// Execute a DELETE request
    pub async fn delete(&self, path: &str) -> Result<Value, Hook0McpError> {
        let url = self.url(path);
        debug!("DELETE {}", url);

        let response = self
            .client
            .delete(url.clone())
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(Hook0McpError::Http)?;

        self.handle_response(response).await
    }

    /// Handle HTTP response
    async fn handle_response(&self, response: reqwest::Response) -> Result<Value, Hook0McpError> {
        let status = response.status();
        let url = response.url().clone();

        if status.is_success() {
            // Handle empty responses (204 No Content)
            if status == StatusCode::NO_CONTENT {
                return Ok(Value::Null);
            }

            // Try to parse JSON response
            let text = response.text().await.map_err(Hook0McpError::Http)?;
            if text.is_empty() {
                return Ok(Value::Null);
            }

            serde_json::from_str(&text).map_err(|e| {
                warn!("Failed to parse response as JSON: {}", e);
                Hook0McpError::Json(e)
            })
        } else {
            // Try to extract error message from response
            let error_body = response.text().await.unwrap_or_default();
            let message = if let Ok(json) = serde_json::from_str::<Value>(&error_body) {
                json.get("message")
                    .or_else(|| json.get("error"))
                    .or_else(|| json.get("detail"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(&error_body)
                    .to_string()
            } else {
                error_body
            };

            warn!("API error: {} {} - {}", status.as_u16(), url, message);

            Err(Hook0McpError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_building() {
        let config = Config {
            api_url: Url::parse("https://app.hook0.com").unwrap(),
            api_token: "test-token".to_string(),
            transport: crate::config::Transport::Stdio,
            read_only: false,
        };

        let client = Hook0Client::new(&config).unwrap();

        // Should add api/v1 prefix
        let url = client.url("/applications");
        assert_eq!(url.as_str(), "https://app.hook0.com/api/v1/applications");

        // Should not duplicate api/v1 prefix
        let url = client.url("/api/v1/applications");
        assert_eq!(url.as_str(), "https://app.hook0.com/api/v1/applications");
    }
}
