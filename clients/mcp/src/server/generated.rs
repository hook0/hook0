// The tools this server exposes, derived from the OpenAPI snapshot the API crate commits.
// Do not edit by hand: run `UPDATE_MCP_TOOLS=1 cargo test -p hook0-sdkgen mcp_tool_definitions`.

/// Information about an MCP tool, generated from OpenAPI
#[derive(Debug, Clone)]
pub struct GeneratedToolInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub method: &'static str,
    pub path_template: &'static str,
    pub input_schema: &'static str,
}

impl GeneratedToolInfo {
    /// Returns true if this is a write operation (POST, PUT, PATCH, DELETE)
    pub fn is_write_operation(&self) -> bool {
        self.method != "GET"
    }
}

/// All available MCP tools generated from OpenAPI
pub const GENERATED_TOOLS: &[GeneratedToolInfo] = &[
    GeneratedToolInfo {
        name: "applications.create",
        description: "Create a new application",
        method: "POST",
        path_template: "/api/v1/applications/",
        input_schema: "{\"properties\":{\"name\":{\"description\":\"Name of the application. Length: 2-50 characters.\",\"type\":\"string\"},\"organization_id\":{\"description\":\"UUID of the organization this application belongs to.\",\"format\":\"uuid\",\"type\":\"string\"}},\"required\":[\"name\",\"organization_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "applications.delete",
        description: "Delete an application",
        method: "DELETE",
        path_template: "/api/v1/applications/{application_id}",
        input_schema: "{\"properties\":{\"application_id\":{\"type\":\"string\"}},\"required\":[\"application_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "applications.get",
        description: "Get an application by its ID",
        method: "GET",
        path_template: "/api/v1/applications/{application_id}",
        input_schema: "{\"properties\":{\"application_id\":{\"type\":\"string\"}},\"required\":[\"application_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "applications.list",
        description: "List applications",
        method: "GET",
        path_template: "/api/v1/applications/",
        input_schema: "{\"properties\":{\"organization_id\":{\"type\":\"string\"}},\"required\":[\"organization_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "applications.update",
        description: "Edit an application",
        method: "PUT",
        path_template: "/api/v1/applications/{application_id}",
        input_schema: "{\"properties\":{\"application_id\":{\"type\":\"string\"},\"name\":{\"description\":\"Name of the application. Length: 2-50 characters.\",\"type\":\"string\"},\"organization_id\":{\"description\":\"UUID of the organization this application belongs to.\",\"format\":\"uuid\",\"type\":\"string\"}},\"required\":[\"application_id\",\"name\",\"organization_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "eventTypes.create",
        description: "Create a new event type",
        method: "POST",
        path_template: "/api/v1/event_types/",
        input_schema: "{\"properties\":{\"application_id\":{\"format\":\"uuid\",\"type\":\"string\"},\"resource_type\":{\"type\":\"string\"},\"service\":{\"type\":\"string\"},\"verb\":{\"type\":\"string\"}},\"required\":[\"application_id\",\"resource_type\",\"service\",\"verb\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "eventTypes.delete",
        description: "Delete an event type",
        method: "DELETE",
        path_template: "/api/v1/event_types/{event_type_name}",
        input_schema: "{\"properties\":{\"application_id\":{\"type\":\"string\"},\"event_type_name\":{\"type\":\"string\"}},\"required\":[\"event_type_name\",\"application_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "eventTypes.get",
        description: "Get an event type by its name",
        method: "GET",
        path_template: "/api/v1/event_types/{event_type_name}",
        input_schema: "{\"properties\":{\"application_id\":{\"type\":\"string\"},\"event_type_name\":{\"type\":\"string\"}},\"required\":[\"event_type_name\",\"application_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "eventTypes.list",
        description: "List event types",
        method: "GET",
        path_template: "/api/v1/event_types/",
        input_schema: "{\"properties\":{\"application_id\":{\"type\":\"string\"}},\"required\":[\"application_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "events.get",
        description: "Get an event by its ID",
        method: "GET",
        path_template: "/api/v1/events/{event_id}",
        input_schema: "{\"properties\":{\"application_id\":{\"type\":\"string\"},\"event_id\":{\"type\":\"string\"}},\"required\":[\"event_id\",\"application_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "events.ingest",
        description: "Ingest an event",
        method: "POST",
        path_template: "/api/v1/event/",
        input_schema: "{\"properties\":{\"application_id\":{\"description\":\"UUID of the application this event belongs to.\",\"format\":\"uuid\",\"type\":\"string\"},\"event_id\":{\"description\":\"Optional unique identifier for this event (client-generated UUID). If not provided, a UUIDv7 will be generated by the server.\",\"format\":\"uuid\",\"type\":\"string\"},\"event_type\":{\"description\":\"The type of event (e.g., 'user.created', 'order.completed'). Length: 1-200 characters.\",\"type\":\"string\"},\"labels\":{\"additionalProperties\":{\"type\":\"string\"},\"description\":\"Labels for event filtering and routing to subscriptions.\",\"type\":\"object\"},\"metadata\":{\"additionalProperties\":{\"type\":\"string\"},\"description\":\"Optional metadata key-value pairs associated with the event.\",\"type\":\"object\"},\"occurred_at\":{\"description\":\"Timestamp when the event occurred.\",\"format\":\"date-time\",\"type\":\"string\"},\"payload\":{\"description\":\"The event payload. For binary content, use base64 encoding. Max length: 699050 characters (512 KiB base64-encoded).\",\"type\":\"string\"},\"payload_content_type\":{\"description\":\"Content type of the payload. Valid values: text/plain, application/json, application/octet-stream+base64. Length: 1-100 characters.\",\"type\":\"string\"}},\"required\":[\"application_id\",\"event_type\",\"labels\",\"occurred_at\",\"payload\",\"payload_content_type\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "events.list",
        description: "List latest events",
        method: "GET",
        path_template: "/api/v1/events/",
        input_schema: "{\"properties\":{\"application_id\":{\"type\":\"string\"}},\"required\":[\"application_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "events.replay",
        description: "Replay an event",
        method: "POST",
        path_template: "/api/v1/events/{event_id}/replay",
        input_schema: "{\"properties\":{\"application_id\":{\"format\":\"uuid\",\"type\":\"string\"},\"event_id\":{\"type\":\"string\"}},\"required\":[\"event_id\",\"application_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "organizations.get",
        description: "Get organization's info by its ID",
        method: "GET",
        path_template: "/api/v1/organizations/{organization_id}/",
        input_schema: "{\"properties\":{\"organization_id\":{\"type\":\"string\"}},\"required\":[\"organization_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "organizations.list",
        description: "List organizations",
        method: "GET",
        path_template: "/api/v1/organizations/",
        input_schema: "{\"properties\":{},\"required\":[],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "payload_content_types.list",
        description: "List supported event payload content types",
        method: "GET",
        path_template: "/api/v1/payload_content_types/",
        input_schema: "{\"properties\":{},\"required\":[],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "requestAttempts.get",
        description: "Get a request attempt by its ID",
        method: "GET",
        path_template: "/api/v1/request_attempts/{request_attempt_id}",
        input_schema: "{\"properties\":{\"application_id\":{\"type\":\"string\"},\"request_attempt_id\":{\"type\":\"string\"}},\"required\":[\"application_id\",\"request_attempt_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "requestAttempts.read",
        description: "List request attempts",
        method: "GET",
        path_template: "/api/v1/request_attempts/",
        input_schema: "{\"properties\":{\"application_id\":{\"type\":\"string\"},\"event.event_type_names\":{\"description\":\"Comma-separated event types\",\"type\":\"string\"},\"event_id\":{\"type\":\"string\"},\"max_created_at\":{\"type\":\"string\"},\"min_created_at\":{\"type\":\"string\"},\"pagination_cursor\":{\"type\":\"string\"},\"subscription_id\":{\"type\":\"string\"}},\"required\":[\"application_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "subscriptions.create",
        description: "Create a new subscription",
        method: "POST",
        path_template: "/api/v1/subscriptions/",
        input_schema: "{\"properties\":{\"application_id\":{\"format\":\"uuid\",\"type\":\"string\"},\"dedicated_workers\":{\"items\":{\"type\":\"string\"},\"type\":\"array\"},\"description\":{\"type\":\"string\"},\"event_types\":{\"items\":{\"type\":\"string\"},\"type\":\"array\"},\"is_enabled\":{\"type\":\"boolean\"},\"label_key\":{\"description\":\"_Kept for backward compatibility, you should use `labels`_\",\"type\":\"string\"},\"label_value\":{\"description\":\"_Kept for backward compatibility, you should use `labels`_\",\"type\":\"string\"},\"labels\":{\"additionalProperties\":{\"type\":\"string\"},\"type\":\"object\"},\"metadata\":{\"additionalProperties\":{\"type\":\"string\"},\"type\":\"object\"},\"target\":{\"properties\":{\"headers\":{\"type\":\"object\"},\"method\":{\"type\":\"string\"},\"type\":{\"example\":\"http\",\"type\":\"string\"},\"url\":{\"format\":\"url\",\"type\":\"string\"}},\"required\":[\"headers\",\"method\",\"type\",\"url\"],\"type\":\"object\"}},\"required\":[\"application_id\",\"event_types\",\"is_enabled\",\"target\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "subscriptions.delete",
        description: "Delete a subscription",
        method: "DELETE",
        path_template: "/api/v1/subscriptions/{subscription_id}",
        input_schema: "{\"properties\":{\"application_id\":{\"type\":\"string\"},\"subscription_id\":{\"type\":\"string\"}},\"required\":[\"subscription_id\",\"application_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "subscriptions.get",
        description: "Get a subscription by its ID",
        method: "GET",
        path_template: "/api/v1/subscriptions/{subscription_id}",
        input_schema: "{\"properties\":{\"subscription_id\":{\"type\":\"string\"}},\"required\":[\"subscription_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "subscriptions.list",
        description: "List subscriptions",
        method: "GET",
        path_template: "/api/v1/subscriptions/",
        input_schema: "{\"properties\":{\"application_id\":{\"type\":\"string\"}},\"required\":[\"application_id\"],\"type\":\"object\"}",
    },
    GeneratedToolInfo {
        name: "subscriptions.update",
        description: "Update a subscription",
        method: "PUT",
        path_template: "/api/v1/subscriptions/{subscription_id}",
        input_schema: "{\"properties\":{\"application_id\":{\"format\":\"uuid\",\"type\":\"string\"},\"dedicated_workers\":{\"items\":{\"type\":\"string\"},\"type\":\"array\"},\"description\":{\"type\":\"string\"},\"event_types\":{\"items\":{\"type\":\"string\"},\"type\":\"array\"},\"is_enabled\":{\"type\":\"boolean\"},\"label_key\":{\"description\":\"_Kept for backward compatibility, you should use `labels`_\",\"type\":\"string\"},\"label_value\":{\"description\":\"_Kept for backward compatibility, you should use `labels`_\",\"type\":\"string\"},\"labels\":{\"additionalProperties\":{\"type\":\"string\"},\"type\":\"object\"},\"metadata\":{\"additionalProperties\":{\"type\":\"string\"},\"type\":\"object\"},\"subscription_id\":{\"type\":\"string\"},\"target\":{\"properties\":{\"headers\":{\"type\":\"object\"},\"method\":{\"type\":\"string\"},\"type\":{\"example\":\"http\",\"type\":\"string\"},\"url\":{\"format\":\"url\",\"type\":\"string\"}},\"required\":[\"headers\",\"method\",\"type\",\"url\"],\"type\":\"object\"}},\"required\":[\"subscription_id\",\"application_id\",\"event_types\",\"is_enabled\",\"target\"],\"type\":\"object\"}",
    },
];

/// Check if a tool name corresponds to a write operation
pub fn is_write_tool(name: &str) -> bool {
    GENERATED_TOOLS
        .iter()
        .find(|t| t.name == name)
        .map(|t| t.is_write_operation())
        .unwrap_or(false)
}

/// Get tool info by name
pub fn get_tool_info(name: &str) -> Option<&'static GeneratedToolInfo> {
    GENERATED_TOOLS.iter().find(|t| t.name == name)
}

/// Interpolate path parameters into a path template
pub fn interpolate_path(
    template: &str,
    params: &std::collections::HashMap<String, String>,
) -> String {
    let mut result = template.to_string();
    for (key, value) in params {
        let placeholder = format!("{{{key}}}");
        result = result.replace(&placeholder, value);
    }
    result
}
