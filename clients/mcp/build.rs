//! Build script for hook0-mcp
//!
//! This script fetches the OpenAPI spec from Hook0 at compile time
//! and generates Rust code for:
//! - Tool definitions (names, descriptions, schemas)
//! - Tool dispatch logic
//! - Read/write tool classification (inferred from HTTP method)
//!
//! Only operations tagged with "mcp" are included.

use openapiv3::{OpenAPI, Operation, ReferenceOr, Schema, SchemaKind, Type};
use serde::Serialize;
use std::env;
use std::fs;
use std::path::PathBuf;

const OPENAPI_URL: &str = "https://app.hook0.com/api/v1/swagger.json";

/// Operation metadata extracted from OpenAPI
#[derive(Debug, Clone, Serialize)]
struct OperationMeta {
    operation_id: String,
    method: String,
    path: String,
    summary: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
    parameters: Vec<ParameterMeta>,
    request_body_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
struct ParameterMeta {
    name: String,
    location: String, // "path", "query", "header"
    required: bool,
    description: Option<String>,
    schema_type: String,
}

fn main() {
    println!("cargo:rerun-if-env-changed=HOOK0_OPENAPI_URL");
    println!("cargo:rerun-if-env-changed=SKIP_OPENAPI_FETCH");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

    // Skip fetching in CI or when explicitly requested
    let skip_fetch = env::var("SKIP_OPENAPI_FETCH").is_ok();

    if skip_fetch {
        println!("cargo:warning=Skipping OpenAPI fetch (SKIP_OPENAPI_FETCH is set)");
        write_fallback_code(&out_dir);
        return;
    }

    // Fetch OpenAPI spec (supports HOOK0_OPENAPI_URL for local development)
    let openapi_url = env::var("HOOK0_OPENAPI_URL").unwrap_or_else(|_| OPENAPI_URL.to_string());

    let spec = match fetch_openapi_spec(&openapi_url) {
        Ok(spec) => spec,
        Err(e) => {
            println!(
                "cargo:warning=Failed to fetch OpenAPI spec: {}. Using fallback.",
                e
            );
            write_fallback_code(&out_dir);
            return;
        }
    };

    // Extract operation metadata (only operations tagged with "mcp")
    let operations = extract_mcp_operations(&spec);

    if operations.is_empty() {
        println!("cargo:warning=No MCP-tagged operations found. Using fallback.");
        write_fallback_code(&out_dir);
        return;
    }

    // Generate Rust code
    let generated_code = generate_rust_code(&operations);

    let generated_path = out_dir.join("generated.rs");
    fs::write(&generated_path, &generated_code).expect("Failed to write generated.rs");

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
        "cargo:warning=Generated MCP code with {} operations (filtered by 'mcp' tag)",
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
            ReferenceOr::Item(item) => item,
            ReferenceOr::Reference { .. } => continue,
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

            let parameters = extract_parameters(operation, spec);
            let request_body_schema = extract_request_body_schema(operation, spec);

            operations.push(OperationMeta {
                operation_id,
                method: method.to_string(),
                path: path.clone(),
                summary: operation.summary.clone(),
                description: operation.description.clone(),
                tags: operation.tags.clone(),
                parameters,
                request_body_schema,
            });
        }
    }

    operations.sort_by(|a, b| a.operation_id.cmp(&b.operation_id));
    operations
}

fn extract_parameters(operation: &Operation, _spec: &OpenAPI) -> Vec<ParameterMeta> {
    let mut params = Vec::new();

    for param_ref in &operation.parameters {
        let param = match param_ref {
            ReferenceOr::Item(p) => p,
            ReferenceOr::Reference { .. } => continue,
        };

        let (name, location, required, description, schema) = match param {
            openapiv3::Parameter::Query { parameter_data, .. } => (
                parameter_data.name.clone(),
                "query".to_string(),
                parameter_data.required,
                parameter_data.description.clone(),
                extract_schema_type(&parameter_data.format),
            ),
            openapiv3::Parameter::Path { parameter_data, .. } => (
                parameter_data.name.clone(),
                "path".to_string(),
                true, // Path params are always required
                parameter_data.description.clone(),
                extract_schema_type(&parameter_data.format),
            ),
            openapiv3::Parameter::Header { parameter_data, .. } => (
                parameter_data.name.clone(),
                "header".to_string(),
                parameter_data.required,
                parameter_data.description.clone(),
                extract_schema_type(&parameter_data.format),
            ),
            openapiv3::Parameter::Cookie { .. } => continue,
        };

        params.push(ParameterMeta {
            name,
            location,
            required,
            description,
            schema_type: schema,
        });
    }

    params
}

fn extract_schema_type(format: &openapiv3::ParameterSchemaOrContent) -> String {
    match format {
        openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => match schema_ref {
            ReferenceOr::Item(schema) => schema_to_type_string(schema),
            ReferenceOr::Reference { .. } => "string".to_string(),
        },
        openapiv3::ParameterSchemaOrContent::Content(_) => "string".to_string(),
    }
}

