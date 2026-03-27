//! HTTP request forwarder - forwards webhook requests to local server

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::message::ServerRequestData;

/// Result of forwarding a request to the local server
#[derive(Debug)]
pub struct ForwardResult {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Vec<u8>,
    /// Time taken for the request
    pub duration: Duration,
}

/// Forward a webhook request to the local server
pub async fn forward_request(
    client: &reqwest::Client,
    target_url: &str,
    request: &ServerRequestData,
) -> Result<ForwardResult> {
    let start = Instant::now();

    // Build the full URL with path and query
    let mut url = format!("{}{}", target_url.trim_end_matches('/'), request.path);
    if let Some(ref query) = request.query {
        url = format!("{}?{}", url, query);
    }

    // Decode the body
    let body = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &request.body)
        .context("Failed to decode request body")?;

    // Build the request
    let method = request
        .method
        .parse::<reqwest::Method>()
        .unwrap_or(reqwest::Method::POST);

    let mut req_builder = client.request(method, &url);

    // Add headers
    for (name, value) in &request.headers {
        // Skip hop-by-hop headers
        if !is_hop_by_hop_header(name) {
            req_builder = req_builder.header(name, value);
        }
    }

    // Send request
    let response = req_builder
        .body(body)
        .send()
        .await
        .context("Failed to send request to local server")?;

    let status = response.status().as_u16();

    // Collect response headers
    let headers: HashMap<String, String> = response
        .headers()
        .iter()
        .filter(|(name, _)| !is_hop_by_hop_header(name.as_str()))
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|v| (name.to_string(), v.to_string()))
        })
        .collect();

    // Get response body
    let body = response
        .bytes()
        .await
        .context("Failed to read response body")?
        .to_vec();

    Ok(ForwardResult {
        status,
        headers,
        body,
        duration: start.elapsed(),
    })
}

/// Check if a header is a hop-by-hop header that shouldn't be forwarded
fn is_hop_by_hop_header(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    matches!(
        name_lower.as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailers"
            | "transfer-encoding"
            | "upgrade"
            | "host"
    )
}
