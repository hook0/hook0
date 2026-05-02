//! HTTP client for Hook0 API

use crate::config::Config;
use crate::error::Hook0McpError;
use reqwest::{Client, StatusCode, header};
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

    /// Build a URL for an API path. Accepts paths that include a `?query`
    /// suffix (e.g. `/event_types?application_id=…`); the query is preserved
    /// rather than percent-encoded into the path component.
    fn url(&self, path: &str) -> Url {
        // Split off the optional query string before touching the path.
        // `Url::set_path` percent-encodes literal `?` in its argument, which
        // would corrupt callers like `get_paginated("/event_types?app=X")`.
        let (path_only, query_only) = match path.find('?') {
            Some(idx) => (&path[..idx], Some(&path[idx + 1..])),
            None => (path, None),
        };

        let mut url = self.base_url.clone();
        let base_path = url.path().trim_end_matches('/');
        let clean_path = path_only.trim_start_matches('/');

        // Ensure we're using the API v1 prefix
        let full_path = if clean_path.starts_with("api/v1/") {
            format!("{}/{}", base_path, clean_path)
        } else {
            format!("{}/api/v1/{}", base_path, clean_path)
        };

        url.set_path(&full_path);
        url.set_query(query_only);
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

    /// Execute a GET request and follow `Link: rel="next"` (RFC 8288)
    /// pagination until exhausted or a hard cap is hit. Returns the merged
    /// array as a `Value::Array`. When the cap is reached, wraps the result
    /// in an envelope `{ "items": [...], "_truncated": true, "_truncated_count": N,
    /// "_message": "..." }` so the LLM consumer knows to call again with a cursor.
    ///
    /// `cap` defaults to 1000 items. `?limit=100` is appended to the initial
    /// request only (subsequent links are full URLs that already carry it).
    pub async fn get_paginated(&self, path: &str) -> Result<Value, Hook0McpError> {
        const CAP: usize = 1000;
        const PAGE_LIMIT: usize = 100;
        let initial_url = {
            let mut u = self.url(path);
            // Append ?limit=100 unless caller already set one. This is a best-effort
            // hint to the API; out-of-range responses (HTTP 400) propagate as errors.
            let has_limit = u.query_pairs().any(|(k, _)| k == "limit");
            if !has_limit {
                u.query_pairs_mut()
                    .append_pair("limit", &PAGE_LIMIT.to_string());
            }
            u
        };
        let mut url = initial_url;
        let mut acc: Vec<Value> = Vec::new();
        let mut truncated = false;
        loop {
            debug!("GET (paginated) {}", url);
            let response = self
                .client
                .get(url.clone())
                .bearer_auth(&self.token)
                .send()
                .await
                .map_err(Hook0McpError::Http)?;
            let link_header = response
                .headers()
                .get(reqwest::header::LINK)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_owned());
            let page_value = self.handle_response(response).await?;
            match page_value {
                Value::Array(rows) => {
                    for row in rows {
                        if acc.len() >= CAP {
                            truncated = true;
                            break;
                        }
                        acc.push(row);
                    }
                }
                // Non-array body: just return as-is. Shouldn't happen for the list
                // endpoints we paginate, but stay defensive.
                other => return Ok(other),
            }
            if truncated {
                break;
            }
            match link_header.as_deref().and_then(parse_next_link) {
                Some(next) => match url::Url::parse(&next) {
                    Ok(parsed) => url = parsed,
                    Err(e) => {
                        warn!(
                            "MCP cursor follow: failed to parse next URL '{}': {}",
                            next, e
                        );
                        break;
                    }
                },
                None => break,
            }
        }

        if truncated {
            Ok(serde_json::json!({
                "items": acc,
                "_truncated": true,
                "_truncated_count": CAP,
                "_message": format!(
                    "Result truncated at {CAP} items. The remote tenant has more rows; \
                     follow the API's Link header directly with `pagination_cursor` to fetch beyond this cap."
                ),
            }))
        } else {
            Ok(Value::Array(acc))
        }
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

/// Parses an RFC 8288 `Link` header and returns the URI for `rel="next"` if present.
fn parse_next_link(header: &str) -> Option<String> {
    for part in header.split(',') {
        let part = part.trim();
        let close_bracket = part.find('>')?;
        if !part.starts_with('<') {
            continue;
        }
        let uri = &part[1..close_bracket];
        let rest = &part[close_bracket + 1..];
        let rest_lower = rest.to_lowercase();
        if rest_lower.contains("rel=\"next\"") || rest_lower.contains("rel=next") {
            return Some(uri.to_owned());
        }
    }
    None
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

    #[test]
    fn url_preserves_query_string() {
        // Regression: get_paginated() and server.rs handlers pass paths shaped
        // as `/event_types?application_id=…`. Naively running set_path() on
        // such a string percent-encodes the `?` into `%3F`, producing a
        // malformed URL that the API answers with 404. The url() helper must
        // split path/query before set_path().
        let config = Config {
            api_url: Url::parse("https://app.hook0.com").unwrap(),
            api_token: "test-token".to_string(),
            transport: crate::config::Transport::Stdio,
            read_only: false,
        };
        let client = Hook0Client::new(&config).unwrap();

        let url = client.url("/event_types?application_id=11111111-1111-1111-1111-111111111111");
        assert_eq!(url.path(), "/api/v1/event_types");
        assert_eq!(
            url.query(),
            Some("application_id=11111111-1111-1111-1111-111111111111"),
        );
        assert!(
            !url.as_str().contains("%3F"),
            "raw `?` must not be percent-encoded into the path: {url}"
        );

        // Same contract for the already-prefixed form.
        let url = client.url("/api/v1/subscriptions?application_id=foo");
        assert_eq!(url.path(), "/api/v1/subscriptions");
        assert_eq!(url.query(), Some("application_id=foo"));
    }

    #[test]
    fn parse_next_link_simple() {
        let h = r#"<https://api/x?cursor=N>; rel="next""#;
        assert_eq!(parse_next_link(h).unwrap(), "https://api/x?cursor=N");
    }

    #[test]
    fn parse_next_link_combined() {
        let h = r#"<https://api/x?cursor=P>; rel="prev", <https://api/x?cursor=N>; rel="next""#;
        assert_eq!(parse_next_link(h).unwrap(), "https://api/x?cursor=N");
    }

    #[test]
    fn parse_next_link_only_prev() {
        let h = r#"<https://api/x?cursor=P>; rel="prev""#;
        assert!(parse_next_link(h).is_none());
    }

    #[test]
    fn parse_next_link_malformed() {
        assert!(parse_next_link("garbage").is_none());
        assert!(parse_next_link("").is_none());
    }
}
