//! Build script for hook0-mcp
//!
//! This script fetches the OpenAPI spec from Hook0 at compile time
//! and generates metadata for documentation purposes.
//! Only operations tagged with "mcp" are included.

use openapiv3::OpenAPI;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::PathBuf;

const OPENAPI_URL: &str = "https://app.hook0.com/api/v1/swagger.json";

/// Operation metadata extracted from OpenAPI
#[derive(Debug, Serialize)]
struct OperationMeta {
    operation_id: String,
    method: String,
    path: String,
    summary: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
}

fn main() {
    println!("cargo:rerun-if-env-changed=HOOK0_OPENAPI_URL");
    println!("cargo:rerun-if-env-changed=SKIP_OPENAPI_FETCH");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

    // Skip fetching in CI or when explicitly requested
    let skip_fetch = env::var("SKIP_OPENAPI_FETCH").is_ok();

    if skip_fetch {
        println!("cargo:warning=Skipping OpenAPI fetch (SKIP_OPENAPI_FETCH is set)");
        write_empty_metadata(&out_dir);
        return;
    }

    // Fetch OpenAPI spec (supports HOOK0_OPENAPI_URL for local development)
    let openapi_url = env::var("HOOK0_OPENAPI_URL").unwrap_or_else(|_| OPENAPI_URL.to_string());

    let spec = match fetch_openapi_spec(&openapi_url) {
        Ok(spec) => spec,
        Err(e) => {
            println!(
                "cargo:warning=Failed to fetch OpenAPI spec: {}. Using empty metadata.",
                e
            );
            write_empty_metadata(&out_dir);
            return;
        }
    };

    // Extract operation metadata (only operations tagged with "mcp")
    let operations = extract_mcp_operations(&spec);

    // Write metadata file for documentation
    let metadata = serde_json::json!({
        "version": spec.info.version,
        "title": spec.info.title,
        "operations": operations,
    });

    let metadata_path = out_dir.join("mcp_metadata.json");
    fs::write(
        &metadata_path,
        serde_json::to_string_pretty(&metadata).expect("Failed to serialize"),
    )
    .expect("Failed to write mcp_metadata.json");

    println!(
        "cargo:warning=Generated MCP metadata with {} operations (filtered by 'mcp' tag)",
        operations.len()
    );
}

fn fetch_openapi_spec(url: &str) -> Result<OpenAPI, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let text = response
        .text()
        .map_err(|e| format!("Failed to read body: {}", e))?;
    serde_json::from_str(&text).map_err(|e| format!("Failed to parse JSON: {}", e))
}

/// Extract operations that have the "mcp" tag
fn extract_mcp_operations(spec: &OpenAPI) -> Vec<OperationMeta> {
    let mut operations = Vec::new();

    for (path, path_item) in &spec.paths.paths {
        let path_item = match path_item {
            openapiv3::ReferenceOr::Item(item) => item,
            openapiv3::ReferenceOr::Reference { .. } => continue,
        };

        for (method, operation) in [
            ("GET", path_item.get.as_ref()),
            ("POST", path_item.post.as_ref()),
            ("PUT", path_item.put.as_ref()),
            ("PATCH", path_item.patch.as_ref()),
            ("DELETE", path_item.delete.as_ref()),
        ] {
            let operation = match operation {
                Some(op) => op,
                None => continue,
            };

            let operation_id = match &operation.operation_id {
                Some(id) => id.clone(),
                None => continue,
            };

            // Only include operations with "mcp" tag
            let has_mcp_tag = operation.tags.iter().any(|t| t == "mcp");
            if !has_mcp_tag {
                continue;
            }

            operations.push(OperationMeta {
                operation_id,
                method: method.to_string(),
                path: path.clone(),
                summary: operation.summary.clone(),
                description: operation.description.clone(),
                tags: operation.tags.clone(),
            });
        }
    }

    operations.sort_by(|a, b| a.operation_id.cmp(&b.operation_id));
    operations
}

fn write_empty_metadata(out_dir: &std::path::Path) {
    let metadata = serde_json::json!({
        "version": "unknown",
        "title": "Hook0 API",
        "operations": [],
    });

    let metadata_path = out_dir.join("mcp_metadata.json");
    fs::write(
        &metadata_path,
        serde_json::to_string_pretty(&metadata).expect("Failed to serialize"),
    )
    .expect("Failed to write mcp_metadata.json");
}
