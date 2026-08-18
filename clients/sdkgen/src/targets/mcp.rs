//! Emits the tool definitions the Hook0 MCP server compiles.
//!
//! The server is published to crates.io with no copy of the OpenAPI snapshot beside it, so the
//! definitions travel as a committed source file rather than as a build artefact: this module
//! writes the file, the server merely compiles it, and a drift test keeps the two in step.
//!
//! Anything the emitter cannot make sense of — an operation without a name, a body that carries no
//! JSON schema, a schema still pointing at a reference no client can follow — stops the emission
//! rather than yielding a smaller tool list. A server that starts with no tool, or with a tool
//! whose schema no client can read, is a defect that hides itself rather than a degraded mode.

use serde_json::{Map, Value, json};

use crate::emit::{EmittedFile, FileTree, Ownership, RelativePath};
use crate::error::{Error, preview};
use crate::limits::Limits;
use crate::model::{ApiModel, EntityModel};
use crate::snapshot::{Operation, ParameterLocation, RequestBody};
use crate::targets::{Contract, Decoding, LanguageSpec, Target, rust, update_command};

/// Tag an operation carries to become a tool of the MCP server.
pub const MCP_TAG: &str = "mcp";

/// How the target is named, and what `UPDATE_SDK` accepts to rewrite it.
pub const NAME: &str = "mcp";

/// Where the server keeps the module it compiles the tool table out of.
const ROOT: &str = "clients/mcp/src/server";

/// The one file of that directory this target owns; everything beside it is hand-written.
const FILE: &str = "generated.rs";

/// The one document of the shared corpus the server this target lands in is held to.
///
/// The server sends requests to the Hook0 API, so what every request carries is a contract about
/// it, and the client that sends them sits beside the tool table under `clients/mcp`. The other
/// three documents are not: the server holds no retry policy, so a rule about what to repeat says
/// nothing it could honour; it never receives a webhook, so it verifies no signature; and it
/// applies none of the payload and response bounds the SDKs are configured with. Holding it to
/// those would make the guard wrong rather than strict.
const HELD_TO: [&str; 1] = ["request.json"];

/// The line saying where the tool table comes from, which no command in it ever changes.
const HEADER: &str =
    "// The tools this server exposes, derived from the OpenAPI snapshot the API crate commits.";

/// The scaffolding the tool table is read through, which the snapshot has no say over.
const PRELUDE: &str = r#"
/// Information about an MCP tool, generated from OpenAPI
#[derive(Debug, Clone)]
pub struct GeneratedToolInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub method: &'static str,
    pub path_template: &'static str,
    pub input_schema: &'static str,
    /// Which of the arguments travel in the query string, under the names the API reads them by.
    ///
    /// Stated rather than worked out from the schema: the schema says what a caller fills in and
    /// not where any of it goes, and a path parameter is only recognisable by the placeholder it
    /// fills. Every other argument would have to be sorted by a rule about the method, which holds
    /// for the operations the API declares today and would put an argument in the body the day one
    /// of them answers a query string and a body at once.
    pub query_parameters: &'static [&'static str],
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

/// This target, as the registry carries it.
pub(super) fn target() -> Target {
    Target {
        name: NAME,
        tag: MCP_TAG,
        root: ROOT,
        // The module the tool table sits in is hand-written apart from that one file, so nothing
        // beside it is ever swept away as stale.
        ownership: Ownership::Files,
        contract: Contract::Only(&HELD_TO),
        decoding: Decoding::PassThrough,
        language: rust(),
        emit,
    }
}

/// The tool table, as the one file this target owns.
///
/// The language is not read: every name and every schema travels as a string literal the snapshot
/// dictates, so nothing here is spelled the way Rust wants it spelled.
fn emit(_language: &LanguageSpec, model: &ApiModel) -> Result<FileTree, Error> {
    let limits = Limits::DEFAULT;

    let file = EmittedFile {
        path: RelativePath::build(FILE, &limits)?,
        contents: tool_definitions(&model.entities)?,
    };

    FileTree::build(vec![file], &limits)
}

/// The whole content of the tool definitions file, as the model dictates it.
///
/// Every operation the model carries becomes exactly one tool, named after its operation id and
/// ordered by that name, so the same model always writes the same bytes.
pub fn tool_definitions(model: &EntityModel) -> Result<String, Error> {
    let mut tools = named_operations(model)?;
    if tools.is_empty() {
        return Err(Error::EmptySelection);
    }
    tools.sort_by(|left, right| left.0.cmp(right.0));

    let mut source = format!(
        "{HEADER}\n// Do not edit by hand: run `{}`.\n{PRELUDE}",
        update_command(NAME)
    );
    for (name, operation) in tools {
        source.push_str(&tool(name, operation)?);
    }
    source.push_str(EPILOGUE);

    Ok(source)
}