fn schema_to_type_string(schema: &Schema) -> String {
    match &schema.schema_kind {
        SchemaKind::Type(Type::String(_)) => "string".to_string(),
        SchemaKind::Type(Type::Number(_)) => "number".to_string(),
        SchemaKind::Type(Type::Integer(_)) => "integer".to_string(),
        SchemaKind::Type(Type::Boolean(_)) => "boolean".to_string(),
        SchemaKind::Type(Type::Array(_)) => "array".to_string(),
        SchemaKind::Type(Type::Object(_)) => "object".to_string(),
        _ => "string".to_string(),
    }
}

fn extract_request_body_schema(operation: &Operation, spec: &OpenAPI) -> Option<serde_json::Value> {
    let body = operation.request_body.as_ref()?;
    let body = match body {
        ReferenceOr::Item(b) => b,
        ReferenceOr::Reference { reference } => {
            // Resolve reference
            let ref_name = reference.strip_prefix("#/components/requestBodies/")?;
            let _bodies = spec.components.as_ref()?.request_bodies.get(ref_name)?;
            return None; // For simplicity, skip resolved refs
        }
    };

    let content = body.content.get("application/json")?;
    let schema = content.schema.as_ref()?;

    // Convert schema to JSON for tool input_schema
    match schema {
        ReferenceOr::Item(s) => schema_to_json_schema(s, spec),
        ReferenceOr::Reference { reference } => {
            // Resolve schema reference
            let ref_name = reference.strip_prefix("#/components/schemas/")?;
            let resolved = spec.components.as_ref()?.schemas.get(ref_name)?;
            match resolved {
                ReferenceOr::Item(s) => schema_to_json_schema(s, spec),
                _ => None,
            }
        }
    }
}

fn schema_to_json_schema(schema: &Schema, spec: &OpenAPI) -> Option<serde_json::Value> {
    let mut result = serde_json::Map::new();

    match &schema.schema_kind {
        SchemaKind::Type(Type::Object(obj)) => {
            result.insert("type".to_string(), serde_json::json!("object"));

            let mut properties = serde_json::Map::new();
            let mut required = Vec::new();

            for (name, prop_ref) in &obj.properties {
                let prop_schema = match prop_ref {
                    ReferenceOr::Item(s) => schema_to_json_schema(s, spec),
                    ReferenceOr::Reference { reference } => {
                        let ref_name = reference.strip_prefix("#/components/schemas/")?;
                        let resolved = spec.components.as_ref()?.schemas.get(ref_name)?;
                        match resolved {
                            ReferenceOr::Item(s) => schema_to_json_schema(s, spec),
                            _ => None,
                        }
                    }
                };
                if let Some(prop) = prop_schema {
                    properties.insert(name.clone(), prop);
                }
            }

            for req in &obj.required {
                required.push(serde_json::json!(req));
            }

            result.insert(
                "properties".to_string(),
                serde_json::Value::Object(properties),
            );
            result.insert("required".to_string(), serde_json::Value::Array(required));
        }
        SchemaKind::Type(Type::String(s)) => {
            result.insert("type".to_string(), serde_json::json!("string"));
            if let Some(desc) = &schema.schema_data.description {
                result.insert("description".to_string(), serde_json::json!(desc));
            }
            if !s.enumeration.is_empty() {
                let enum_values: Vec<_> = s.enumeration.iter().filter_map(|v| v.clone()).collect();
                result.insert("enum".to_string(), serde_json::json!(enum_values));
            }
        }
        SchemaKind::Type(Type::Integer(_)) => {
            result.insert("type".to_string(), serde_json::json!("integer"));
            if let Some(desc) = &schema.schema_data.description {
                result.insert("description".to_string(), serde_json::json!(desc));
            }
        }
        SchemaKind::Type(Type::Number(_)) => {
            result.insert("type".to_string(), serde_json::json!("number"));
            if let Some(desc) = &schema.schema_data.description {
                result.insert("description".to_string(), serde_json::json!(desc));
            }
        }
        SchemaKind::Type(Type::Boolean(_)) => {
            result.insert("type".to_string(), serde_json::json!("boolean"));
            if let Some(desc) = &schema.schema_data.description {
                result.insert("description".to_string(), serde_json::json!(desc));
            }
        }
        SchemaKind::Type(Type::Array(arr)) => {
            result.insert("type".to_string(), serde_json::json!("array"));
            if let Some(items_ref) = &arr.items {
                let items_schema = match items_ref {
                    ReferenceOr::Item(s) => schema_to_json_schema(s.as_ref(), spec),
                    ReferenceOr::Reference { .. } => None,
                };
                if let Some(items_val) = items_schema {
                    result.insert("items".to_string(), items_val);
                }
            }
        }
        _ => {
            result.insert("type".to_string(), serde_json::json!("object"));
        }
    }

    if let Some(desc) = &schema.schema_data.description {
        result.insert("description".to_string(), serde_json::json!(desc));
    }

    Some(serde_json::Value::Object(result))
}

