//! MCP Server implementation for Hook0
//!
//! Tool definitions, dispatch logic, and read/write classification all come from
//! the OpenAPI snapshot the API crate commits. They live in `generated.rs`, a
//! committed source file the `hook0-sdkgen` crate writes and guards, so this crate
//! builds from its own contents alone and never reads the snapshot.

use crate::client::Hook0Client;
use crate::error::{McpError, McpErrorExt};
use crate::prompts;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, GetPromptRequestParams,
    GetPromptResponse, Implementation, ListPromptsResult, ListResourcesResult, ListToolsResult,
    PaginatedRequestParams, ProtocolVersion, ReadResourceRequestParams, ReadResourceResponse,
    ReadResourceResult, Resource, ResourceContents, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler};
use serde_json::{Map, Value, json};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// The version `get_info` advertises, and the one a client asking for something unimplemented is
/// answered with.
///
/// Named here so that it can be an element of the list below rather than a second statement beside
/// it. Written out twice, the two could disagree — and the way they disagree is a server advertising
/// a version it then refuses, which leaves a client negotiating towards something it cannot have.
const ADVERTISED_PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::V_2025_11_25;

/// Protocol versions this handler actually implements.
///
/// Deliberately excludes `2026-07-28`: it requires the stateless lifecycle,
/// `subscriptions/listen`, and MRTR input-required handling that this server
/// does not provide. rmcp enforces this list on both entry points: it bounds what
/// `initialize` may negotiate down to, and it rejects any request whose inline
/// `_meta` names an unlisted version with `-32022 Unsupported protocol version`.
///
/// `README.md` states this list for whoever points a client at this server, and the handshake suite
/// holds the two together, so a version added or dropped here without the file changing fails.
const SUPPORTED_PROTOCOL_VERSIONS: &[ProtocolVersion] = &[
    ProtocolVersion::V_2024_11_05,
    ProtocolVersion::V_2025_03_26,
    ProtocolVersion::V_2025_06_18,
    ADVERTISED_PROTOCOL_VERSION,
];

mod generated;

pub use generated::{
    GENERATED_TOOLS, GeneratedToolInfo, get_tool_info, interpolate_path, is_write_tool,
};

/// Hook0 MCP Server
#[derive(Clone)]
pub struct Hook0McpServer {
    client: Arc<Hook0Client>,
    /// Read-only mode (only expose GET endpoints)
    read_only: bool,
}

/// Helper to create a Tool from generated info
fn make_tool_from_generated(info: &GeneratedToolInfo) -> Tool {
    let schema: Value = serde_json::from_str(info.input_schema)
        .unwrap_or_else(|_| json!({"type": "object", "properties": {}, "required": []}));

    Tool::new(
        info.name.to_string(),
        info.description.to_string(),
        schema.as_object().cloned().unwrap_or_default(),
    )
}

impl Hook0McpServer {
    /// Create a new Hook0 MCP server
    ///
    /// # Arguments
    /// * `client` - The HTTP client for Hook0 API
    /// * `read_only` - If true, only read operations (GET) are exposed
    pub fn new(client: Hook0Client, read_only: bool) -> Self {
        Self {
            client: Arc::new(client),
            read_only,
        }
    }

