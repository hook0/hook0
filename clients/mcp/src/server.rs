//! MCP Server implementation for Hook0

use crate::client::Hook0Client;
use crate::error::{McpError, McpErrorExt};
use crate::prompts;
use rmcp::model::*;
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler};
use serde_json::{Map, Value, json};
use std::sync::Arc;
use tracing::{debug, info};

/// Tools that are write operations (not available in read-only mode)
const WRITE_TOOLS: &[&str] = &[
    "create_application",
    "delete_application",
    "create_event_type",
    "create_subscription",
    "delete_subscription",
    "ingest_event",
    "retry_delivery",
];

/// Hook0 MCP Server
#[derive(Clone)]
pub struct Hook0McpServer {
    client: Arc<Hook0Client>,
    /// Read-only mode (only expose GET endpoints)
    read_only: bool,
}

/// Helper to create a Tool with all required fields
fn make_tool(name: &str, description: &str, schema: Value) -> Tool {
    Tool {
        name: name.to_string().into(),
        title: None,
        description: Some(description.to_string().into()),
        input_schema: schema.as_object().cloned().unwrap_or_default().into(),
        output_schema: None,
        annotations: None,
        icons: None,
        meta: None,
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

    /// Check if a tool is a write operation
    fn is_write_tool(name: &str) -> bool {
        WRITE_TOOLS.contains(&name)
    }

    // ==================== Tool Implementations ====================

    /// List all organizations
    async fn list_organizations(&self) -> Result<Value, McpError> {
        self.client
            .get("/organizations/")
            .await
            .map_err(|e| e.into())
    }

    /// List all applications
    async fn list_applications(&self, organization_id: Option<&str>) -> Result<Value, McpError> {
        let path = match organization_id {
            Some(org_id) => format!("/organizations/{}/applications/", org_id),
            None => "/applications/".to_string(),
        };
        self.client.get(&path).await.map_err(|e| e.into())
    }

    /// Get application details
    async fn get_application(&self, application_id: &str) -> Result<Value, McpError> {
        self.client
            .get(&format!("/applications/{}/", application_id))
            .await
            .map_err(|e| e.into())
    }

    /// Create an application
    async fn create_application(
        &self,
        organization_id: &str,
        name: &str,
    ) -> Result<Value, McpError> {
        let body = json!({
            "organization_id": organization_id,
            "name": name
        });
        self.client
            .post("/applications/", Some(body))
            .await
            .map_err(|e| e.into())
    }

    /// Delete an application
    async fn delete_application(&self, application_id: &str) -> Result<Value, McpError> {
        self.client
            .delete(&format!("/applications/{}/", application_id))
            .await
            .map_err(|e| e.into())
    }

    /// List event types for an application
    async fn list_event_types(&self, application_id: &str) -> Result<Value, McpError> {
        self.client
            .get(&format!("/applications/{}/event_types/", application_id))
            .await
            .map_err(|e| e.into())
    }

    /// Create an event type
    async fn create_event_type(&self, application_id: &str, name: &str) -> Result<Value, McpError> {
        let body = json!({
            "application_id": application_id,
            "name": name
        });
        self.client
            .post("/event_types/", Some(body))
            .await
            .map_err(|e| e.into())
    }

    /// List subscriptions for an application
    async fn list_subscriptions(&self, application_id: &str) -> Result<Value, McpError> {
        self.client
            .get(&format!("/applications/{}/subscriptions/", application_id))
            .await
            .map_err(|e| e.into())
    }

    /// Create a subscription
    async fn create_subscription(
        &self,
        application_id: &str,
        name: &str,
        target_url: &str,
        event_types: Vec<String>,
    ) -> Result<Value, McpError> {
        let body = json!({
            "application_id": application_id,
            "name": name,
            "target": target_url,
            "event_types": event_types
        });
        self.client
            .post("/subscriptions/", Some(body))
            .await
            .map_err(|e| e.into())
    }

    /// Delete a subscription
    async fn delete_subscription(&self, subscription_id: &str) -> Result<Value, McpError> {
        self.client
            .delete(&format!("/subscriptions/{}/", subscription_id))
            .await
            .map_err(|e| e.into())
    }

    /// List events for an application
    async fn list_events(&self, application_id: &str) -> Result<Value, McpError> {
        self.client
            .get(&format!("/applications/{}/events/", application_id))
            .await
            .map_err(|e| e.into())
    }

    /// Get event details
    async fn get_event(&self, event_id: &str) -> Result<Value, McpError> {
        self.client
            .get(&format!("/events/{}/", event_id))
            .await
            .map_err(|e| e.into())
    }

    /// Ingest an event
    async fn ingest_event(
        &self,
        application_id: &str,
        event_type: &str,
        payload: Value,
    ) -> Result<Value, McpError> {
        let body = json!({
            "application_id": application_id,
            "event_type": event_type,
            "payload": payload,
            "payload_content_type": "application/json"
        });
        self.client
            .post("/events/", Some(body))
            .await
            .map_err(|e| e.into())
    }

    /// List request attempts for an event
    async fn list_request_attempts(&self, event_id: &str) -> Result<Value, McpError> {
        self.client
            .get(&format!("/events/{}/request_attempts/", event_id))
            .await
            .map_err(|e| e.into())
    }

    /// Retry a request attempt
    async fn retry_request_attempt(&self, request_attempt_id: &str) -> Result<Value, McpError> {
        self.client
            .post(
                &format!("/request_attempts/{}/retry/", request_attempt_id),
                None,
            )
            .await
            .map_err(|e| e.into())
    }

    // ==================== Tool Dispatch ====================

    /// Dispatch a tool call
    async fn dispatch_tool(
        &self,
        name: &str,
        args: &Map<String, Value>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Dispatching tool: {} with args: {:?}", name, args);

        let result = match name {
            // Organization operations
            "list_organizations" => self.list_organizations().await?,

            // Application operations
            "list_applications" => {
                let org_id = args.get("organization_id").and_then(|v| v.as_str());
                self.list_applications(org_id).await?
            }
            "get_application" => {
                let app_id = args
                    .get("application_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing application_id", None))?;
                self.get_application(app_id).await?
            }
            "create_application" => {
                let org_id = args
                    .get("organization_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing organization_id", None))?;
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing name", None))?;
                self.create_application(org_id, name).await?
            }
            "delete_application" => {
                let app_id = args
                    .get("application_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing application_id", None))?;
                self.delete_application(app_id).await?
            }

            // Event type operations
            "list_event_types" => {
                let app_id = args
                    .get("application_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing application_id", None))?;
                self.list_event_types(app_id).await?
            }
            "create_event_type" => {
                let app_id = args
                    .get("application_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing application_id", None))?;
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing name", None))?;
                self.create_event_type(app_id, name).await?
            }

            // Subscription operations
            "list_subscriptions" => {
                let app_id = args
                    .get("application_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing application_id", None))?;
                self.list_subscriptions(app_id).await?
            }
            "create_subscription" => {
                let app_id = args
                    .get("application_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing application_id", None))?;
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing name", None))?;
                let target_url = args
                    .get("target_url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing target_url", None))?;
                let event_types = args
                    .get("event_types")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                self.create_subscription(app_id, name, target_url, event_types)
                    .await?
            }
            "delete_subscription" => {
                let sub_id = args
                    .get("subscription_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing subscription_id", None))?;
                self.delete_subscription(sub_id).await?
            }

            // Event operations
            "list_events" => {
                let app_id = args
                    .get("application_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing application_id", None))?;
                self.list_events(app_id).await?
            }
            "get_event" => {
                let event_id = args
                    .get("event_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing event_id", None))?;
                self.get_event(event_id).await?
            }
            "ingest_event" => {
                let app_id = args
                    .get("application_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing application_id", None))?;
                let event_type = args
                    .get("event_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing event_type", None))?;
                let payload = args
                    .get("payload")
                    .cloned()
                    .unwrap_or(Value::Object(Map::new()));
                self.ingest_event(app_id, event_type, payload).await?
            }

            // Request attempt operations
            "list_request_attempts" => {
                let event_id = args
                    .get("event_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing event_id", None))?;
                self.list_request_attempts(event_id).await?
            }
            "retry_delivery" => {
                let attempt_id = args
                    .get("request_attempt_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing request_attempt_id", None))?;
                self.retry_request_attempt(attempt_id).await?
            }

            _ => return Err(McpError::tool_not_found(name)),
        };

        // Convert result to CallToolResult
        let content = serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string());
        Ok(CallToolResult::success(vec![Content::text(content)]))
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
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        info!("Listing tools");

        let tools = vec![
            // Organization tools
            make_tool(
                "list_organizations",
                "List all organizations accessible to the authenticated user",
                json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            ),
            // Application tools
            make_tool(
                "list_applications",
                "List all applications. Optionally filter by organization.",
                json!({
                    "type": "object",
                    "properties": {
                        "organization_id": {
                            "type": "string",
                            "description": "Optional organization ID to filter by"
                        }
                    },
                    "required": []
                }),
            ),
            make_tool(
                "get_application",
                "Get details of a specific application",
                json!({
                    "type": "object",
                    "properties": {
                        "application_id": {
                            "type": "string",
                            "description": "The application ID"
                        }
                    },
                    "required": ["application_id"]
                }),
            ),
            make_tool(
                "create_application",
                "Create a new Hook0 application within an organization",
                json!({
                    "type": "object",
                    "properties": {
                        "organization_id": {
                            "type": "string",
                            "description": "The organization ID"
                        },
                        "name": {
                            "type": "string",
                            "description": "Name for the new application"
                        }
                    },
                    "required": ["organization_id", "name"]
                }),
            ),
            make_tool(
                "delete_application",
                "Delete an application. WARNING: This is destructive and cannot be undone.",
                json!({
                    "type": "object",
                    "properties": {
                        "application_id": {
                            "type": "string",
                            "description": "The application ID to delete"
                        }
                    },
                    "required": ["application_id"]
                }),
            ),
            // Event type tools
            make_tool(
                "list_event_types",
                "List all event types for an application",
                json!({
                    "type": "object",
                    "properties": {
                        "application_id": {
                            "type": "string",
                            "description": "The application ID"
                        }
                    },
                    "required": ["application_id"]
                }),
            ),
            make_tool(
                "create_event_type",
                "Register a new event type. Name should follow: service.resource.action",
                json!({
                    "type": "object",
                    "properties": {
                        "application_id": {
                            "type": "string",
                            "description": "The application ID"
                        },
                        "name": {
                            "type": "string",
                            "description": "Event type name (e.g., 'order.payment.completed')"
                        }
                    },
                    "required": ["application_id", "name"]
                }),
            ),
            // Subscription tools
            make_tool(
                "list_subscriptions",
                "List all webhook subscriptions for an application",
                json!({
                    "type": "object",
                    "properties": {
                        "application_id": {
                            "type": "string",
                            "description": "The application ID"
                        }
                    },
                    "required": ["application_id"]
                }),
            ),
            make_tool(
                "create_subscription",
                "Create a webhook subscription to receive events at a target URL",
                json!({
                    "type": "object",
                    "properties": {
                        "application_id": {
                            "type": "string",
                            "description": "The application ID"
                        },
                        "name": {
                            "type": "string",
                            "description": "Name for the subscription"
                        },
                        "target_url": {
                            "type": "string",
                            "description": "URL where webhooks will be delivered"
                        },
                        "event_types": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Event types to subscribe to"
                        }
                    },
                    "required": ["application_id", "name", "target_url"]
                }),
            ),
            make_tool(
                "delete_subscription",
                "Delete a webhook subscription",
                json!({
                    "type": "object",
                    "properties": {
                        "subscription_id": {
                            "type": "string",
                            "description": "The subscription ID to delete"
                        }
                    },
                    "required": ["subscription_id"]
                }),
            ),
            // Event tools
            make_tool(
                "list_events",
                "List events for an application",
                json!({
                    "type": "object",
                    "properties": {
                        "application_id": {
                            "type": "string",
                            "description": "The application ID"
                        }
                    },
                    "required": ["application_id"]
                }),
            ),
            make_tool(
                "get_event",
                "Get details of a specific event",
                json!({
                    "type": "object",
                    "properties": {
                        "event_id": {
                            "type": "string",
                            "description": "The event ID"
                        }
                    },
                    "required": ["event_id"]
                }),
            ),
            make_tool(
                "ingest_event",
                "Ingest a new event, triggering webhook delivery to matching subscriptions",
                json!({
                    "type": "object",
                    "properties": {
                        "application_id": {
                            "type": "string",
                            "description": "The application ID"
                        },
                        "event_type": {
                            "type": "string",
                            "description": "The event type (must be registered)"
                        },
                        "payload": {
                            "type": "object",
                            "description": "The event payload as JSON"
                        }
                    },
                    "required": ["application_id", "event_type", "payload"]
                }),
            ),
            // Request attempt tools
            make_tool(
                "list_request_attempts",
                "List delivery attempts for an event (for debugging)",
                json!({
                    "type": "object",
                    "properties": {
                        "event_id": {
                            "type": "string",
                            "description": "The event ID"
                        }
                    },
                    "required": ["event_id"]
                }),
            ),
            make_tool(
                "retry_delivery",
                "Retry a failed webhook delivery attempt",
                json!({
                    "type": "object",
                    "properties": {
                        "request_attempt_id": {
                            "type": "string",
                            "description": "The request attempt ID to retry"
                        }
                    },
                    "required": ["request_attempt_id"]
                }),
            ),
        ];

        // Filter out write tools if in read-only mode
        let tools = if self.read_only {
            tools
                .into_iter()
                .filter(|t| !Self::is_write_tool(&t.name))
                .collect()
        } else {
            tools
        };

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        info!("Calling tool: {}", request.name);

        // Reject write tools in read-only mode
        if self.read_only && Self::is_write_tool(&request.name) {
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
        _request: Option<PaginatedRequestParam>,
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
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        info!("Reading resource: {}", request.uri);

        let content = match request.uri.as_str() {
            "hook0://organizations" => self.list_organizations().await?,
            "hook0://applications" => self.list_applications(None).await?,
            uri if uri.starts_with("hook0://applications/") => {
                let app_id = uri.strip_prefix("hook0://applications/").unwrap();
                if app_id.contains("/events") {
                    let app_id = app_id.strip_suffix("/events").unwrap();
                    self.list_events(app_id).await?
                } else if app_id.contains("/subscriptions") {
                    let app_id = app_id.strip_suffix("/subscriptions").unwrap();
                    self.list_subscriptions(app_id).await?
                } else if app_id.contains("/event_types") {
                    let app_id = app_id.strip_suffix("/event_types").unwrap();
                    self.list_event_types(app_id).await?
                } else {
                    self.get_application(app_id).await?
                }
            }
            uri if uri.starts_with("hook0://events/") => {
                let rest = uri.strip_prefix("hook0://events/").unwrap();
                if rest.contains("/attempts") {
                    let event_id = rest.strip_suffix("/attempts").unwrap();
                    self.list_request_attempts(event_id).await?
                } else {
                    self.get_event(rest).await?
                }
            }
            _ => {
                return Err(McpError::resource_not_found(
                    format!("Resource not found: {}", request.uri),
                    None,
                ));
            }
        };

        let text = serde_json::to_string_pretty(&content).unwrap_or_else(|_| content.to_string());

        Ok(ReadResourceResult {
            contents: vec![ResourceContents::text(text, request.uri)],
        })
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        info!("Listing prompts");
        Ok(ListPromptsResult {
            prompts: prompts::list_prompts(),
            next_cursor: None,
            meta: None,
        })
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
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
