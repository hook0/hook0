//! Build script that generates API client code from the Hook0 OpenAPI specification.
//!
//! This script downloads the OpenAPI spec at build time and generates Rust code
//! for API method implementations. The generated code is then included in the
//! api/generated.rs module.

use heck::ToSnakeCase;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

const OPENAPI_URL: &str = "https://app.hook0.com/api/v1/swagger.json";
const OPENAPI_FALLBACK_PATH: &str = "openapi.json";

#[derive(Debug, Deserialize)]
struct OpenApiSpec {
    paths: HashMap<String, PathItem>,
    // Note: components field is parsed but unused - kept for future schema generation
    #[serde(default)]
    #[allow(dead_code)]
    components: Components,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct Components {
    #[serde(default)]
    schemas: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct PathItem {
    #[serde(default)]
    get: Option<Operation>,
    #[serde(default)]
    post: Option<Operation>,
    #[serde(default)]
    put: Option<Operation>,
    #[serde(default)]
    delete: Option<Operation>,
}

#[derive(Debug, Deserialize)]
struct Operation {
    #[serde(rename = "operationId")]
    operation_id: Option<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    parameters: Vec<Parameter>,
    #[serde(default, rename = "requestBody")]
    request_body: Option<RequestBody>,
    #[serde(default)]
    responses: HashMap<String, serde_json::Value>,
    // Note: tags field is parsed but unused - kept for future filtering
    #[serde(default)]
    #[allow(dead_code)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Parameter {
    name: String,
    #[serde(rename = "in")]
    location: String,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    schema: Option<Schema>,
}

#[derive(Debug, Deserialize)]
struct Schema {
    #[serde(rename = "type")]
    schema_type: Option<String>,
    #[serde(default)]
    format: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RequestBody {
    #[serde(default)]
    content: HashMap<String, MediaType>,
}

#[derive(Debug, Deserialize)]
struct MediaType {
    #[serde(default)]
    schema: Option<SchemaRef>,
}

#[derive(Debug, Deserialize)]
struct SchemaRef {
    #[serde(rename = "$ref")]
    reference: Option<String>,
}

/// Represents a parsed API operation ready for code generation
#[derive(Debug)]
#[allow(dead_code)]
struct ApiMethod {
    name: String,
    http_method: String,
    path: String,
    summary: String,
    // Following fields are parsed for future full client generation
    path_params: Vec<(String, String)>,        // (name, rust_type)
    query_params: Vec<(String, String, bool)>, // (name, rust_type, required)
    request_body_type: Option<String>,
    response_type: String,
    returns_optional: bool,
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=openapi.json");

    // Try to load OpenAPI spec
    let spec = load_openapi_spec();

    // Generate code
    let generated_code = generate_api_info(&spec);

    // Write to OUT_DIR
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("openapi_info.rs");
    fs::write(&dest_path, generated_code).expect("Failed to write generated code");
}

fn load_openapi_spec() -> OpenApiSpec {
    // First try local file (for offline builds and CI)
    if let Ok(content) = fs::read_to_string(OPENAPI_FALLBACK_PATH) {
        if let Ok(spec) = serde_json::from_str(&content) {
            println!(
                "cargo:warning=Using local OpenAPI spec from {}",
                OPENAPI_FALLBACK_PATH
            );
            return spec;
        }
    }

    // Try to download from URL (it saves the file internally)
    match download_spec() {
        Ok(spec) => spec,
        Err(e) => {
            println!(
                "cargo:warning=Failed to download OpenAPI spec: {}. Using empty spec.",
                e
            );
            OpenApiSpec {
                paths: HashMap::new(),
                components: Components::default(),
            }
        }
    }
}

fn download_spec() -> Result<OpenApiSpec, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(OPENAPI_URL).send().map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let text = response.text().map_err(|e| e.to_string())?;

    // Save to OUT_DIR for caching (don't modify source tree)
    if let Ok(out_dir) = env::var("OUT_DIR") {
        let cache_path = std::path::Path::new(&out_dir).join("openapi_cache.json");
        let _ = fs::write(cache_path, &text);
    }

    serde_json::from_str(&text).map_err(|e| e.to_string())
}

fn generate_api_info(spec: &OpenApiSpec) -> String {
    let methods = extract_api_methods(spec);

    let mut code = String::new();

    // Generate API info struct
    code.push_str("/// Auto-generated API information from OpenAPI spec.\n");
    code.push_str("/// This is informational and used for validation/documentation.\n");
    code.push_str("#[allow(dead_code)]\n");
    code.push_str("pub struct OpenApiInfo {\n");
    code.push_str("    pub endpoints: &'static [EndpointInfo],\n");
    code.push_str("}\n\n");

    code.push_str("#[allow(dead_code)]\n");
    code.push_str("#[derive(Debug)]\n");
    code.push_str("pub struct EndpointInfo {\n");
    code.push_str("    pub name: &'static str,\n");
    code.push_str("    pub method: &'static str,\n");
    code.push_str("    pub path: &'static str,\n");
    code.push_str("    pub summary: &'static str,\n");
    code.push_str("}\n\n");

    // Generate endpoint list
    code.push_str("#[allow(dead_code)]\n");
    code.push_str("pub const API_ENDPOINTS: &[EndpointInfo] = &[\n");

    for method in &methods {
        code.push_str(&format!(
            "    EndpointInfo {{ name: \"{}\", method: \"{}\", path: \"{}\", summary: \"{}\" }},\n",
            method.name,
            method.http_method,
            method.path,
            method
                .summary
                .replace('\\', "\\\\")
                .replace('\"', "\\\"")
                .replace('\n', " ")
        ));
    }

    code.push_str("];\n\n");

    // Generate OpenAPI info instance
    code.push_str("#[allow(dead_code)]\n");
    code.push_str("pub const OPENAPI_INFO: OpenApiInfo = OpenApiInfo {\n");
    code.push_str("    endpoints: API_ENDPOINTS,\n");
    code.push_str("};\n\n");

    // Generate method existence constants for compile-time validation
    code.push_str("/// Compile-time constants for endpoint validation\n");
    code.push_str("pub mod endpoints {\n");

    for method in &methods {
        let const_name = method.name.to_uppercase();
        code.push_str(&format!(
            "    pub const {}: &str = \"{}\";\n",
            const_name, method.path
        ));
    }

    code.push_str("}\n\n");

    // Generate a helper function to get endpoint by name
    code.push_str("#[allow(dead_code)]\n");
    code.push_str("pub fn get_endpoint(name: &str) -> Option<&'static EndpointInfo> {\n");
    code.push_str("    API_ENDPOINTS.iter().find(|e| e.name == name)\n");
    code.push_str("}\n");

    code
}

fn extract_api_methods(spec: &OpenApiSpec) -> Vec<ApiMethod> {
    let mut methods = Vec::new();

    for (path, item) in &spec.paths {
        if let Some(op) = &item.get {
            if let Some(method) = parse_operation("GET", path, op) {
                methods.push(method);
            }
        }
        if let Some(op) = &item.post {
            if let Some(method) = parse_operation("POST", path, op) {
                methods.push(method);
            }
        }
        if let Some(op) = &item.put {
            if let Some(method) = parse_operation("PUT", path, op) {
                methods.push(method);
            }
        }
        if let Some(op) = &item.delete {
            if let Some(method) = parse_operation("DELETE", path, op) {
                methods.push(method);
            }
        }
    }

    // Sort by name for consistent output
    methods.sort_by(|a, b| a.name.cmp(&b.name));
    methods
}

fn parse_operation(http_method: &str, path: &str, op: &Operation) -> Option<ApiMethod> {
    let operation_id = op.operation_id.as_ref()?;

    // Convert operationId to snake_case function name
    let name = operation_id.to_snake_case();

    // Extract path parameters
    let path_params: Vec<(String, String)> = op
        .parameters
        .iter()
        .filter(|p| p.location == "path")
        .map(|p| {
            let rust_type = schema_to_rust_type(p.schema.as_ref());
            (p.name.clone(), rust_type)
        })
        .collect();

    // Extract query parameters
    let query_params: Vec<(String, String, bool)> = op
        .parameters
        .iter()
        .filter(|p| p.location == "query")
        .map(|p| {
            let rust_type = schema_to_rust_type(p.schema.as_ref());
            (p.name.clone(), rust_type, p.required)
        })
        .collect();

    // Extract request body type
    let request_body_type = op.request_body.as_ref().and_then(|rb| {
        rb.content
            .get("application/json")
            .and_then(|mt| mt.schema.as_ref())
            .and_then(|s| s.reference.as_ref())
            .map(|r| r.replace("#/components/schemas/", ""))
    });

    // Determine response type
    let response_type = determine_response_type(&op.responses);
    let returns_optional = op.responses.contains_key("404");

    let summary = op
        .summary
        .clone()
        .or_else(|| op.description.clone())
        .unwrap_or_default();

    Some(ApiMethod {
        name,
        http_method: http_method.to_string(),
        path: path.to_string(),
        summary,
        path_params,
        query_params,
        request_body_type,
        response_type,
        returns_optional,
    })
}

fn schema_to_rust_type(schema: Option<&Schema>) -> String {
    match schema {
        Some(s) => match (s.schema_type.as_deref(), s.format.as_deref()) {
            (Some("string"), Some("uuid")) => "Uuid".to_string(),
            (Some("string"), Some("date-time")) => "DateTime<Utc>".to_string(),
            (Some("string"), _) => "String".to_string(),
            (Some("integer"), _) => "i64".to_string(),
            (Some("boolean"), _) => "bool".to_string(),
            (Some("array"), _) => "Vec<serde_json::Value>".to_string(),
            (Some("object"), _) => "serde_json::Value".to_string(),
            _ => "String".to_string(),
        },
        None => "String".to_string(),
    }
}

fn determine_response_type(responses: &HashMap<String, serde_json::Value>) -> String {
    // Check 200 or 201 response for type
    for status in ["200", "201"] {
        if let Some(response) = responses.get(status) {
            if let Some(content) = response.get("content") {
                if let Some(json) = content.get("application/json") {
                    if let Some(schema) = json.get("schema") {
                        if let Some(ref_path) = schema.get("$ref") {
                            if let Some(ref_str) = ref_path.as_str() {
                                return ref_str.replace("#/components/schemas/", "");
                            }
                        }
                        if let Some("array") = schema.get("type").and_then(|t| t.as_str()) {
                            if let Some(items) = schema.get("items") {
                                if let Some(ref_path) = items.get("$ref") {
                                    if let Some(ref_str) = ref_path.as_str() {
                                        return format!(
                                            "Vec<{}>",
                                            ref_str.replace("#/components/schemas/", "")
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 204 No Content
    if responses.contains_key("204") {
        return "()".to_string();
    }

    "serde_json::Value".to_string()
}
