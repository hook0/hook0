//! Invariants the API model holds whatever document it was built from.

use hook0_sdkgen::{ApiModel, Error, Limits, PUBLIC_TAG, Snapshot};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Map, Value, json};

mod common;

/// Seeds of past failures, replayed before anything random is drawn.
const REGRESSIONS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/proptest-regressions/api_model_properties.txt"
);

/// Largest number of component schemas a generated document declares, the error schema aside.
const SCHEMA_SLOTS: usize = 6;

/// Largest number of fields a generated schema declares.
const FIELD_SLOTS: usize = 5;

/// Largest number of values a generated enum declares.
const ENUM_SLOTS: usize = 4;

/// Largest number of objects a generated schema nests.
const NESTING_SLOTS: usize = 6;

/// Largest number of operations a generated document declares.
const OPERATION_SLOTS: usize = 8;

/// Name the generated error schema is declared under.
const ERROR_SCHEMA: &str = "Failure";

/// JSON types a generated leaf declares.
const TYPES: [&str; 4] = ["string", "integer", "boolean", "number"];

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 128,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSIONS))),
        ..ProptestConfig::default()
    }
}

#[derive(Debug, Clone)]
enum GeneratedField {
    Leaf(usize),
    Enumerated(usize),
    Nested(Vec<usize>),
    Listed(usize),
    Keyed(usize),
}

#[derive(Debug, Clone)]
struct GeneratedOperation {
    schema: usize,
    reads: bool,
    answers: Option<u16>,
    listed: bool,
}

fn generated_field() -> impl Strategy<Value = GeneratedField> {
    prop_oneof![
        4 => (0..TYPES.len()).prop_map(GeneratedField::Leaf),
        2 => (1..=ENUM_SLOTS).prop_map(GeneratedField::Enumerated),
        2 => prop::collection::vec(0..TYPES.len(), 1..FIELD_SLOTS).prop_map(GeneratedField::Nested),
        1 => (0..TYPES.len()).prop_map(GeneratedField::Listed),
        1 => (0..TYPES.len()).prop_map(GeneratedField::Keyed),
    ]
}

/// Schemas always carry a field: a component declaring none is free-form JSON rather than an
/// object type, which the model refuses by name and which is exercised where it is written down.
fn generated_schemas() -> impl Strategy<Value = Vec<Vec<GeneratedField>>> {
    prop::collection::vec(
        prop::collection::vec(generated_field(), 1..FIELD_SLOTS),
        1..SCHEMA_SLOTS,
    )
}

fn generated_operations(schemas: usize) -> impl Strategy<Value = Vec<GeneratedOperation>> {
    prop::collection::vec(
        (
            0..schemas,
            any::<bool>(),
            prop::option::of(prop_oneof![Just(200u16), Just(201), Just(204)]),
            any::<bool>(),
        )
            .prop_map(|(schema, reads, answers, listed)| GeneratedOperation {
                schema,
                reads,
                answers,
                listed,
            }),
        0..OPERATION_SLOTS,
    )
}

/// Lays the generated schemas and operations out as an OpenAPI document.
///
/// Schemas and fields are numbered, so no two of them ever reach the same derived name: what a
/// collision does is a behaviour of its own, exercised where it is written down rather than
/// stumbled upon here.
fn document(schemas: &[Vec<GeneratedField>], operations: &[GeneratedOperation]) -> Vec<u8> {
    let mut declared = Map::new();
    declared.insert(ERROR_SCHEMA.to_owned(), error_schema());

    for (index, fields) in schemas.iter().enumerate() {
        declared.insert(schema_name(index), object(fields, &schema_name(index)));
    }

    let mut paths = Map::new();
    for (index, operation) in operations.iter().enumerate() {
        let mut written = Map::new();
        written.insert(
            "operationId".to_owned(),
            json!(format!("things.get{index}")),
        );
        written.insert("responses".to_owned(), responses(operation, schemas.len()));

        if operation.reads {
            written.insert(
                "requestBody".to_owned(),
                json!({"content": {"application/json": {
                    "schema": reference(&schema_name(operation.schema % schemas.len())),
                }}}),
            );
        }

        paths.insert(format!("/p{index}"), json!({"get": Value::Object(written)}));
    }

    common::spec_with(
        Value::Object(paths),
        json!({"schemas": Value::Object(declared)}),
    )
}

fn responses(operation: &GeneratedOperation, schemas: usize) -> Value {
    let mut responses = Map::new();
    responses.insert(
        "400".to_owned(),
        json!({
            "description": "",
            "content": {"application/json": {"schema": reference(ERROR_SCHEMA)}},
        }),
    );

    if let Some(status) = operation.answers {
        let named = reference(&schema_name(operation.schema % schemas));
        let body = if operation.listed {
            json!({"type": "array", "items": named})
        } else {
            named
        };
        responses.insert(
            status.to_string(),
            json!({"description": "", "content": {"application/json": {"schema": body}}}),
        );
    }

    Value::Object(responses)
}

