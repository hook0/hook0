//! Generates this server's tool definitions from the OpenAPI snapshot committed in the repository.
//!
//! Only operations tagged `mcp` become tools. The snapshot is read from disk, so the build never
//! reaches the network, and anything the build cannot make sense of — a missing snapshot, an
//! operation without a name, a body schema that resolves to nothing, a tagged operation the
//! generator would silently skip — fails the build. A server that starts with no tool, or with a
//! tool whose schema no client can read, is a defect that hides itself rather than a degraded mode.

use serde::Serialize;
use serde_json::{Map, Value, json};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// The document the tools are generated from, regenerated and guarded by the API crate.
const SNAPSHOT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../api/openapi.snapshot.json"
);

/// Tag that turns an operation into a tool.
const MCP_TAG: &str = "mcp";

/// Keys of a path item that describe an operation.
const HTTP_METHODS: [&str; 5] = ["get", "post", "put", "patch", "delete"];

/// Media type a request body is read from.
const JSON_MEDIA_TYPE: &str = "application/json";

/// Where a schema reference is looked up.
const SCHEMA_REFERENCE_PREFIX: &str = "#/components/schemas/";

/// Type a parameter falls back to when the snapshot names none.
const DEFAULT_PARAMETER_TYPE: &str = "string";

/// Largest snapshot accepted, in bytes.
const MAX_SNAPSHOT_BYTES: u64 = 8 * 1024 * 1024;

/// Largest number of tools accepted.
const MAX_TOOLS: usize = 512;

/// Longest chain of `$ref` hops followed when resolving a schema.
const MAX_REFERENCE_DEPTH: usize = 8;

/// One operation of the snapshot, as a tool needs it.
#[derive(Debug, Clone, Serialize)]
struct OperationMeta {
    operation_id: String,
    method: String,
    path: String,
    summary: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
    parameters: Vec<ParameterMeta>,
    request_body_schema: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
struct ParameterMeta {
    name: String,
    location: String,
    required: bool,
    description: Option<String>,
    schema_type: String,
}

fn main() {
    println!("cargo:rerun-if-changed={SNAPSHOT_PATH}");

    if let Err(reason) = generate() {
        panic!("{reason}");
    }
}

fn generate() -> Result<(), String> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(|_| "OUT_DIR is not set".to_owned())?);

    let snapshot = read_snapshot()?;
    let operations = tool_operations(&snapshot)?;

    if operations.is_empty() {
        return Err(format!(
            "{SNAPSHOT_PATH} carries no operation tagged `{MCP_TAG}`, so this server would expose no tool"
        ));
    }
    if operations.len() > MAX_TOOLS {
        return Err(format!(
            "{SNAPSHOT_PATH} yields {} tools, above the {MAX_TOOLS} accepted",
            operations.len()
        ));
    }

    let generated = generate_rust_code(&operations)?;
    fs::write(out_dir.join("generated.rs"), generated)
        .map_err(|err| format!("the generated tool definitions could not be written: {err}"))?;

    let metadata = json!({
        "version": snapshot["info"]["version"],
        "title": snapshot["info"]["title"],
        "operations": operations,
    });
    let metadata = serde_json::to_string_pretty(&metadata)
        .map_err(|err| format!("the tool metadata could not be serialized: {err}"))?;
    fs::write(out_dir.join("mcp_metadata.json"), metadata)
        .map_err(|err| format!("the tool metadata could not be written: {err}"))?;

    Ok(())
}

fn read_snapshot() -> Result<Value, String> {
    let path = Path::new(SNAPSHOT_PATH);

    let metadata = fs::metadata(path).map_err(|err| {
        format!(
            "{SNAPSHOT_PATH} cannot be read ({err}). This server's tools are generated from that \
             snapshot, which the API crate writes and keeps in step with its handlers."
        )
    })?;
    if metadata.len() > MAX_SNAPSHOT_BYTES {
        return Err(format!(
            "{SNAPSHOT_PATH} is {} bytes long, above the {MAX_SNAPSHOT_BYTES} accepted",
            metadata.len()
        ));
    }

    let bytes = fs::read(path).map_err(|err| format!("{SNAPSHOT_PATH} cannot be read: {err}"))?;
    serde_json::from_slice(&bytes)
        .map_err(|err| format!("{SNAPSHOT_PATH} is not valid JSON: {err}"))
}

/// The operations the snapshot marks as tools, ordered by name so a rebuild yields the same file.
fn tool_operations(snapshot: &Value) -> Result<Vec<OperationMeta>, String> {
    let paths = snapshot
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{SNAPSHOT_PATH} carries no `paths` object"))?;

    let mut operations = Vec::new();

    for (path, item) in paths {
        if !path.starts_with('/') {
            continue;
        }

        let item = item
            .as_object()
            .ok_or_else(|| format!("the path item of `{path}` is not an object"))?;

        if item.contains_key("parameters") {
            return Err(format!(
                "`{path}` declares parameters on the path itself, which this generator does not \
                 spread over the operations of that path: move them onto each operation, or teach \
                 this script to merge them"
            ));
        }

        for method in HTTP_METHODS {
            let Some(operation) = item.get(method) else {
                continue;
            };
            let operation = operation
                .as_object()
                .ok_or_else(|| format!("the `{method}` operation of `{path}` is not an object"))?;

            if !is_tagged(operation) {
                continue;
            }

            operations.push(read_operation(operation, path, method, snapshot)?);
        }
    }

    operations.sort_by(|left, right| left.operation_id.cmp(&right.operation_id));
    Ok(operations)
}