/// Every operation of the model under the name its tool answers to.
///
/// An operation the `entity.verb` convention could not place still names a tool as long as it
/// declares an operation id; one that declares none stops the emission rather than disappearing.
fn named_operations(model: &EntityModel) -> Result<Vec<(&str, &Operation)>, Error> {
    let mut named: Vec<(&str, &Operation)> = model
        .entities()
        .iter()
        .flat_map(|entity| entity.methods.iter())
        .map(|method| (method.operation_id.as_str(), &method.operation))
        .collect();

    for set_aside in model.unconventional() {
        let operation = &set_aside.operation;
        let operation_id =
            operation
                .operation_id
                .as_deref()
                .ok_or_else(|| Error::UnnamedOperation {
                    location: preview(&operation.location()),
                })?;
        named.push((operation_id, operation));
    }

    Ok(named)
}

fn tool(name: &str, operation: &Operation) -> Result<String, Error> {
    let description = operation
        .summary
        .as_ref()
        .or(operation.description.as_ref())
        .map(|text| escape(text))
        .unwrap_or_default();

    Ok(format!(
        "    GeneratedToolInfo {{\n        name: \"{}\",\n        description: \"{}\",\n        method: \"{}\",\n        path_template: \"{}\",\n        input_schema: \"{}\",\n{}    }},\n",
        escape(name),
        description,
        operation.method,
        escape(&operation.path),
        escape(&input_schema(name, operation)?),
        query_parameters(operation)
    ))
}

/// Longest line the emitted table carries, which is what the committed format check enforces.
const MAX_LINE_CHARS: usize = 100;

/// How far the members of a tool sit in, and how far the items of a list of theirs sit in again.
const MEMBER_INDENT: &str = "        ";
const ITEM_INDENT: &str = "            ";

/// The arguments this operation carries in its query string, as the table states them.
///
/// The document already says where every parameter travels, and the schema a caller fills in is
/// the one place that says nothing about it: it is one flat object holding the path, the query and
/// the body at once. Writing the query names out here is what lets the server put each argument
/// where the API reads it, without inferring anything from the method.
///
/// Written on one line while one line fits, and one name per line once it does not, because that
/// is what the formatter would do to it either way — and a generated file the formatter would
/// rewrite is one the format check fails on source nobody typed.
fn query_parameters(operation: &Operation) -> String {
    let names: Vec<String> = operation
        .parameters
        .iter()
        .filter(|parameter| parameter.location == ParameterLocation::Query)
        .map(|parameter| format!("\"{}\"", escape(&parameter.name)))
        .collect();

    let together = format!(
        "{MEMBER_INDENT}query_parameters: &[{}],\n",
        names.join(", ")
    );
    if together.trim_end().chars().count() <= MAX_LINE_CHARS {
        return together;
    }

    let mut apart = format!("{MEMBER_INDENT}query_parameters: &[\n");
    for name in &names {
        apart.push_str(&format!("{ITEM_INDENT}{name},\n"));
    }
    apart.push_str(&format!("{MEMBER_INDENT}],\n"));
    apart
}

/// The schema a caller fills in: the parameters of the operation, plus the fields of its body.
fn input_schema(name: &str, operation: &Operation) -> Result<String, Error> {
    let mut properties = Map::new();
    let mut required = Vec::new();

    for parameter in &operation.parameters {
        if parameter.location == ParameterLocation::Cookie {
            return Err(Error::UnsupportedParameter {
                operation_id: preview(name),
                parameter: preview(&parameter.name),
            });
        }

        let mut property = Map::new();
        property.insert("type".to_owned(), json!(parameter.schema_type));
        if let Some(description) = &parameter.description {
            property.insert("description".to_owned(), json!(description));
        }
        properties.insert(parameter.name.clone(), Value::Object(property));

        // A path parameter is required by construction: a path template left with a hole in it
        // reaches nothing.
        if parameter.required || parameter.location == ParameterLocation::Path {
            required.push(json!(parameter.name));
        }
    }

    match &operation.request_body {
        None => {}
        Some(RequestBody::Other) => {
            return Err(Error::BodyWithoutJsonSchema {
                operation_id: preview(name),
            });
        }
        Some(RequestBody::Json(body)) => {
            if let Some(fields) = body.get("properties").and_then(Value::as_object) {
                for (field, schema) in fields {
                    properties.insert(field.clone(), schema.clone());
                }
            }
            if let Some(fields) = body.get("required").and_then(Value::as_array) {
                for field in fields {
                    if !required.contains(field) {
                        required.push(field.clone());
                    }
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
        return Err(Error::UnresolvedSchema {
            operation_id: preview(name),
            reference: preview(reference),
        });
    }

    serde_json::to_string(&schema).map_err(|err| Error::UnserializableSchema {
        subject: preview(name),
        reason: err.to_string(),
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

/// Turns snapshot text into what a Rust string literal may carry.
fn escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