fn object(fields: &[GeneratedField], owner: &str) -> Value {
    let mut properties = Map::new();
    let mut required = Vec::new();

    for (index, field) in fields.iter().enumerate() {
        let name = format!("f{index}");
        properties.insert(
            name.clone(),
            field_schema(field, &format!("{owner}.{name}")),
        );
        required.push(json!(name));
    }

    json!({"type": "object", "properties": properties, "required": required})
}

fn field_schema(field: &GeneratedField, origin: &str) -> Value {
    match field {
        GeneratedField::Leaf(declared) => json!({"type": TYPES[*declared]}),
        GeneratedField::Enumerated(values) => json!({
            "type": "string",
            "enum": (0..*values).map(|value| json!(format!("v{value}"))).collect::<Vec<_>>(),
        }),
        GeneratedField::Listed(declared) => {
            json!({"type": "array", "items": {"type": TYPES[*declared]}})
        }
        GeneratedField::Keyed(declared) => {
            json!({"type": "object", "additionalProperties": {"type": TYPES[*declared]}})
        }
        GeneratedField::Nested(leaves) => {
            let nested: Vec<GeneratedField> =
                leaves.iter().copied().map(GeneratedField::Leaf).collect();
            object(&nested, origin)
        }
    }
}

/// One operation answering a failure, so a document under test carries an error contract to
/// discover whatever else it declares. A document that answers none has none, which is a behaviour
/// of its own exercised where it is written down.
fn failing(operations: &[GeneratedOperation]) -> Vec<GeneratedOperation> {
    let mut carried = vec![GeneratedOperation {
        schema: 0,
        reads: false,
        answers: None,
        listed: false,
    }];
    carried.extend_from_slice(operations);
    carried
}

/// The smallest schema carrying a catalogue, which every generated failure points at.
fn error_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"id": {"type": "string", "enum": ["one", "two"]}},
        "required": ["id"],
    })
}

fn schema_name(index: usize) -> String {
    format!("S{index}")
}

fn reference(name: &str) -> Value {
    json!({"$ref": format!("#/components/schemas/{name}")})
}

fn build(bytes: &[u8], limits: &Limits) -> Result<(Snapshot, ApiModel), Error> {
    let snapshot = Snapshot::from_bytes(bytes, PUBLIC_TAG, limits)?;
    let model = ApiModel::from_snapshot(&snapshot, limits)?;
    Ok((snapshot, model))
}

/// A document declaring one schema whose objects nest as deep as asked, and one operation
/// answering the failure the error contract is discovered from.
fn nested(depth: usize) -> Vec<u8> {
    let mut schema = json!({"type": "string"});
    for _ in 0..depth {
        schema = json!({
            "type": "object",
            "properties": {"f0": schema},
            "required": ["f0"],
        });
    }

    common::spec_with(
        json!({"/p": {"get": {
            "operationId": "things.get",
            "responses": {"400": {"description": "", "content": {"application/json": {
                "schema": reference(ERROR_SCHEMA),
            }}}},
        }}}),
        json!({"schemas": {ERROR_SCHEMA: error_schema(), "Deep": schema}}),
    )
}

