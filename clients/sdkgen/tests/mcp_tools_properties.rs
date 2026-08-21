//! Invariants the MCP emitter holds whatever document the model was built from.

use std::collections::BTreeSet;

use hook0_sdkgen::{
    EntityModel, Error, Limits, MCP_TAG, Operation, ParameterLocation, RequestBody, Snapshot, mcp,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Map, Value, json};

mod common;

/// Seeds of past failures, replayed before anything random is drawn.
const REGRESSIONS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/proptest-regressions/mcp_tools_properties.txt"
);

/// Distinct paths a generated document spreads its operations over.
const PATH_SLOTS: usize = 6;

/// Largest number of operations a generated document declares.
const OPERATION_SLOTS: usize = 12;

/// Largest number of parameters an operation declares.
const PARAMETER_SLOTS: usize = 4;

/// Largest number of fields a request body declares.
const FIELD_SLOTS: usize = 4;

/// Where a generated parameter travels, cookies included so the emitter has to refuse one.
const LOCATIONS: [&str; 4] = ["path", "query", "header", "cookie"];

/// JSON types a generated parameter declares.
const TYPES: [&str; 4] = ["string", "integer", "boolean", "number"];

/// The schema a referenced body points at, so emission has a reference to resolve.
const SHARED_BODY: &str = "Shared";

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSIONS))),
        ..ProptestConfig::default()
    }
}

#[derive(Debug, Clone)]
struct GeneratedParameter {
    name: String,
    location: usize,
    required: bool,
    described: bool,
    schema_type: Option<usize>,
}

#[derive(Debug, Clone)]
enum GeneratedBody {
    /// A JSON body written where the operation declares it.
    Inline(Vec<String>),
    /// A JSON body reached through `#/components/schemas`.
    Shared,
    /// A body in a media type the emitter has no way to describe.
    NonJson,
}

#[derive(Debug, Clone)]
struct GeneratedOperation {
    operation_id: Option<String>,
    method: usize,
    path: usize,
    tagged: bool,
    summary: Option<String>,
    parameters: Vec<GeneratedParameter>,
    body: Option<GeneratedBody>,
}

/// Operation ids spanning what the convention accepts and what it does not, since both still name
/// a tool.
fn operation_id() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-z]{1,6}\\.[a-z]{1,6}",
        "[a-z]{1,6}\\.(list|get|create|update|delete)",
        "[a-z]{1,8}",
        "[a-z]{1,4}\\.[a-z]{1,4}\\.[a-z]{1,4}",
    ]
}

fn generated_parameter() -> impl Strategy<Value = GeneratedParameter> {
    (
        "[a-z_]{1,8}",
        0..LOCATIONS.len(),
        any::<bool>(),
        any::<bool>(),
        prop::option::of(0..TYPES.len()),
    )
        .prop_map(
            |(name, location, required, described, schema_type)| GeneratedParameter {
                name,
                location,
                required,
                described,
                schema_type,
            },
        )
}

fn generated_body() -> impl Strategy<Value = GeneratedBody> {
    prop_oneof![
        8 => prop::collection::vec("[a-z_]{1,8}", 0..FIELD_SLOTS).prop_map(GeneratedBody::Inline),
        4 => Just(GeneratedBody::Shared),
        1 => Just(GeneratedBody::NonJson),
    ]
}

fn generated_operation() -> impl Strategy<Value = GeneratedOperation> {
    (
        prop::option::of(operation_id()),
        0..common::HTTP_METHODS.len(),
        0..PATH_SLOTS,
        any::<bool>(),
        prop::option::of("[a-z ]{0,20}"),
        prop::collection::vec(generated_parameter(), 0..PARAMETER_SLOTS),
        prop::option::of(generated_body()),
    )
        .prop_map(
            |(operation_id, method, path, tagged, summary, parameters, body)| GeneratedOperation {
                operation_id,
                method,
                path,
                tagged,
                summary,
                parameters,
                body,
            },
        )
}

fn generated_operations() -> impl Strategy<Value = Vec<GeneratedOperation>> {
    prop::collection::vec(generated_operation(), 0..OPERATION_SLOTS)
}

/// Lays the generated operations out as an OpenAPI document; later ones take over their slot, and
/// an id already spoken for is dropped so the document never repeats one.
fn document(operations: &[GeneratedOperation]) -> Vec<u8> {
    let mut paths = Map::new();
    let mut spoken_for = BTreeSet::new();

    for operation in operations {
        if let Some(operation_id) = operation.operation_id.as_ref()
            && !spoken_for.insert(operation_id.clone())
        {
            continue;
        }

        let mut body = Map::new();
        if let Some(operation_id) = operation.operation_id.as_ref() {
            body.insert("operationId".to_owned(), json!(operation_id));
        }
        if let Some(summary) = operation.summary.as_ref() {
            body.insert("summary".to_owned(), json!(summary));
        }
        if operation.tagged {
            body.insert("tags".to_owned(), json!([MCP_TAG]));
        }
        if !operation.parameters.is_empty() {
            body.insert("parameters".to_owned(), parameters(&operation.parameters));
        }
        if let Some(declared) = operation.body.as_ref() {
            body.insert("requestBody".to_owned(), request_body(declared));
        }

        let item = paths
            .entry(format!("/p{}", operation.path))
            .or_insert_with(|| json!({}));

        if let Some(item) = item.as_object_mut() {
            item.insert(
                common::HTTP_METHODS[operation.method].to_owned(),
                Value::Object(body),
            );
        }
    }

    common::spec_with(
        Value::Object(paths),
        json!({
            "schemas": {
                SHARED_BODY: {
                    "type": "object",
                    "properties": {"alpha": {"type": "string"}},
                    "required": ["alpha"],
                },
            },
        }),
    )
}

