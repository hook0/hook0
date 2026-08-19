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

/// The names a path template asks to have filled in.
fn placeholders_of(template: &str) -> impl Iterator<Item = &str> {
    template
        .split('/')
        .filter(|segment| segment.starts_with('{') && segment.ends_with('}'))
        .map(|segment| &segment[1..segment.len() - 1])
}

/// Whether a value is one of the two segments a URL resolves away rather than keeps.
///
/// The spellings are the URL standard's own: a dot is written `.` or `%2e` in either case, and a
/// segment is a dot segment when it is one dot or two. That is why this cannot be left to
/// escaping, which would write `%2E%2E` and change nothing about what the path then means.
fn is_dot_segment(value: &str) -> bool {
    let dots = value.to_ascii_lowercase().replace("%2e", ".");
    dots == "." || dots == ".."
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

        // A value that is a dot segment is refused rather than written.
        //
        // Escaping answers a separator, and it does not answer this. The URL standard reads `%2e`
        // as a dot when it removes dot segments, so every spelling of `..` collapses the same way
        // and the request would reach a path this tool never declared. Nothing Hook0 names is `.`
        // or `..`, so refusing costs a caller nothing, and it tells the model its call was not
        // made where a quietly rewritten path would have looked like an answer.
        for name in placeholders_of(tool_info.path_template) {
            let Some(value) = path_params.get(name) else {
                continue;
            };
            if is_dot_segment(value) {
                return Err(McpError::invalid_params(
                    format!(
                        "`{name}` is `{value}`, which names a path segment rather than a value, so the request was not made"
                    ),
                    None,
                ));
            }
        }

        // Interpolate path template with parameters
        let path = interpolate_path(tool_info.path_template, &path_params);

        // What the tool declares travels in the query string, in the order the table states it, so
        // two calls asking the same thing put the same request line on the wire. The values come
        // out of the map built above, which is every argument this server can write as text: one
        // named here and not passed is simply not asked for, which is what an optional filter left
        // out has to mean.
        let query: Vec<(String, String)> = tool_info
            .query_parameters
            .iter()
            .filter_map(|name| {
                path_params
                    .get(*name)
                    .map(|value| ((*name).to_owned(), value.clone()))
            })
            .collect();

        // Execute the appropriate HTTP method
        let result = match tool_info.method {
            "GET" => self.client.get(&path, &query).await,
            "POST" => {
                // Build request body from non-path args
                let body = self.build_request_body(args, tool_info);
                self.client.post(&path, &query, body).await
            }
            "PUT" => {
                let body = self.build_request_body(args, tool_info);
                self.client.put(&path, &query, body).await
            }
            "PATCH" => {
                let body = self.build_request_body(args, tool_info);
                self.client.patch(&path, &query, body).await
            }
            "DELETE" => self.client.delete(&path, &query).await,
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

    /// Build request body from arguments, excluding those that travel somewhere else
    ///
    /// An argument travels in exactly one place: the path, the query string, or the body. The
    /// first two are taken out here, so a parameter that reaches the API in the request line is
    /// not also written into the document beside it.
    fn build_request_body(
        &self,
        args: &Map<String, Value>,
        tool_info: &GeneratedToolInfo,
    ) -> Option<Value> {
        // Extract path parameter names from template (e.g., {application_id})
        let path_param_names: Vec<&str> = tool_info
            .path_template
            .split('/')
            .filter(|s| s.starts_with('{') && s.ends_with('}'))
            .map(|s| &s[1..s.len() - 1])
            .collect();

        // Filter out everything that travels in the request line to build body
        let body_args: Map<String, Value> = args
            .iter()
            .filter(|(key, _)| !path_param_names.contains(&key.as_str()))
            .filter(|(key, _)| !tool_info.query_parameters.contains(&key.as_str()))
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
            "hook0://organizations" => self.client.get("/organizations/", &[]).await,
            "hook0://applications" => self.client.get("/applications/", &[]).await,
            uri => {
                // The prefix is stripped where it is tested, rather than tested in a match guard
                // and stripped again in the arm: the second reading is what has to be kept in step
                // with the first, and nothing in the language keeps it there.
                if let Some(rest) = uri.strip_prefix("hook0://applications/") {
                    if let Some(app_id) = rest.strip_suffix("/events") {
                        self.client
                            .get(&format!("/applications/{}/events/", app_id), &[])
                            .await
                    } else if let Some(app_id) = rest.strip_suffix("/subscriptions") {
                        self.client
                            .get(&format!("/applications/{}/subscriptions/", app_id), &[])
                            .await
                    } else if let Some(app_id) = rest.strip_suffix("/event_types") {
                        self.client
                            .get(&format!("/applications/{}/event_types/", app_id), &[])
                            .await
                    } else {
                        self.client
                            .get(&format!("/applications/{}/", rest), &[])
                            .await
                    }
                } else if let Some(rest) = uri.strip_prefix("hook0://events/") {
                    if let Some(event_id) = rest.strip_suffix("/attempts") {
                        self.client
                            .get(&format!("/events/{}/request_attempts/", event_id), &[])
                            .await
                    } else {
                        self.client.get(&format!("/events/{}/", rest), &[]).await
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
    use super::{
        GENERATED_TOOLS, get_tool_info, interpolate_path, is_dot_segment, placeholders_of,
    };
    use serde_json::Value;
    use std::collections::{BTreeSet, HashMap};
    use url::Url;

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

    /// A real template, so that a placeholder renamed in the document is a failure here rather
    /// than a case that quietly stops testing anything.
    fn a_real_template() -> &'static str {
        let tool =
            get_tool_info("eventTypes.get").expect("the tool table carries `eventTypes.get`");
        assert!(
            tool.path_template.contains("{event_type_name}"),
            "`eventTypes.get` no longer names `event_type_name` in its path"
        );
        tool.path_template
    }

    /// Two placeholders, which no tool declares today. The helper has to be right about the second
    /// one before an operation grows one, and a case that waits for that operation to arrive is a
    /// case that tests nothing until the day it is needed.
    const TWO_PLACEHOLDERS: &str =
        "/api/v1/applications/{application_id}/event_types/{event_type_name}";

    fn written_into(template: &str, values: &[(&str, &str)]) -> String {
        let params: HashMap<String, String> = values
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect();
        interpolate_path(template, &params)
    }

    fn written(values: &[(&str, &str)]) -> String {
        written_into(a_real_template(), values)
    }

    /// Where a value ends up once the client has set it on a URL, which is the only reading that
    /// decides whether a request went where its tool said it would.
    fn asked_for(value: &str) -> String {
        let mut url = Url::parse("https://app.hook0.com").expect("the origin parses");
        url.set_path(&written(&[("event_type_name", value)]));
        url.path().to_owned()
    }

    /// An argument reaches this server from a model, which read it somewhere. A slash in one used
    /// to end the segment it was written into and start another.
    #[test]
    fn a_value_carrying_a_separator_stays_one_segment() {
        let path = written(&[("event_type_name", "../../organizations/DEADBEEF")]);
        assert!(
            !path.contains("/organizations/"),
            "a value naming another segment reached the path as one: {path}"
        );
        assert!(
            path.contains("..%2F..%2Forganizations%2FDEADBEEF"),
            "the separators in the value were not escaped: {path}"
        );
    }

    /// The half escaping cannot answer, which is why the dispatcher refuses it instead.
    ///
    /// Measured rather than assumed: `%2E%2E` is written into the path and the URL still resolves
    /// it away, because the standard reads an encoded dot as a dot when it removes dot segments.
    /// A reader who "fixes" the refusal by escaping harder can run this and watch it fail.
    #[test]
    fn escaping_a_dot_segment_does_not_stop_it_moving_the_request() {
        let mut url = Url::parse("https://app.hook0.com").expect("the origin parses");
        url.set_path("/api/v1/event_types/%2E%2E");
        assert_eq!(
            url.path(),
            "/api/v1/",
            "an encoded dot segment is no longer resolved away, so the refusal below could be \
             replaced by escaping"
        );
    }

    /// Every spelling the URL standard treats as a dot segment, and the ordinary values that must
    /// not be caught with them.
    #[test]
    fn a_dot_segment_is_recognised_in_every_spelling_it_has() {
        for refused in [".", "..", "%2e", "%2E%2E", ".%2e", "%2e."] {
            assert!(
                is_dot_segment(refused),
                "`{refused}` names a path segment and was not recognised"
            );
        }
        for kept in [
            "billing.invoice.paid",
            "...",
            "a%2eb",
            "",
            "0198f3c1-0000-7000-8000-000000000000",
        ] {
            assert!(
                !is_dot_segment(kept),
                "`{kept}` is an ordinary value and was refused"
            );
        }
    }

    /// The refusal only looks at what the template will actually write into the path. An argument
    /// of the same name travelling in the body or the query is not this rule's business.
    #[test]
    fn only_what_the_template_asks_for_is_held_to_the_rule() {
        assert_eq!(
            placeholders_of(TWO_PLACEHOLDERS).collect::<Vec<_>>(),
            vec!["application_id", "event_type_name"]
        );
        assert_eq!(
            placeholders_of("/api/v1/applications/").count(),
            0,
            "a template with no placeholder asked for one"
        );
    }

    /// An event type is named `service.resource_type.verb`, so the dots that are part of a name
    /// have to survive. Escaping them would ask the API for a name nobody has.
    #[test]
    fn a_name_carrying_dots_reaches_the_api_as_it_was_written() {
        let reached = asked_for("billing.invoice.paid");
        assert!(
            reached.ends_with("/billing.invoice.paid"),
            "an ordinary event type name did not survive: {reached}"
        );
    }

    /// The map this is handed has no order, so a value spelling another placeholder used to
    /// produce one path or another depending on which key came out first.
    #[test]
    fn what_is_written_never_depends_on_the_order_of_the_map() {
        let values = [
            ("application_id", "{event_type_name}"),
            ("event_type_name", "SECOND"),
        ];
        let once = written_into(TWO_PLACEHOLDERS, &values);
        for _ in 0..64 {
            let again = written_into(TWO_PLACEHOLDERS, &values);
            assert_eq!(once, again, "the same call wrote two different paths");
        }
        assert!(
            once.contains("%7Bevent_type_name%7D"),
            "a value spelling a placeholder was substituted a second time: {once}"
        );
    }

    /// A tool asking for a value it was not given is a question for its caller. Inventing half a
    /// path would send the request somewhere nobody asked about.
    #[test]
    fn a_placeholder_nothing_answers_is_left_alone() {
        let path = written_into(
            TWO_PLACEHOLDERS,
            &[("event_type_name", "billing.invoice.paid")],
        );
        assert!(
            path.contains("{application_id}"),
            "an unanswered placeholder was not left in place: {path}"
        );
    }
}