fn is_tagged(operation: &Map<String, Value>) -> bool {
    operation
        .get("tags")
        .and_then(Value::as_array)
        .is_some_and(|tags| tags.iter().any(|tag| tag == MCP_TAG))
}

fn read_operation(
    operation: &Map<String, Value>,
    path: &str,
    method: &str,
    snapshot: &Value,
) -> Result<OperationMeta, String> {
    let operation_id = operation
        .get("operationId")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            format!(
                "the `{method}` operation of `{path}` is tagged `{MCP_TAG}` but declares no \
                 operation id, and a tool cannot be named without one"
            )
        })?;

    let tags = operation
        .get("tags")
        .and_then(Value::as_array)
        .map(|tags| {
            tags.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();

    Ok(OperationMeta {
        operation_id: operation_id.to_owned(),
        method: method.to_uppercase(),
        path: path.to_owned(),
        summary: text(operation.get("summary")),
        description: text(operation.get("description")),
        tags,
        parameters: read_parameters(operation, operation_id)?,
        request_body_schema: read_request_body_schema(operation, operation_id, snapshot)?,
    })
}

fn read_parameters(
    operation: &Map<String, Value>,
    operation_id: &str,
) -> Result<Vec<ParameterMeta>, String> {
    let Some(parameters) = operation.get("parameters") else {
        return Ok(Vec::new());
    };
    let parameters = parameters
        .as_array()
        .ok_or_else(|| format!("the parameters of `{operation_id}` are not a list"))?;

    parameters
        .iter()
        .map(|parameter| read_parameter(parameter, operation_id))
        .collect()
}

fn read_parameter(parameter: &Value, operation_id: &str) -> Result<ParameterMeta, String> {
    if parameter.get("$ref").is_some() {
        return Err(format!(
            "`{operation_id}` reaches a parameter through a reference, which this generator does \
             not follow: the snapshot is expected to carry its parameters inline"
        ));
    }

    let name = parameter
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("a parameter of `{operation_id}` carries no name"))?;

    let location = parameter
        .get("in")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("parameter `{name}` of `{operation_id}` says nowhere it travels"))?;

    let required = match location {
        "path" => true,
        "query" | "header" => parameter
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        other => {
            return Err(format!(
                "parameter `{name}` of `{operation_id}` travels in `{other}`, which a tool call \
                 has no way to carry"
            ));
        }
    };

    let schema_type = parameter
        .get("schema")
        .and_then(|schema| schema.get("type"))
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_PARAMETER_TYPE);

    Ok(ParameterMeta {
        name: name.to_owned(),
        location: location.to_owned(),
        required,
        description: text(parameter.get("description")),
        schema_type: schema_type.to_owned(),
    })
}

/// The JSON body an operation reads, with its reference into the components resolved.
fn read_request_body_schema(
    operation: &Map<String, Value>,
    operation_id: &str,
    snapshot: &Value,
) -> Result<Option<Value>, String> {
    let Some(body) = operation.get("requestBody") else {
        return Ok(None);
    };

    let schema = body
        .get("content")
        .and_then(|content| content.get(JSON_MEDIA_TYPE))
        .and_then(|content| content.get("schema"))
        .ok_or_else(|| {
            format!(
                "the body of `{operation_id}` declares no `{JSON_MEDIA_TYPE}` schema, so a tool \
                 call would have nothing to fill in"
            )
        })?;

    resolve_schema(schema, operation_id, snapshot).map(Some)
}

/// Follows `$ref` hops until a schema is reached, which is also what stops a reference cycle.
fn resolve_schema(schema: &Value, operation_id: &str, snapshot: &Value) -> Result<Value, String> {
    let mut current = schema;
    let mut hops = 0;

    while let Some(reference) = current.get("$ref").and_then(Value::as_str) {
        if hops >= MAX_REFERENCE_DEPTH {
            return Err(format!(
                "the body schema of `{operation_id}` nests deeper than the {MAX_REFERENCE_DEPTH} \
                 references accepted"
            ));
        }
        hops += 1;

        let name = reference
            .strip_prefix(SCHEMA_REFERENCE_PREFIX)
            .ok_or_else(|| {
                format!("the body schema of `{operation_id}` points outside the components: `{reference}`")
            })?;

        current = snapshot
            .get("components")
            .and_then(|components| components.get("schemas"))
            .and_then(|schemas| schemas.get(name))
            .ok_or_else(|| {
                format!("the body schema of `{operation_id}` points at `{name}`, which the snapshot does not declare")
            })?;
    }

    Ok(current.clone())
}

fn text(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(str::to_owned)
}