fn parameters(declared: &[GeneratedParameter]) -> Value {
    Value::Array(
        declared
            .iter()
            .map(|parameter| {
                let mut written = Map::new();
                written.insert("name".to_owned(), json!(parameter.name));
                written.insert("in".to_owned(), json!(LOCATIONS[parameter.location]));
                // OpenAPI has no optional path parameter, so one is always written as required.
                written.insert(
                    "required".to_owned(),
                    json!(parameter.required || LOCATIONS[parameter.location] == "path"),
                );
                if parameter.described {
                    written.insert("description".to_owned(), json!("what it stands for"));
                }
                if let Some(schema_type) = parameter.schema_type {
                    written.insert("schema".to_owned(), json!({"type": TYPES[schema_type]}));
                }
                Value::Object(written)
            })
            .collect(),
    )
}

fn request_body(declared: &GeneratedBody) -> Value {
    match declared {
        GeneratedBody::NonJson => {
            json!({"content": {"text/plain": {"schema": {"type": "string"}}}})
        }
        GeneratedBody::Shared => json!({
            "content": {
                "application/json": {
                    "schema": {"$ref": format!("#/components/schemas/{SHARED_BODY}")},
                },
            },
        }),
        GeneratedBody::Inline(fields) => {
            let mut properties = Map::new();
            for field in fields {
                properties.insert(field.clone(), json!({"type": "string"}));
            }
            json!({
                "content": {
                    "application/json": {
                        "schema": {
                            "type": "object",
                            "properties": properties,
                            "required": fields,
                        },
                    },
                },
            })
        }
    }
}

fn model(bytes: &[u8]) -> Result<EntityModel, Error> {
    let limits = Limits::default();
    let snapshot = Snapshot::from_bytes(bytes, MCP_TAG, &limits)?;
    EntityModel::from_snapshot(&snapshot, &limits)
}

/// Every operation the model carries, however the convention could place it.
fn selected(model: &EntityModel) -> Vec<&Operation> {
    model
        .entities()
        .iter()
        .flat_map(|entity| entity.methods.iter())
        .map(|method| &method.operation)
        .chain(
            model
                .unconventional()
                .iter()
                .map(|set_aside| &set_aside.operation),
        )
        .collect()
}

/// Whether the selection carries something the emitter has no way to turn into a tool.
fn carries_the_indescribable(model: &EntityModel) -> bool {
    selected(model).into_iter().any(|operation| {
        operation.operation_id.is_none()
            || operation.request_body == Some(RequestBody::Other)
            || operation
                .parameters
                .iter()
                .any(|parameter| parameter.location == ParameterLocation::Cookie)
    })
}

proptest! {
    #![proptest_config(config())]

    /// Every operation the target selects becomes one tool, named after it — none is dropped on
    /// the way out, and none is invented.
    #[test]
    fn every_selected_operation_yields_exactly_one_tool(operations in generated_operations()) {
        let Ok(model) = model(&document(&operations)) else {
            return Ok(());
        };
        let Ok(source) = mcp::tool_definitions(&model) else {
            return Ok(());
        };

        let mut emitted = common::tool_names(&source);
        let mut expected: Vec<String> = selected(&model)
            .into_iter()
            .filter_map(|operation| operation.operation_id.clone())
            .collect();

        emitted.sort();
        expected.sort();
        prop_assert_eq!(emitted, expected);
    }

    /// No two tools answer to the same name, which a client would have no way to tell apart.
    #[test]
    fn tool_names_are_unique(operations in generated_operations()) {
        let Ok(model) = model(&document(&operations)) else {
            return Ok(());
        };
        let Ok(source) = mcp::tool_definitions(&model) else {
            return Ok(());
        };

        let emitted = common::tool_names(&source);
        let distinct: BTreeSet<&String> = emitted.iter().collect();

        prop_assert_eq!(distinct.len(), emitted.len(), "a tool name is emitted twice");
    }

    /// Emitting the same model twice writes the same bytes, so a regeneration that changed
    /// nothing leaves no diff behind.
    #[test]
    fn emission_is_stable_under_re_emission(operations in generated_operations()) {
        let Ok(model) = model(&document(&operations)) else {
            return Ok(());
        };

        prop_assert_eq!(mcp::tool_definitions(&model), mcp::tool_definitions(&model));
    }

    /// A tool no client could call is never emitted quietly: the emission stops, and it only
    /// stops for something it could not describe.
    #[test]
    fn what_cannot_be_described_stops_the_emission(operations in generated_operations()) {
        let Ok(model) = model(&document(&operations)) else {
            return Ok(());
        };

        let emitted = mcp::tool_definitions(&model);

        if carries_the_indescribable(&model) {
            prop_assert!(emitted.is_err(), "an operation no tool can carry was emitted anyway");
        } else if selected(&model).is_empty() {
            prop_assert_eq!(emitted, Err(Error::EmptySelection));
        } else {
            let source = emitted.map_err(|error| TestCaseError::fail(error.to_string()))?;
            prop_assert!(
                !source.contains("$ref"),
                "the emitted source still points at a reference no caller can follow"
            );
        }
    }
}