    /// Generic tool dispatcher using generated tool info
    async fn dispatch_tool(
        &self,
        name: &str,
        args: &Map<String, Value>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Dispatching tool: {} with args: {:?}", name, args);

        let tool_info = get_tool_info(name).ok_or_else(|| McpError::tool_not_found(name))?;

        // Build path parameters map for interpolation
        // Handle string, numeric, and boolean values for path parameters
        let mut path_params: HashMap<String, String> = HashMap::new();
        for (key, value) in args {
            let string_value = match value {
                Value::String(s) => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                Value::Bool(b) => Some(b.to_string()),
                _ => None,
            };
            if let Some(s) = string_value {
                path_params.insert(key.clone(), s);
            }
        }

        // Interpolate path template with parameters
        let path = interpolate_path(tool_info.path_template, &path_params);

        // Execute the appropriate HTTP method
        let result = match tool_info.method {
            "GET" => self.client.get(&path).await,
            "POST" => {
                // Build request body from non-path args
                let body = self.build_request_body(args, tool_info.path_template);
                self.client.post(&path, body).await
            }
            "PUT" => {
                let body = self.build_request_body(args, tool_info.path_template);
                self.client.put(&path, body).await
            }
            "PATCH" => {
                let body = self.build_request_body(args, tool_info.path_template);
                self.client.patch(&path, body).await
            }
            "DELETE" => self.client.delete(&path).await,
            _ => {
                return Err(McpError::internal_error(
                    format!("Unknown HTTP method: {}", tool_info.method),
                    None,
                ));
            }
        };

        let result = result.map_err(|e| -> McpError { e.into() })?;

        // Convert result to CallToolResult
        let content = serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string());
        Ok(CallToolResult::success(vec![ContentBlock::text(content)]))
    }

    /// Build request body from arguments, excluding path parameters
    fn build_request_body(&self, args: &Map<String, Value>, path_template: &str) -> Option<Value> {
        // Extract path parameter names from template (e.g., {application_id})
        let path_param_names: Vec<&str> = path_template
            .split('/')
            .filter(|s| s.starts_with('{') && s.ends_with('}'))
            .map(|s| &s[1..s.len() - 1])
            .collect();

        // Filter out path parameters from args to build body
        let body_args: Map<String, Value> = args
            .iter()
            .filter(|(key, _)| !path_param_names.contains(&key.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        if body_args.is_empty() {
            None
        } else {
            Some(Value::Object(body_args))
        }
    }
}

impl ServerHandler for Hook0McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
        )
        .with_server_info(
            Implementation::new("hook0-mcp", env!("CARGO_PKG_VERSION"))
                .with_title("Hook0 MCP Server")
                .with_website_url("https://www.hook0.com/"),
        )
        .with_instructions(
            "Hook0 MCP Server - Manage webhooks, subscriptions, and events. \
             Use tools to create applications, register event types, \
             configure subscriptions, and debug delivery issues.",
        )
        .with_protocol_version(ADVERTISED_PROTOCOL_VERSION)
    }

    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(SUPPORTED_PROTOCOL_VERSIONS)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        info!("Listing tools (read_only: {})", self.read_only);

        // Generate tools from the auto-generated definitions
        let tools: Vec<Tool> = GENERATED_TOOLS
            .iter()
            .filter(|t| !self.read_only || !t.is_write_operation())
            .map(make_tool_from_generated)
            .collect();

        info!("Returning {} tools", tools.len());

        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        info!("Calling tool: {}", request.name);

        // Reject write tools in read-only mode
        if self.read_only && is_write_tool(&request.name) {
            return Err(McpError::invalid_params(
                format!(
                    "Tool '{}' is not available in read-only mode. \
                     Set HOOK0_READ_ONLY=false to enable write operations.",
                    request.name
                ),
                None,
            ));
        }

        let args = request.arguments.unwrap_or_default();
        self.dispatch_tool(&request.name, &args)
            .await
            .map(Into::into)
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        info!("Listing resources");

        let resources = vec![
            Resource::new("hook0://organizations", "Organizations")
                .with_description("List all accessible organizations")
                .with_mime_type("application/json"),
            Resource::new("hook0://applications", "Applications")
                .with_description("List all applications")
                .with_mime_type("application/json"),
        ];

        Ok(ListResourcesResult::with_all_items(resources))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        info!("Reading resource: {}", request.uri);

        let content = match request.uri.as_str() {
            "hook0://organizations" => self.client.get("/organizations/").await,
            "hook0://applications" => self.client.get("/applications/").await,
            uri => {
                // The prefix is stripped where it is tested, rather than tested in a match guard
                // and stripped again in the arm: the second reading is what has to be kept in step
                // with the first, and nothing in the language keeps it there.
                if let Some(rest) = uri.strip_prefix("hook0://applications/") {
                    if let Some(app_id) = rest.strip_suffix("/events") {
                        self.client
                            .get(&format!("/applications/{}/events/", app_id))
                            .await
                    } else if let Some(app_id) = rest.strip_suffix("/subscriptions") {
                        self.client
                            .get(&format!("/applications/{}/subscriptions/", app_id))
                            .await
                    } else if let Some(app_id) = rest.strip_suffix("/event_types") {
                        self.client
                            .get(&format!("/applications/{}/event_types/", app_id))
                            .await
                    } else {
                        self.client.get(&format!("/applications/{}/", rest)).await
                    }
                } else if let Some(rest) = uri.strip_prefix("hook0://events/") {
                    if let Some(event_id) = rest.strip_suffix("/attempts") {
                        self.client
                            .get(&format!("/events/{}/request_attempts/", event_id))
                            .await
                    } else {
                        self.client.get(&format!("/events/{}/", rest)).await
                    }
                } else {
                    return Err(McpError::resource_not_found(
                        format!("Resource not found: {}", request.uri),
                        None,
                    ));
                }
            }
        };

        let content = content.map_err(|e| -> McpError { e.into() })?;
        let text = serde_json::to_string_pretty(&content).unwrap_or_else(|_| content.to_string());

        Ok(ReadResourceResult::new(vec![ResourceContents::text(text, request.uri)]).into())
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        info!("Listing prompts");
        Ok(ListPromptsResult::with_all_items(prompts::list_prompts()))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResponse, McpError> {
        info!("Getting prompt: {}", request.name);
        // Convert JsonObject to HashMap<String, String>
        let args = request.arguments.map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        });
        prompts::get_prompt(&request.name, args).map(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::GENERATED_TOOLS;
    use serde_json::Value;
    use std::collections::BTreeSet;

    /// The document the tools are generated from, read here on its own terms: the committed tool
    /// table is compared against what the snapshot says, not against another reading by the code
    /// that wrote it.
    const SNAPSHOT_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../api/openapi.snapshot.json"
    );

    const MCP_TAG: &str = "mcp";

    const HTTP_METHODS: [&str; 5] = ["get", "post", "put", "patch", "delete"];

    fn tagged_operation_ids() -> BTreeSet<String> {
        let bytes = std::fs::read(SNAPSHOT_PATH).expect("the committed snapshot is readable");
        let snapshot: Value = serde_json::from_slice(&bytes).expect("the snapshot is JSON");

        let mut tagged = BTreeSet::new();
        let paths = snapshot["paths"]
            .as_object()
            .expect("the snapshot carries paths");

        for item in paths.values() {
            for method in HTTP_METHODS {
                let Some(operation) = item.get(method) else {
                    continue;
                };
                let carries_tag = operation["tags"]
                    .as_array()
                    .is_some_and(|tags| tags.iter().any(|tag| tag == MCP_TAG));
                if !carries_tag {
                    continue;
                }
                let name = operation["operationId"]
                    .as_str()
                    .expect("a tagged operation carries an operation id");
                tagged.insert(name.to_owned());
            }
        }

        tagged
    }

    #[test]
    fn a_tool_is_exposed_for_every_tagged_operation_and_for_nothing_else() {
        let tagged = tagged_operation_ids();
        assert!(
            !tagged.is_empty(),
            "the snapshot marks no operation for this server"
        );
        assert!(
            !GENERATED_TOOLS.is_empty(),
            "this server exposes no tool at all"
        );

        let exposed: BTreeSet<String> = GENERATED_TOOLS
            .iter()
            .map(|tool| tool.name.to_owned())
            .collect();

        assert_eq!(
            exposed, tagged,
            "the tools this server exposes are not the operations the snapshot marks for it"
        );
    }

    #[test]
    fn every_tool_carries_a_schema_a_caller_can_fill_in() {
        for tool in GENERATED_TOOLS {
            let schema: Value = serde_json::from_str(tool.input_schema)
                .unwrap_or_else(|err| panic!("the schema of `{}` is not JSON: {err}", tool.name));

            assert_eq!(
                schema["type"], "object",
                "the schema of `{}` is not an object",
                tool.name
            );
            assert!(
                schema["properties"].is_object(),
                "the schema of `{}` carries no properties object",
                tool.name
            );
            assert!(
                !tool.input_schema.contains("$ref"),
                "the schema of `{}` still points at a reference no caller can follow",
                tool.name
            );
            assert!(
                !tool.path_template.is_empty(),
                "the tool `{}` answers on no path",
                tool.name
            );
        }
    }
}