fn generate_rust_code(operations: &[OperationMeta]) -> String {
    let mut code = String::new();

    // Header
    code.push_str("// Auto-generated MCP tool definitions from OpenAPI spec\n");
    code.push_str("// DO NOT EDIT - regenerate by running `cargo build`\n\n");

    // Generate ToolInfo struct and array
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

    // Generate the tools array
    code.push_str("/// All available MCP tools generated from OpenAPI\n");
    code.push_str("pub const GENERATED_TOOLS: &[GeneratedToolInfo] = &[\n");

    for op in operations {
        let description = op
            .summary
            .as_ref()
            .or(op.description.as_ref())
            .map(|s| escape_string(s))
            .unwrap_or_default();

        let input_schema = generate_input_schema(op);
        let escaped_schema = escape_string(&input_schema);

        code.push_str(&format!(
            "    GeneratedToolInfo {{\n        name: \"{}\",\n        description: \"{}\",\n        method: \"{}\",\n        path_template: \"{}\",\n        input_schema: \"{}\",\n    }},\n",
            op.operation_id, description, op.method, op.path, escaped_schema
        ));
    }

    code.push_str("];\n\n");

    // Generate helper function to check if a tool is a write operation
    code.push_str("/// Check if a tool name corresponds to a write operation\n");
    code.push_str("pub fn is_write_tool(name: &str) -> bool {\n");
    code.push_str("    GENERATED_TOOLS\n");
    code.push_str("        .iter()\n");
    code.push_str("        .find(|t| t.name == name)\n");
    code.push_str("        .map(|t| t.is_write_operation())\n");
    code.push_str("        .unwrap_or(false)\n");
    code.push_str("}\n\n");

    // Generate function to get tool info by name
    code.push_str("/// Get tool info by name\n");
    code.push_str("pub fn get_tool_info(name: &str) -> Option<&'static GeneratedToolInfo> {\n");
    code.push_str("    GENERATED_TOOLS.iter().find(|t| t.name == name)\n");
    code.push_str("}\n\n");

    // Generate path interpolation helper
    code.push_str("/// Interpolate path parameters into a path template\n");
    code.push_str("pub fn interpolate_path(template: &str, params: &std::collections::HashMap<String, String>) -> String {\n");
    code.push_str("    let mut result = template.to_string();\n");
    code.push_str("    for (key, value) in params {\n");
    code.push_str("        let placeholder = format!(\"{{{}}}\" , key);\n");
    code.push_str("        result = result.replace(&placeholder, value);\n");
    code.push_str("    }\n");
    code.push_str("    result\n");
    code.push_str("}\n");

    code
}

fn generate_input_schema(op: &OperationMeta) -> String {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    // Add path and query parameters
    for param in &op.parameters {
        let mut prop = serde_json::Map::new();
        prop.insert("type".to_string(), serde_json::json!(param.schema_type));
        if let Some(desc) = &param.description {
            prop.insert("description".to_string(), serde_json::json!(desc));
        }
        properties.insert(param.name.clone(), serde_json::Value::Object(prop));

        if param.required {
            required.push(serde_json::json!(param.name));
        }
    }

    // Merge request body schema if present
    if let Some(body_schema) = &op.request_body_schema {
        if let Some(body_props) = body_schema.get("properties").and_then(|p| p.as_object()) {
            for (key, value) in body_props {
                properties.insert(key.clone(), value.clone());
            }
        }
        if let Some(body_required) = body_schema.get("required").and_then(|r| r.as_array()) {
            for req in body_required {
                if !required.contains(req) {
                    required.push(req.clone());
                }
            }
        }
    }

    let schema = serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    });

    serde_json::to_string(&schema)
        .unwrap_or_else(|_| r#"{"type":"object","properties":{},"required":[]}"#.to_string())
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn write_fallback_code(out_dir: &std::path::Path) {
    // Fallback with hardcoded tools when OpenAPI is not available
    let fallback = r#"// Fallback MCP tool definitions (OpenAPI spec unavailable)
// Regenerate by running `cargo build` with network access

/// Information about an MCP tool
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

/// Fallback tools - these are placeholders until OpenAPI spec is fetched
pub const GENERATED_TOOLS: &[GeneratedToolInfo] = &[];

/// Check if a tool name corresponds to a write operation
pub fn is_write_tool(_name: &str) -> bool {
    false
}

/// Get tool info by name
pub fn get_tool_info(_name: &str) -> Option<&'static GeneratedToolInfo> {
    None
}

/// Interpolate path parameters into a path template
pub fn interpolate_path(template: &str, params: &std::collections::HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in params {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}
"#;

    let generated_path = out_dir.join("generated.rs");
    fs::write(&generated_path, fallback).expect("Failed to write fallback generated.rs");

    // Write empty metadata
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
