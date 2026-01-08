//! Integration tests for Hook0 MCP Server
//!
//! These tests are BLACKBOX tests - they test the MCP server by spawning it as a
//! subprocess and communicating via the MCP protocol (JSON-RPC over stdio).
//!
//! Tests run against the production Hook0 API using MCP_SERVICE_TOKEN.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

fn next_request_id() -> u64 {
    REQUEST_ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: &'static str,
    params: Value,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcError {
    code: i64,
    message: String,
    #[serde(default)]
    data: Option<Value>,
}

/// A handle to the MCP server process
struct McpServerProcess {
    child: Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
}

impl McpServerProcess {
    /// Start the MCP server process
    fn start() -> Self {
        // Get the token from environment
        let token = env::var("MCP_SERVICE_TOKEN")
            .or_else(|_| env::var("HOOK0_API_TOKEN"))
            .expect("MCP_SERVICE_TOKEN or HOOK0_API_TOKEN must be set");

        let api_url =
            env::var("HOOK0_API_URL").unwrap_or_else(|_| "https://app.hook0.com".to_string());

        // Find the binary - it's in the workspace target directory
        let binary = env::var("CARGO_BIN_EXE_hook0-mcp")
            .unwrap_or_else(|_| "../../target/debug/hook0-mcp".to_string());

        let mut child = Command::new(binary)
            .env("HOOK0_API_TOKEN", token)
            .env("HOOK0_API_URL", api_url)
            .env("MCP_TRANSPORT", "stdio")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start MCP server");

        let stdin = child.stdin.take().expect("Failed to get stdin");
        let stdout = child.stdout.take().expect("Failed to get stdout");
        let reader = BufReader::new(stdout);

        Self {
            child,
            stdin,
            reader,
        }
    }

    /// Send a JSON-RPC request and receive response
    fn send_request(&mut self, method: &'static str, params: Value) -> JsonRpcResponse {
        let id = next_request_id();
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method,
            params,
        };

        let request_json = serde_json::to_string(&request).expect("Failed to serialize request");

        // Write request
        writeln!(self.stdin, "{}", request_json).expect("Failed to write request");
        self.stdin.flush().expect("Failed to flush");

        // Read response
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .expect("Failed to read response");

        serde_json::from_str(&line).expect("Failed to parse response")
    }

    /// Send initialize request (required by MCP protocol)
    fn initialize(&mut self) -> JsonRpcResponse {
        let response = self.send_request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }),
        );

        // Send initialized notification (no response expected)
        self.send_notification("notifications/initialized", json!({}));

        response
    }

    /// Send a JSON-RPC notification (no response expected)
    fn send_notification(&mut self, method: &'static str, params: Value) {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        let notification_json =
            serde_json::to_string(&notification).expect("Failed to serialize notification");

        writeln!(self.stdin, "{}", notification_json).expect("Failed to write notification");
        self.stdin.flush().expect("Failed to flush");
    }
}

impl Drop for McpServerProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// =============================================================================
// Server Tests
// =============================================================================

mod server {
    use super::*;

    #[test]
    fn test_server_info() {
        let mut server = McpServerProcess::start();

        let response = server.initialize();

        let result = response.result.expect("Initialize should return a result");
        let server_info = result
            .get("serverInfo")
            .expect("Result should have serverInfo");

        let name = server_info
            .get("name")
            .expect("serverInfo should have name")
            .as_str()
            .expect("name should be string");
        assert_eq!(name, "hook0-mcp");

        let version = server_info
            .get("version")
            .expect("serverInfo should have version")
            .as_str()
            .expect("version should be string");
        assert!(
            version.len() >= 5,
            "Version should be meaningful (e.g. '1.0.0'), got '{}'",
            version
        );
    }

    #[test]
    fn test_server_capabilities() {
        let mut server = McpServerProcess::start();

        let response = server.initialize();

        let result = response.result.expect("Initialize should return a result");
        let capabilities = result
            .get("capabilities")
            .expect("Result should have capabilities");

        // Verify all three capability types exist
        capabilities
            .get("tools")
            .expect("Should have tools capability");
        capabilities
            .get("resources")
            .expect("Should have resources capability");
        capabilities
            .get("prompts")
            .expect("Should have prompts capability");
    }
}

// =============================================================================
// Tools Tests
// =============================================================================

mod tools {
    use super::*;

