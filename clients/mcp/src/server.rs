//! MCP Server implementation for Hook0
//!
//! This server dynamically generates tools from the OpenAPI specification.
//! Tool definitions, dispatch logic, and read/write classification are all
//! auto-generated at build time from the OpenAPI spec.

use crate::client::Hook0Client;
use crate::error::{McpError, McpErrorExt};
use crate::prompts;
use rmcp::model::{
    Annotated, CallToolRequestParams, CallToolResult, Content, GetPromptRequestParams,
    GetPromptResult, Implementation, ListPromptsResult, ListResourcesResult, ListToolsResult,
    PaginatedRequestParams, ProtocolVersion, RawResource, ReadResourceRequestParams,
    ReadResourceResult, ResourceContents, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler};
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

// Include auto-generated code from build.rs
mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

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

    Tool {
        name: info.name.to_string().into(),
        title: None,
        description: Some(info.description.to_string().into()),
        input_schema: schema.as_object().cloned().unwrap_or_default().into(),
        output_schema: None,
        annotations: None,
        icons: None,
        meta: None,
        execution: None,
    }
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
        Ok(CallToolResult::success(vec![Content::text(content)]))
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
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
            server_info: Implementation {
                name: "hook0-mcp".into(),
                title: Some("Hook0 MCP Server".into()),
                description: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: Some("https://www.hook0.com/".into()),
            },
            instructions: Some(
                "Hook0 MCP Server - Manage webhooks, subscriptions, and events. \
                 Use tools to create applications, register event types, \
                 configure subscriptions, and debug delivery issues."
                    .into(),
            ),
        }
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

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
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
        self.dispatch_tool(&request.name, &args).await
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        info!("Listing resources");

        let mut org_resource = RawResource::new("hook0://organizations", "Organizations");
        org_resource.description = Some("List all accessible organizations".into());
        org_resource.mime_type = Some("application/json".into());

        let mut app_resource = RawResource::new("hook0://applications", "Applications");
        app_resource.description = Some("List all applications".into());
        app_resource.mime_type = Some("application/json".into());

        let resources = vec![
            Annotated::new(org_resource, None),
            Annotated::new(app_resource, None),
        ];

        Ok(ListResourcesResult {
            resources,
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        info!("Reading resource: {}", request.uri);

        let content = match request.uri.as_str() {
            "hook0://organizations" => self.client.get("/organizations/").await,
            "hook0://applications" => self.client.get("/applications/").await,
            uri if uri.starts_with("hook0://applications/") => {
                let rest = uri.strip_prefix("hook0://applications/").unwrap();
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
            }
            uri if uri.starts_with("hook0://events/") => {
                let rest = uri.strip_prefix("hook0://events/").unwrap();
                if let Some(event_id) = rest.strip_suffix("/attempts") {
                    self.client
                        .get(&format!("/events/{}/request_attempts/", event_id))
                        .await
                } else {
                    self.client.get(&format!("/events/{}/", rest)).await
                }
            }
            _ => {
                return Err(McpError::resource_not_found(
                    format!("Resource not found: {}", request.uri),
                    None,
                ));
            }
        };

        let content = content.map_err(|e| -> McpError { e.into() })?;
        let text = serde_json::to_string_pretty(&content).unwrap_or_else(|_| content.to_string());

        Ok(ReadResourceResult {
            contents: vec![ResourceContents::text(text, request.uri)],
        })
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        info!("Listing prompts");
        Ok(ListPromptsResult {
            meta: None,
            next_cursor: None,
            prompts: prompts::list_prompts(),
        })
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        info!("Getting prompt: {}", request.name);
        // Convert JsonObject to HashMap<String, String>
        let args = request.arguments.map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        });
        prompts::get_prompt(&request.name, args)
    }
}
