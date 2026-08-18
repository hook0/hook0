//! Integration tests for Hook0 MCP Server
//!
//! These tests are BLACKBOX tests - they test the MCP server by spawning it as a
//! subprocess and communicating via the MCP protocol (JSON-RPC over stdio).
//!
//! Most tests run against a Hook0 API using MCP_SERVICE_TOKEN and are `#[ignore]`d
//! so they only run where one is available. The `protocol` module is the exception:
//! it covers the MCP handshake, which never reaches the API, and always runs.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

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
    /// Start the MCP server process using credentials from the environment
    fn start() -> Self {
        // Get the token from environment
        let token = env::var("MCP_SERVICE_TOKEN")
            .or_else(|_| env::var("HOOK0_API_TOKEN"))
            .expect("MCP_SERVICE_TOKEN or HOOK0_API_TOKEN must be set");

        let api_url =
            env::var("HOOK0_API_URL").unwrap_or_else(|_| "https://app.hook0.com".to_string());

        Self::spawn(&token, &api_url)
    }

    /// Start the MCP server with a placeholder token.
    ///
    /// Protocol-level requests (`initialize`, `tools/list`) never reach the Hook0
    /// API, and the server only checks at startup that the token is non-empty, so
    /// these tests need neither credentials nor a reachable API.
    fn start_without_credentials() -> Self {
        Self::spawn("placeholder-token-never-used", "https://app.hook0.com")
    }

    fn spawn(token: &str, api_url: &str) -> Self {
        // Find the binary - it's in the workspace target directory
        let binary = env::var("CARGO_BIN_EXE_hook0-mcp")
            .unwrap_or_else(|_| "../../target/debug/hook0-mcp".to_string());

        let mut child = Command::new(binary)
            .env("HOOK0_API_TOKEN", token)
            .env("HOOK0_API_URL", api_url)
            .env("MCP_TRANSPORT", "stdio")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Print to test output to prevent pipe buffer blocking
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

        // Read response with timeout to prevent hanging forever
        let timeout = Duration::from_secs(30);
        let mut line = String::new();

        let result = std::thread::scope(|s| {
            let reader = &mut self.reader;
            let line_ref = &mut line;
            let handle = s.spawn(move || reader.read_line(line_ref));

            // Wait for thread with timeout
            let start = std::time::Instant::now();
            while !handle.is_finished() {
                if start.elapsed() > timeout {
                    panic!(
                        "Timeout waiting for response from MCP server after {:?}",
                        timeout
                    );
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            handle.join().expect("Reader thread panicked")
        });

        result.expect("Failed to read response");
        serde_json::from_str(&line).expect("Failed to parse response")
    }

    /// Send initialize request (required by MCP protocol)
    fn initialize(&mut self) -> JsonRpcResponse {
        self.initialize_with_version("2024-11-05")
    }

    /// Send initialize request requesting a specific protocol version
    fn initialize_with_version(&mut self, protocol_version: &str) -> JsonRpcResponse {
        let response = self.send_request(
            "initialize",
            json!({
                "protocolVersion": protocol_version,
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
// Protocol Negotiation Tests
// =============================================================================

/// These tests exercise the MCP protocol handshake only, so unlike the rest of
/// this file they need no credentials and no reachable Hook0 API, and are
/// therefore not `#[ignore]`d.
mod protocol {
    use super::*;
    use rmcp::model::ProtocolVersion;

    /// Where the README states which revisions this server answers on.
    const REVISIONS_HEADING: &str = "## Protocol revisions";

    /// What `get_info()` advertises, and what a client asking for an unimplemented revision is
    /// answered with: the newest revision the README lists.
    ///
    /// Derived rather than stated, for the reason the list itself is derived — a server that
    /// advertised a revision its own README does not list would be sending clients towards
    /// something they cannot have, and a constant here saying otherwise would agree with it.
    fn advertised_revision() -> String {
        documented_revisions()
            .pop()
            .expect("documented_revisions refuses an empty list")
    }

    /// The revisions the README lists under [`REVISIONS_HEADING`], sorted.
    ///
    /// Read out of the file rather than restated here, because a copy of the list beside the one
    /// users read is a copy that goes stale silently — which is exactly what a client discovering
    /// the difference at `initialize` time would be paying for.
    ///
    /// Only whole bullets of the shape ``- `<revision>` `` count, so the prose around the list can
    /// name a revision without being read as claiming it.
    fn documented_revisions() -> Vec<String> {
        let readme = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md"),
        )
        .expect("the README this crate publishes");

        let mut listed: Vec<String> = readme
            .lines()
            .skip_while(|line| line.trim() != REVISIONS_HEADING)
            .skip(1)
            .take_while(|line| !line.starts_with("## "))
            .filter_map(|line| line.trim().strip_prefix("- `"))
            .filter_map(|listed| listed.strip_suffix('`'))
            .map(str::to_string)
            .collect();
        listed.sort();

        assert!(
            !listed.is_empty(),
            "no revision is listed under `{REVISIONS_HEADING}` in the README, so this suite would \
             be checking the server against nothing"
        );
        listed
    }

    fn negotiated_version(response: &JsonRpcResponse) -> String {
        response
            .result
            .as_ref()
            .expect("Initialize should return a result")
            .get("protocolVersion")
            .expect("Result should have protocolVersion")
            .as_str()
            .expect("protocolVersion should be a string")
            .to_string()
    }

    /// What the server answers on and what the README says it answers on are one fact, and this is
    /// what keeps them one.
    ///
    /// Every revision rmcp knows is offered to a running server, and what comes back says whether
    /// the server took it: the requested revision means yes, the advertised one means it negotiated
    /// down. So the supported set is measured rather than restated, and holding it against the
    /// README catches the drift in both directions — a revision gained without a line in the file,
    /// and a line in the file for a revision the server has dropped.
    ///
    /// This covers the `initialize` path. The other entry point needs no test of ours: rmcp checks
    /// a request's inline `_meta` revision against the same list and answers `-32022 Unsupported
    /// protocol version` when it is absent from it.
    #[test]
    fn test_the_revisions_answered_are_the_revisions_documented() {
        let advertised = advertised_revision();
        let mut answered = Vec::new();
        for revision in ProtocolVersion::KNOWN_VERSIONS {
            let mut server = McpServerProcess::start_without_credentials();
            let negotiated = negotiated_version(&server.initialize_with_version(revision.as_str()));

            match negotiated == revision.as_str() {
                true => answered.push(negotiated),
                false => assert_eq!(
                    negotiated,
                    advertised,
                    "a revision this server does not implement ({}) should negotiate down to the \
                     newest one it documents, not to something else",
                    revision.as_str()
                ),
            }
        }
        answered.sort();

        assert_eq!(
            answered,
            documented_revisions(),
            "the revisions this server answers on are not the ones its README lists under \
             `{REVISIONS_HEADING}`, so whoever reads it would point a client at the wrong thing"
        );
    }

    /// `resultType` (SEP-2322) exists only from 2026-07-28. rmcp strips it for
    /// older peers, and every version this server supports is older, so it must
    /// never appear on the wire. Guards the `with_all_items` constructors, which
    /// set `result_type: Some(ResultType::COMPLETE)` before stripping.
    #[test]
    fn test_results_carry_no_result_type() {
        for version in documented_revisions() {
            let mut server = McpServerProcess::start_without_credentials();
            server.initialize_with_version(&version);

            let response = server.send_request("tools/list", json!({}));
            let result = response
                .result
                .expect("tools/list should return a result even with no tools");

            assert!(
                result.get("resultType").is_none(),
                "tools/list must not carry resultType at protocol version {}, got: {}",
                version,
                result
            );
        }
    }
}

// =============================================================================
// Server Tests
// =============================================================================

mod server {
    use super::*;

    #[test]
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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

        // Tools come from src/server/generated.rs, derived from the OpenAPI snapshot.
        // If no tools are available, that file is broken - fail fast!
        assert!(
            !tools.is_empty(),
            "No tools available! Regenerate them with \
             UPDATE_SDK=mcp cargo test -p hook0-sdkgen sdk_targets. \
             NEVER silently skip - fix the tool definitions."
        );

        // Should have at least the core tools (list_organizations, list_applications, etc.)
        assert!(
            tools.len() >= 5,
            "Should have at least 5 tools, got {}. \
             The OpenAPI snapshot may be incomplete or its `mcp` tag too sparse.",
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
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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
            // Validate contents exists and is an array (empty is valid if no applications exist)
            result
                .get("contents")
                .expect("Result should have contents")
                .as_array()
                .expect("contents should be an array");
        }
    }

    #[test]
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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

// =============================================================================
// Refusal Tests
// =============================================================================

/// What reaches an assistant when Hook0 refuses one of its tool calls.
///
/// The server is driven over its stdio transport, as everywhere else here, but the Hook0 it is
/// pointed at is a socket this test listens on and answers a real problem document from. No
/// credentials and no instance are needed, so unlike the tests above these are not `#[ignore]`d:
/// what they are about is how an answer is read, and the answer is right here.
mod refusals {
    use super::*;
    use std::io::Read;
    use std::net::TcpListener;
    use std::thread;

    /// The problem document Hook0 answers a duplicated ingestion with, as its API writes it.
    const ALREADY_INGESTED: &str = r#"{"id":"EventAlreadyIngested","title":"Event already Ingested","detail":"This event was previously ingested and recorded inside Hook0 service.","status":409}"#;

    /// Longest request this listener reads before it answers, in bytes.
    const MAX_REQUEST_BYTES: usize = 64 * 1024;

    /// Longest this listener waits on a socket that has stopped saying anything.
    const PATIENCE: Duration = Duration::from_secs(10);

    /// A Hook0 that refuses the first request it is sent, and answers where it is listening.
    fn refusing_hook0(status: &'static str, body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to listen");
        let at = format!(
            "http://{}",
            listener.local_addr().expect("Failed to read the port")
        );

        thread::spawn(move || {
            let Ok((mut connection, _)) = listener.accept() else {
                return;
            };
            let _ = connection.set_read_timeout(Some(PATIENCE));
            let _ = connection.set_write_timeout(Some(PATIENCE));

            // Read the head, so the request is off the socket before it is answered.
            let mut held = Vec::new();
            let mut byte = [0u8; 1];
            while held.len() < MAX_REQUEST_BYTES && !held.ends_with(b"\r\n\r\n") {
                match connection.read(&mut byte) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => held.push(byte[0]),
                }
            }

            // And then the body, which nothing here reads for what it says. It is taken off the
            // socket because leaving it there is what decides whether this answer arrives: a
            // connection dropped with bytes still unread is reset rather than closed, and the
            // client reads the reset in place of the refusal it was sent.
            let announced = String::from_utf8_lossy(&held)
                .lines()
                .filter_map(|line| line.split_once(':'))
                .find(|(name, _)| name.trim().eq_ignore_ascii_case("content-length"))
                .and_then(|(_, counted)| counted.trim().parse::<usize>().ok());
            if let Some(carried) = announced {
                let mut body = vec![0u8; carried.min(MAX_REQUEST_BYTES)];
                let _ = connection.read_exact(&mut body);
            }

            let _ = write!(
                connection,
                "HTTP/1.1 {status}\r\nContent-Type: application/problem+json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = connection.flush();
        });

        at
    }

    /// One ingestion, with every argument the generated tool declares it needs.
    fn an_ingestion() -> Value {
        json!({
            "name": "events.ingest",
            "arguments": {
                "application_id": "6f8a3e1c-9a2b-4d5e-8f70-1c2d3e4f5a6b",
                "event_id": "0195c0de-0000-7000-8000-000000000001",
                "event_type": "smoke.event.sent",
                "labels": { "language": "mcp" },
                "occurred_at": "2026-01-01T00:00:00Z",
                "payload": "{\"from\":\"a test\"}",
                "payload_content_type": "application/json",
            },
        })
    }

    #[test]
    fn a_refused_tool_call_names_the_problem_hook0_named() {
        let mut server = McpServerProcess::spawn(
            "placeholder-token-never-used",
            &refusing_hook0("409 Conflict", ALREADY_INGESTED),
        );
        server.initialize();

        let response = server.send_request("tools/call", an_ingestion());

        let error = response.error.expect("A refused call should return error");
        // Its stable name is the only part of a refusal an assistant can act on: it is what tells
        // an ingestion that already happened, and must not be tried again, from any other conflict.
        assert!(
            error.message.contains("EventAlreadyIngested"),
            "A refusal has to name the problem Hook0 named: {}",
            error.message
        );
    }

    #[test]
    fn a_refused_tool_call_says_what_the_problem_said() {
        let mut server = McpServerProcess::spawn(
            "placeholder-token-never-used",
            &refusing_hook0("409 Conflict", ALREADY_INGESTED),
        );
        server.initialize();

        let response = server.send_request("tools/call", an_ingestion());

        let error = response.error.expect("A refused call should return error");
        assert!(
            error
                .message
                .contains("This event was previously ingested and recorded inside Hook0 service."),
            "A refusal has to carry what Hook0 said about it: {}",
            error.message
        );
    }
}

// =============================================================================
// Query Parameter Tests
// =============================================================================

/// What an operation's query parameters do between the tool call and the request line.
///
/// Driven the same way as the refusals above — over stdio, against a socket this test listens on —
/// but what is read off that socket here is the request line rather than the answer. A tool
/// declares an argument required, an assistant supplies it, and the only place the value can
/// legitimately end up for a `GET` is the query string; these say so by looking at the bytes.
///
/// Not `#[ignore]`d, for the same reason the refusals are not: what they are about is what this
/// server composes, and nothing about that needs an instance to be reachable.
mod query_parameters {
    use super::*;
    use std::io::Read;
    use std::net::TcpListener;
    use std::sync::mpsc::{self, Receiver};
    use std::thread;

    /// Longest request this listener reads before it answers, in bytes.
    const MAX_REQUEST_BYTES: usize = 64 * 1024;

    /// Longest this listener waits on a socket that has stopped saying anything.
    const PATIENCE: Duration = Duration::from_secs(10);

    /// Longest a case waits for the request line to come back off the listener.
    const DELIVERY: Duration = Duration::from_secs(20);

    /// A Hook0 that answers an empty list, and hands back the request line it was asked with.
    fn recording_hook0() -> (String, Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to listen");
        let at = format!(
            "http://{}",
            listener.local_addr().expect("Failed to read the port")
        );
        let (asked, heard) = mpsc::channel();

        thread::spawn(move || {
            let Ok((mut connection, _)) = listener.accept() else {
                return;
            };
            let _ = connection.set_read_timeout(Some(PATIENCE));
            let _ = connection.set_write_timeout(Some(PATIENCE));

            let mut held = Vec::new();
            let mut byte = [0u8; 1];
            while held.len() < MAX_REQUEST_BYTES && !held.ends_with(b"\r\n\r\n") {
                match connection.read(&mut byte) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => held.push(byte[0]),
                }
            }

            let head = String::from_utf8_lossy(&held).into_owned();
            let _ = asked.send(head.lines().next().unwrap_or_default().to_owned());

            let body = "[]";
            let _ = write!(
                connection,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = connection.flush();
        });

        (at, heard)
    }

    /// The request line one tool call put on the wire.
    fn asked_with(arguments: Value) -> String {
        let (at, heard) = recording_hook0();
        let mut server = McpServerProcess::spawn("placeholder-token-never-used", &at);
        server.initialize();
        server.send_request("tools/call", arguments);

        heard
            .recv_timeout(DELIVERY)
            .expect("the server never reached the Hook0 it was pointed at")
    }

    /// An argument the tool declares required reaches the API, rather than being asked for and
    /// dropped. `applications.list` takes one, and it is the whole of what the operation selects
    /// on, so a request without it is not a narrower question — it is a different one.
    #[test]
    fn applications_list_carries_the_organization_it_was_given() {
        let organization = "6f8a3e1c-9a2b-4d5e-8f70-1c2d3e4f5a6b";

        let asked = asked_with(json!({
            "name": "applications.list",
            "arguments": { "organization_id": organization },
        }));

        assert!(
            asked.contains(&format!("organization_id={organization}")),
            "the organization the tool was given reached nothing: {asked}"
        );
    }

    /// A value travels percent-encoded, and a space travels as `%20` rather than as `+`.
    ///
    /// Pinned rather than left to the encoder, because it is a decision: `+` is a literal plus
    /// under RFC 3986 and reads as a space only to something decoding the query as a form. A
    /// cursor is opaque text the API hands out, so it is the argument most likely to carry one.
    #[test]
    fn a_query_value_travels_percent_encoded() {
        let asked = asked_with(json!({
            "name": "requestAttempts.list",
            "arguments": {
                "application_id": "6f8a3e1c-9a2b-4d5e-8f70-1c2d3e4f5a6b",
                "pagination_cursor": "one two",
            },
        }));

        assert!(
            asked.contains("pagination_cursor=one%20two"),
            "a space has to travel as `%20`: {asked}"
        );
    }

    /// Every argument reaches the API, not merely the first one. `requestAttempts.list` declares
    /// seven, where every other tool declares one, so it is the case that tells composing a query
    /// string apart from filling in a single value and calling it done.
    #[test]
    fn request_attempts_list_carries_every_filter_it_was_given() {
        let filters = [
            ("application_id", "6f8a3e1c-9a2b-4d5e-8f70-1c2d3e4f5a6b"),
            ("event.event_type_names", "smoke.event.sent"),
            ("event_id", "0195c0de-0000-7000-8000-000000000001"),
            ("max_created_at", "2026-01-02T00:00:00Z"),
            ("min_created_at", "2026-01-01T00:00:00Z"),
            ("pagination_cursor", "cursor-1"),
            ("subscription_id", "0195c0de-0000-7000-8000-000000000002"),
        ];

        let mut arguments = serde_json::Map::new();
        for (name, value) in filters {
            arguments.insert(name.to_owned(), json!(value));
        }

        let asked = asked_with(json!({
            "name": "requestAttempts.list",
            "arguments": Value::Object(arguments),
        }));

        for (name, _) in filters {
            assert!(
                asked.contains(&format!("{name}=")),
                "`{name}` was asked for and reached nothing: {asked}"
            );
        }
    }
}

// =============================================================================
// Documented Tools
// =============================================================================

/// The tool table the README publishes, held against the tools a running server offers.
///
/// This needs no credentials for the reason `protocol` does not: `tools/list` is answered from the
/// generated table and never reaches the Hook0 API.
mod documented_tools {
    use super::*;

    /// Where the README lists the tools this server exposes.
    const TOOLS_HEADING: &str = "## Available Tools";

    /// The tools the README lists under [`TOOLS_HEADING`], sorted.
    ///
    /// Read out of the file rather than restated here, for the reason the protocol revisions are:
    /// a copy of the list beside the one readers see is a copy that goes stale in silence. This one
    /// did, and for a whole release the README sent readers to fifteen tools under names that had
    /// been renamed out from under it.
    ///
    /// Only whole rows of the shape ``| `<tool>` | … |`` count, so the prose around the tables can
    /// name a tool without being read as listing it.
    fn documented_tools() -> Vec<String> {
        let readme = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md"),
        )
        .expect("the README this crate publishes");

        let mut listed: Vec<String> = readme
            .lines()
            .skip_while(|line| line.trim() != TOOLS_HEADING)
            .skip(1)
            .take_while(|line| !line.starts_with("## "))
            .filter_map(|line| line.trim().strip_prefix("| `"))
            .filter_map(|row| row.split_once("` |"))
            .map(|(named, _)| named.to_owned())
            .collect();
        listed.sort();

        assert!(
            !listed.is_empty(),
            "no tool is listed under `{TOOLS_HEADING}` in the README, so this suite would be \
             checking the server against nothing"
        );
        listed
    }

    /// The tools a running server offers, sorted.
    fn offered_tools() -> Vec<String> {
        let mut server = McpServerProcess::start_without_credentials();
        server.initialize();

        let result = server
            .send_request("tools/list", json!({}))
            .result
            .expect("tools/list returns a result");

        let mut offered: Vec<String> = result
            .get("tools")
            .and_then(Value::as_array)
            .expect("tools/list returns a list of tools")
            .iter()
            .map(|tool| {
                tool.get("name")
                    .and_then(Value::as_str)
                    .expect("every tool carries a name")
                    .to_owned()
            })
            .collect();
        offered.sort();

        assert!(!offered.is_empty(), "the server offered no tool at all");
        offered
    }

    /// What the server offers and what its README says it offers are one fact.
    ///
    /// Measured rather than restated, so the drift is caught in both directions: a tool gained
    /// without a row in the file, and a row in the file for a tool the server no longer has. The
    /// second is the one that shipped, and it is invisible from the inside. A reader follows the
    /// README, calls a name that is not there, and the only thing that says so is the answer.
    #[test]
    fn test_the_tools_offered_are_the_tools_documented() {
        assert_eq!(
            offered_tools(),
            documented_tools(),
            "the tools this server offers are not the ones its README lists under \
             `{TOOLS_HEADING}`, so whoever reads it would call a name that is not there"
        );
    }
}