    #[test]
    fn test_list_tools() {
        let mut server = McpServerProcess::start();
        server.initialize();

        let response = server.send_request("tools/list", json!({}));

        let result = response.result.expect("list_tools should return a result");
        let tools = result
            .get("tools")
            .expect("Result should have tools")
            .as_array()
            .expect("tools should be an array");

        // Tools are generated from OpenAPI spec at build time.
        // If no tools are available, the build is broken - fail fast!
        assert!(
            !tools.is_empty(),
            "No tools available! OpenAPI spec may not be accessible at build time. \
             Check that HOOK0_API_URL is reachable during build or verify build.rs fallback. \
             NEVER silently skip - fix the build configuration."
        );

        // Should have at least the core tools (list_organizations, list_applications, etc.)
        assert!(
            tools.len() >= 5,
            "Should have at least 5 tools, got {}. \
             OpenAPI spec may be incomplete or build.rs filtering is too aggressive.",
            tools.len()
        );

        // Verify each tool has required fields with meaningful content
        for tool in tools {
            let name = tool
                .get("name")
                .expect("Tool should have name")
                .as_str()
                .expect("name should be string");
            assert!(
                name.len() >= 3,
                "Tool name should be meaningful, got '{}'",
                name
            );

            let description = tool
                .get("description")
                .expect("Tool should have description")
                .as_str()
                .expect("description should be string");
            assert!(
                description.len() >= 10,
                "Tool description should be meaningful, got '{}'",
                description
            );
        }
    }

    #[test]
    fn test_call_unknown_tool() {
        let mut server = McpServerProcess::start();
        server.initialize();

        let response = server.send_request(
            "tools/call",
            json!({
                "name": "nonexistent_tool",
                "arguments": {}
            }),
        );

        let error = response
            .error
            .expect("Should return error for unknown tool");
        assert!(
            error.message.to_lowercase().contains("unknown")
                || error.message.to_lowercase().contains("not found")
                || error.code != 0,
            "Error should indicate unknown tool: {}",
            error.message
        );
    }
}

// =============================================================================
// Resources Tests
// =============================================================================

mod resources {
    use super::*;

    #[test]
    fn test_list_resources() {
        let mut server = McpServerProcess::start();
        server.initialize();

        let response = server.send_request("resources/list", json!({}));

        let result = response
            .result
            .expect("list_resources should return a result");
        let resources = result
            .get("resources")
            .expect("Result should have resources")
            .as_array()
            .expect("resources should be an array");

        // Should have at least organizations and applications resources
        assert!(
            resources.len() >= 2,
            "Should have at least 2 resources, got {}",
            resources.len()
        );

        // Verify each resource has required fields
        for resource in resources {
            let uri = resource
                .get("uri")
                .expect("Resource should have uri")
                .as_str()
                .expect("uri should be string");
            assert!(
                uri.starts_with("hook0://"),
                "Resource URI should start with 'hook0://', got '{}'",
                uri
            );

            let name = resource
                .get("name")
                .expect("Resource should have name")
                .as_str()
                .expect("name should be string");
            assert!(
                name.len() >= 3,
                "Resource name should be meaningful, got '{}'",
                name
            );
        }
    }

    #[test]
    fn test_read_organizations_resource() {
        let mut server = McpServerProcess::start();
        server.initialize();

        let response = server.send_request(
            "resources/read",
            json!({
                "uri": "hook0://organizations"
            }),
        );

        let result = response
            .result
            .expect("read organizations should return a result");
        let contents = result
            .get("contents")
            .expect("Result should have contents")
            .as_array()
            .expect("contents should be an array");

        assert!(
            !contents.is_empty(),
            "Should have at least one content entry"
        );

        // Verify content structure
        let first_content = &contents[0];
        let uri = first_content
            .get("uri")
            .expect("Content should have uri")
            .as_str()
            .expect("uri should be string");
        assert!(
            uri.contains("organizations"),
            "Content URI should reference organizations, got '{}'",
            uri
        );

        // Should have text or blob content
        let has_text = first_content.get("text").is_some();
        let has_blob = first_content.get("blob").is_some();
        assert!(
            has_text || has_blob,
            "Content should have either text or blob"
        );
    }

