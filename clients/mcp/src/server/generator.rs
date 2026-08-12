//! Writes [`super::generated`] from the OpenAPI snapshot committed in the repository, and fails
//! when the two have drifted apart.
//!
//! Only operations tagged `mcp` become tools. The definitions are a committed source file rather
//! than a build artefact: the published crate carries no copy of the snapshot, and its build reads
//! nothing outside the package. The cost is that a change of the API surface reaches this server
//! only once someone regenerates the file, which is what the test at the bottom checks.
//!
//! Anything the generator cannot make sense of — an operation without a name, a body schema that
//! resolves to nothing, a tagged operation it would silently skip — fails rather than yields a
//! smaller tool list. A server that starts with no tool, or with a tool whose schema no client can
//! read, is a defect that hides itself rather than a degraded mode.

use serde_json::{Map, Value, json};
use std::fs;
use std::path::Path;

/// The document the tools are generated from, written and guarded by the API crate.
const SNAPSHOT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../api/openapi.snapshot.json"
);

/// The committed file this generator writes.
const GENERATED_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/server/generated.rs");

/// Set to any value to rewrite the generated file instead of failing on a difference.
const GENERATED_UPDATE_VAR: &str = "UPDATE_MCP_TOOLS";

/// What to run to adopt a deliberate change of the tool surface.
const GENERATED_UPDATE_COMMAND: &str =
    "UPDATE_MCP_TOOLS=1 cargo test -p hook0-mcp tool_definitions";

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

/// Largest generated file accepted, in bytes.
const MAX_GENERATED_BYTES: u64 = 4 * 1024 * 1024;

/// Largest number of tools accepted.
const MAX_TOOLS: usize = 512;

/// Longest chain of `$ref` hops followed when resolving a schema.
const MAX_REFERENCE_DEPTH: usize = 8;

/// How many differing lines a failure report lists before it stops.
const MAX_REPORTED_LINES: usize = 20;

/// How much of a differing line a failure report prints.
const MAX_RENDERED_LINE_CHARS: usize = 120;

/// One operation of the snapshot, as a tool needs it.
#[derive(Debug, Clone)]
struct OperationMeta {
    operation_id: String,
    method: String,
    path: String,
    summary: Option<String>,
    description: Option<String>,
    parameters: Vec<ParameterMeta>,
    request_body_schema: Option<Value>,
}

#[derive(Debug, Clone)]
struct ParameterMeta {
    name: String,
    required: bool,
    description: Option<String>,
    schema_type: String,
}

/// The whole content of the generated file, as the snapshot dictates it.
fn generated_source() -> Result<String, String> {
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

    generate_rust_code(&operations)
}

fn read_snapshot() -> Result<Value, String> {
    let bytes = read_bounded(Path::new(SNAPSHOT_PATH), MAX_SNAPSHOT_BYTES).map_err(|err| {
        format!(
            "{SNAPSHOT_PATH} cannot be read ({err}). This server's tools are generated from that \
             snapshot, which the API crate writes and keeps in step with its handlers."
        )
    })?;

    serde_json::from_slice(&bytes)
        .map_err(|err| format!("{SNAPSHOT_PATH} is not valid JSON: {err}"))
}

fn read_bounded(path: &Path, max_bytes: u64) -> Result<Vec<u8>, String> {
    let metadata = fs::metadata(path).map_err(|err| err.to_string())?;
    if metadata.len() > max_bytes {
        return Err(format!(
            "it is {} bytes long, above the {max_bytes} accepted",
            metadata.len()
        ));
    }

    fs::read(path).map_err(|err| err.to_string())
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
                 this generator to merge them"
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

    Ok(OperationMeta {
        operation_id: operation_id.to_owned(),
        method: method.to_uppercase(),
        path: path.to_owned(),
        summary: text(operation.get("summary")),
        description: text(operation.get("description")),
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
                format!(
                    "the body schema of `{operation_id}` points outside the components: \
                     `{reference}`"
                )
            })?;

        current = snapshot
            .get("components")
            .and_then(|components| components.get("schemas"))
            .and_then(|schemas| schemas.get(name))
            .ok_or_else(|| {
                format!(
                    "the body schema of `{operation_id}` points at `{name}`, which the snapshot \
                     does not declare"
                )
            })?;
    }

    Ok(current.clone())
}