proptest! {
    #![proptest_config(config())]

    /// Building twice from the same bytes gives the same thing, down to the errors.
    #[test]
    fn a_model_is_rebuilt_identically(
        schemas in generated_schemas(),
        operations in generated_operations(SCHEMA_SLOTS),
    ) {
        let bytes = document(&schemas, &operations);
        let limits = Limits::default();

        prop_assert_eq!(build(&bytes, &limits), build(&bytes, &limits));
    }

    /// Reading the types an operation carries never loses the operation itself.
    #[test]
    fn every_declared_operation_lands_in_the_model_exactly_once(
        schemas in generated_schemas(),
        operations in generated_operations(SCHEMA_SLOTS),
    ) {
        let bytes = document(&schemas, &failing(&operations));
        let declared = common::declared_operations(&bytes);
        let limits = Limits::default();

        let (snapshot, model) = build(&bytes, &limits)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(snapshot.operations().len(), declared.len());
        common::assert_model_invariants(&snapshot, &model.entities, &limits);
    }

    /// Every type a method carries is one the model declares, so no target is ever handed a name
    /// it has nothing to emit for.
    #[test]
    fn every_type_a_method_carries_is_declared(
        schemas in generated_schemas(),
        operations in generated_operations(SCHEMA_SLOTS),
    ) {
        let bytes = document(&schemas, &failing(&operations));
        let limits = Limits::default();
        let (_, model) = build(&bytes, &limits)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        for entity in model.entities.entities() {
            for method in &entity.methods {
                for shape in method.request.iter().chain(
                    method.success.iter().filter_map(|(_, shape)| shape.as_ref())
                ) {
                    for name in named_types(shape) {
                        prop_assert!(
                            model.schemas.contains_key(&name),
                            "`{}` carries `{}`, which the model does not declare",
                            method.operation_id,
                            name
                        );
                    }
                }
            }
        }
    }

    /// A document declaring more object types than the ceiling accepts is refused, and one below
    /// it never is.
    #[test]
    fn the_schema_ceiling_is_enforced(
        schemas in generated_schemas(),
        ceiling in 0usize..(SCHEMA_SLOTS + 2),
    ) {
        // The generated schemas declare objects of their own, so what the document really carries
        // is counted from a build that no ceiling stops.
        let bytes = document(&schemas, &failing(&[]));
        let loose = Limits::default();
        let Ok((_, whole)) = build(&bytes, &loose) else {
            return Ok(());
        };
        let declared = whole.schemas.len();

        let limits = Limits { max_schemas: ceiling, ..Limits::DEFAULT };
        let result = build(&bytes, &limits);

        if declared > ceiling {
            let refused = matches!(result, Err(Error::TooManySchemas { .. }));
            prop_assert!(refused, "a document above the ceiling was accepted: {:?}", result);
        } else {
            let refused = matches!(result, Err(Error::TooManySchemas { .. }));
            prop_assert!(!refused, "a document below the ceiling was refused");
        }
    }

    /// A schema nesting deeper than the ceiling accepts is refused, and one below it never is.
    #[test]
    fn the_nesting_ceiling_is_enforced(
        depth in 1usize..NESTING_SLOTS,
        ceiling in 0usize..NESTING_SLOTS,
    ) {
        let bytes = nested(depth);
        let limits = Limits { max_shape_depth: ceiling, ..Limits::DEFAULT };
        let result = build(&bytes, &limits);

        // The outermost object is read at the surface, each object it nests one level below it,
        // and the value at the bottom one level below the last of them.
        let refused = matches!(result, Err(Error::SchemaTooDeep { .. }));
        prop_assert_eq!(
            refused,
            depth > ceiling,
            "a schema nesting {} objects under a ceiling of {} was answered wrongly: {:?}",
            depth,
            ceiling,
            result
        );
    }

    /// An object carrying more fields than the ceiling accepts is refused, and one below it never
    /// is.
    #[test]
    fn the_field_ceiling_is_enforced(
        fields in 1usize..FIELD_SLOTS,
        ceiling in 0usize..FIELD_SLOTS,
    ) {
        let declared: Vec<GeneratedField> = (0..fields).map(|_| GeneratedField::Leaf(0)).collect();
        let bytes = document(&[declared], &failing(&[]));
        let limits = Limits { max_fields_per_object: ceiling, ..Limits::DEFAULT };
        let result = build(&bytes, &limits);

        let refused = matches!(result, Err(Error::TooManyFields { .. }));
        prop_assert_eq!(refused, fields > ceiling, "unexpected answer: {:?}", result);
    }

    /// An enum carrying more values than the ceiling accepts is refused, and one below it never
    /// is.
    #[test]
    fn the_enum_ceiling_is_enforced(
        values in 1usize..=ENUM_SLOTS,
        ceiling in 0usize..=ENUM_SLOTS,
    ) {
        let bytes = document(&[vec![GeneratedField::Enumerated(values)]], &failing(&[]));
        let limits = Limits { max_enum_values: ceiling, ..Limits::DEFAULT };
        let result = build(&bytes, &limits);

        // The error schema carries a catalogue of its own, which the ceiling applies to too.
        let carried = values.max(error_catalogue_size());
        let refused = matches!(result, Err(Error::TooManyEnumValues { .. }));
        prop_assert_eq!(refused, carried > ceiling, "unexpected answer: {:?}", result);
    }
}

/// How many problems the generated error schema lists.
fn error_catalogue_size() -> usize {
    error_schema()["properties"]["id"]["enum"]
        .as_array()
        .map(Vec::len)
        .unwrap_or_default()
}

/// Every type a shape names, however deeply it wraps it.
fn named_types(shape: &hook0_sdkgen::Shape) -> Vec<String> {
    use hook0_sdkgen::Shape;

    match shape {
        Shape::Named(name) => vec![name.clone()],
        Shape::Array(inner) | Shape::Map(inner) => named_types(inner),
        Shape::Object(object) => {
            let mut named = vec![object.name.clone()];
            for field in &object.fields {
                named.extend(named_types(&field.shape));
            }
            named
        }
        Shape::Scalar(_) | Shape::Enum { .. } | Shape::Json => Vec::new(),
    }
}