fn generate_rust_code(operations: &[OperationMeta]) -> Result<String, String> {
    let mut code = String::new();

    code.push_str("// Auto-generated MCP tool definitions from the committed OpenAPI snapshot\n");
    code.push_str("// DO NOT EDIT - regenerate by running `cargo build`\n\n");

    code.push_str("/// Information about an MCP tool, generated from OpenAPI\n");
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str("pub struct GeneratedToolInfo {\n");
    code.push_str("    pub name: &'static str,\n");
    code.push_str("    pub description: &'static str,\n");
    code.push_str("    pub method: &'static str,\n");
    code.push_str("    pub path_template: &'static str,\n");
    code.push_str("    pub input_schema: &'static str,\n");
    code.push_str("}\n\n");

    code.push_str("impl GeneratedToolInfo {\n");
    code.push_str("    /// Returns true if this is a write operation (POST, PUT, PATCH, DELETE)\n");
    code.push_str("    pub fn is_write_operation(&self) -> bool {\n");
    code.push_str("        self.method != \"GET\"\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    code.push_str("/// All available MCP tools generated from OpenAPI\n");
    code.push_str("pub const GENERATED_TOOLS: &[GeneratedToolInfo] = &[\n");

    for operation in operations {
        let description = operation
            .summary
            .as_ref()
            .or(operation.description.as_ref())
            .map(|text| escape_string(text))
            .unwrap_or_default();

        let input_schema = generate_input_schema(operation)?;

        code.push_str(&format!(
            "    GeneratedToolInfo {{\n        name: \"{}\",\n        description: \"{}\",\n        method: \"{}\",\n        path_template: \"{}\",\n        input_schema: \"{}\",\n    }},\n",
            operation.operation_id,
            description,
            operation.method,
            operation.path,
            escape_string(&input_schema)
        ));
    }

    code.push_str("];\n\n");

    code.push_str("/// Check if a tool name corresponds to a write operation\n");
    code.push_str("pub fn is_write_tool(name: &str) -> bool {\n");
    code.push_str("    GENERATED_TOOLS\n");
    code.push_str("        .iter()\n");
    code.push_str("        .find(|t| t.name == name)\n");
    code.push_str("        .map(|t| t.is_write_operation())\n");
    code.push_str("        .unwrap_or(false)\n");
    code.push_str("}\n\n");

    code.push_str("/// Get tool info by name\n");
    code.push_str("pub fn get_tool_info(name: &str) -> Option<&'static GeneratedToolInfo> {\n");
    code.push_str("    GENERATED_TOOLS.iter().find(|t| t.name == name)\n");
    code.push_str("}\n\n");

    code.push_str("/// Interpolate path parameters into a path template\n");
    code.push_str("pub fn interpolate_path(template: &str, params: &std::collections::HashMap<String, String>) -> String {\n");
    code.push_str("    let mut result = template.to_string();\n");
    code.push_str("    for (key, value) in params {\n");
    code.push_str("        let placeholder = format!(\"{{{}}}\" , key);\n");
    code.push_str("        result = result.replace(&placeholder, value);\n");
    code.push_str("    }\n");
    code.push_str("    result\n");
    code.push_str("}\n");

    Ok(code)
}

/// The schema a caller fills in: the parameters of the operation, plus the fields of its body.
fn generate_input_schema(operation: &OperationMeta) -> Result<String, String> {
    let mut properties = Map::new();
    let mut required = Vec::new();

    for parameter in &operation.parameters {
        let mut property = Map::new();
        property.insert("type".to_owned(), json!(parameter.schema_type));
        if let Some(description) = &parameter.description {
            property.insert("description".to_owned(), json!(description));
        }
        properties.insert(parameter.name.clone(), Value::Object(property));

        if parameter.required {
            required.push(json!(parameter.name));
        }
    }

    if let Some(body) = &operation.request_body_schema {
        if let Some(fields) = body.get("properties").and_then(Value::as_object) {
            for (name, field) in fields {
                properties.insert(name.clone(), field.clone());
            }
        }
        if let Some(fields) = body.get("required").and_then(Value::as_array) {
            for name in fields {
                if !required.contains(name) {
                    required.push(name.clone());
                }
            }
        }
    }

    let schema = json!({
        "type": "object",
        "properties": properties,
        "required": required,
    });

    if let Some(reference) = first_reference(&schema) {
        return Err(format!(
            "the input schema of `{}` still points at `{reference}`, which no client of this \
             server can resolve",
            operation.operation_id
        ));
    }

    serde_json::to_string(&schema).map_err(|err| {
        format!(
            "the input schema of `{}` could not be serialized: {err}",
            operation.operation_id
        )
    })
}

/// The first `$ref` left anywhere in a schema, which a tool caller would have no way to follow.
fn first_reference(schema: &Value) -> Option<&str> {
    match schema {
        Value::Object(fields) => {
            if let Some(reference) = fields.get("$ref").and_then(Value::as_str) {
                return Some(reference);
            }
            fields.values().find_map(first_reference)
        }
        Value::Array(items) => items.iter().find_map(first_reference),
        _ => None,
    }
}

fn escape_string(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