fn text(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(str::to_owned)
}

/// The scaffolding the tool table is read through, which the snapshot has no say over.
const PRELUDE: &str = r#"// The tools this server exposes, derived from the OpenAPI snapshot the API crate commits.
// Do not edit by hand: run `UPDATE_MCP_TOOLS=1 cargo test -p hook0-mcp tool_definitions`.

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
"#;

/// What is read off the tool table, which the snapshot has no say over either.
const EPILOGUE: &str = r#"];

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
"#;

fn generate_rust_code(operations: &[OperationMeta]) -> Result<String, String> {
    let mut code = String::from(PRELUDE);

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

    code.push_str(EPILOGUE);

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

/// The lines that differ between what is committed and what the snapshot dictates, bounded so a
/// wholesale regeneration does not bury the report it is meant to explain.
fn report_difference(committed: &str, generated: &str) -> String {
    let committed: Vec<&str> = committed.lines().collect();
    let generated: Vec<&str> = generated.lines().collect();

    let mut lines = Vec::new();
    let mut reported = 0;

    for index in 0..committed.len().max(generated.len()) {
        let left = committed.get(index).copied();
        let right = generated.get(index).copied();
        if left == right {
            continue;
        }

        if reported >= MAX_REPORTED_LINES {
            lines.push("  ...".to_owned());
            break;
        }
        reported += 1;

        let number = index + 1;
        match (left, right) {
            (Some(left), Some(right)) => {
                lines.push(format!("  {number}- {}", render(left)));
                lines.push(format!("  {number}+ {}", render(right)));
            }
            (Some(left), None) => lines.push(format!("  {number}- {}", render(left))),
            (None, Some(right)) => lines.push(format!("  {number}+ {}", render(right))),
            (None, None) => {}
        }
    }

    lines.join("\n")
}

fn render(line: &str) -> String {
    if line.chars().count() <= MAX_RENDERED_LINE_CHARS {
        return line.to_owned();
    }

    let head: String = line.chars().take(MAX_RENDERED_LINE_CHARS).collect();
    format!("{head}…")
}

/// The tools this server exposes are a committed source file, which is what lets the published
/// crate build without the snapshot. Nothing keeps the two in step on its own, so a change of the
/// API surface that never reached the generated file stops here rather than at a release.
#[test]
fn tool_definitions_match_the_openapi_snapshot() {
    let generated = generated_source().unwrap_or_else(|reason| panic!("{reason}"));

    if std::env::var_os(GENERATED_UPDATE_VAR).is_some() {
        fs::write(GENERATED_PATH, &generated)
            .unwrap_or_else(|err| panic!("{GENERATED_PATH} is not writable: {err}"));
        println!("Wrote {GENERATED_PATH}");
        return;
    }

    let committed = read_bounded(Path::new(GENERATED_PATH), MAX_GENERATED_BYTES)
        .unwrap_or_else(|err| panic!("{GENERATED_PATH} cannot be read: {err}"));
    let committed = String::from_utf8(committed)
        .unwrap_or_else(|err| panic!("{GENERATED_PATH} is not UTF-8: {err}"));

    if committed == generated {
        return;
    }

    panic!(
        "the tools this server exposes are not the ones {SNAPSHOT_PATH} describes.\n\
         Adopt the change with:\n    {GENERATED_UPDATE_COMMAND}\n\
         and commit {GENERATED_PATH}.\n\
         (`-` committed, `+` what the snapshot dictates)\n{}",
        report_difference(&committed, &generated)
    );
}