    #[test]
    fn test_read_applications_resource() {
        let mut server = McpServerProcess::start();
        server.initialize();

        let response = server.send_request(
            "resources/read",
            json!({
                "uri": "hook0://applications"
            }),
        );

        // Applications resource may return an error if there are no applications
        // or return a result with empty/non-empty contents
        if let Some(error) = &response.error {
            // It's acceptable to have an error if there are no applications
            assert!(
                error.message.contains("not found")
                    || error.message.contains("no applications")
                    || error.code != 0,
                "Error should be meaningful: {}",
                error.message
            );
        } else {
            let result = response.result.expect("Should have result if no error");
            let contents = result
                .get("contents")
                .expect("Result should have contents")
                .as_array()
                .expect("contents should be an array");

            // Contents array validated above (as_array succeeds)
            // Empty is valid if no applications exist
        }
    }

    #[test]
    fn test_read_unknown_resource() {
        let mut server = McpServerProcess::start();
        server.initialize();

        let response = server.send_request(
            "resources/read",
            json!({
                "uri": "hook0://unknown/resource"
            }),
        );

        let error = response
            .error
            .expect("Should return error for unknown resource");
        assert!(
            error.code != 0 || !error.message.is_empty(),
            "Error should have code or message"
        );
    }
}

// =============================================================================
// Prompts Tests
// =============================================================================

mod prompts {
    use super::*;

    #[test]
    fn test_list_prompts() {
        let mut server = McpServerProcess::start();
        server.initialize();

        let response = server.send_request("prompts/list", json!({}));

        let result = response
            .result
            .expect("list_prompts should return a result");
        let prompts = result
            .get("prompts")
            .expect("Result should have prompts")
            .as_array()
            .expect("prompts should be an array");

        // Should have at least the 3 documented prompts
        assert!(
            prompts.len() >= 3,
            "Should have at least 3 prompts, got {}",
            prompts.len()
        );

        // Verify each prompt has required fields
        for prompt in prompts {
            let name = prompt
                .get("name")
                .expect("Prompt should have name")
                .as_str()
                .expect("name should be string");
            assert!(
                name.len() >= 5,
                "Prompt name should be meaningful, got '{}'",
                name
            );

            let description = prompt
                .get("description")
                .expect("Prompt should have description")
                .as_str()
                .expect("description should be string");
            assert!(
                description.len() >= 10,
                "Prompt description should be meaningful, got '{}'",
                description
            );
        }
    }

    #[test]
    fn test_get_create_webhook_subscription_prompt() {
        let mut server = McpServerProcess::start();
        server.initialize();

        let response = server.send_request(
            "prompts/get",
            json!({
                "name": "create_webhook_subscription"
            }),
        );

        let result = response.result.expect("get prompt should return a result");
        let messages = result
            .get("messages")
            .expect("Result should have messages")
            .as_array()
            .expect("messages should be an array");

        assert!(!messages.is_empty(), "Should have at least one message");

        // Verify message structure
        let first_message = &messages[0];
        let role = first_message
            .get("role")
            .expect("Message should have role")
            .as_str()
            .expect("role should be string");
        assert!(
            role == "user" || role == "assistant",
            "Role should be 'user' or 'assistant', got '{}'",
            role
        );

        let content = first_message
            .get("content")
            .expect("Message should have content");
        // Content can be string or structured
        let has_text =
            content.is_string() || content.get("text").map(|t| t.is_string()).unwrap_or(false);
        assert!(has_text, "Message content should have text");
    }

    #[test]
    fn test_get_debug_event_delivery_prompt() {
        let mut server = McpServerProcess::start();
        server.initialize();

        let response = server.send_request(
            "prompts/get",
            json!({
                "name": "debug_event_delivery"
            }),
        );

        let result = response.result.expect("get prompt should return a result");
        let messages = result
            .get("messages")
            .expect("Result should have messages")
            .as_array()
            .expect("messages should be an array");

        assert!(!messages.is_empty(), "Should have at least one message");
    }

    #[test]
    fn test_get_setup_application_prompt() {
        let mut server = McpServerProcess::start();
        server.initialize();

        let response = server.send_request(
            "prompts/get",
            json!({
                "name": "setup_application"
            }),
        );

        let result = response.result.expect("get prompt should return a result");
        let messages = result
            .get("messages")
            .expect("Result should have messages")
            .as_array()
            .expect("messages should be an array");

        assert!(!messages.is_empty(), "Should have at least one message");
    }

    #[test]
    fn test_get_unknown_prompt() {
        let mut server = McpServerProcess::start();
        server.initialize();

        let response = server.send_request(
            "prompts/get",
            json!({
                "name": "nonexistent_prompt"
            }),
        );

        let error = response
            .error
            .expect("Should return error for unknown prompt");
        assert!(
            error.code != 0 || !error.message.is_empty(),
            "Error should have code or message"
        );
    }
}
