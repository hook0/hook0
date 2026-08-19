//! Black-box suite over the whole API model: the types the document declares, the error contract
//! its failures follow, and the credentials it accepts.
//!
//! What the committed snapshot says is read back out of the document rather than written down
//! here, so a rename in the API moves both sides of an assertion at once and only a change of
//! *behaviour* fails a test.

use std::collections::{BTreeMap, BTreeSet};

use hook0_sdkgen::{
    ApiModel, Error, IGNORED_KEYWORDS, Limits, MODELLED_KEYWORDS, SDK_TAG, Scalar, Scheme, Shape,
    Snapshot,
};
use serde_json::{Value, json};

mod common;

/// Lowest status the document answers a failure with.
const FAILURE_STATUS: u64 = 400;

fn built(bytes: &[u8]) -> ApiModel {
    let limits = Limits::default();
    let snapshot =
        Snapshot::from_bytes(bytes, SDK_TAG, &limits).expect("the document under test parses");
    ApiModel::from_snapshot(&snapshot, &limits).expect("the document under test yields a model")
}

fn refused(bytes: &[u8]) -> Error {
    let limits = Limits::default();
    let snapshot =
        Snapshot::from_bytes(bytes, SDK_TAG, &limits).expect("the document under test parses");
    ApiModel::from_snapshot(&snapshot, &limits).expect_err("the document under test is refused")
}

fn fixture() -> Value {
    serde_json::from_slice(&common::fixture_bytes()).expect("the committed snapshot is JSON")
}

#[test]
fn the_committed_snapshot_yields_a_model_of_the_whole_api() {
    let document = fixture();
    let model = built(&common::fixture_bytes());

    assert_eq!(model.title, document["info"]["title"]);
    assert_eq!(model.version, document["info"]["version"]);
    assert_eq!(
        model.servers,
        document["servers"]
            .as_array()
            .expect("the committed snapshot declares servers")
            .iter()
            .map(|server| server["url"].as_str().unwrap_or_default().to_owned())
            .collect::<Vec<_>>()
    );

    for name in declared_schemas(&document).keys() {
        assert!(
            model.schemas.contains_key(name),
            "component schema `{name}` reaches no type"
        );
    }
}

#[test]
fn the_committed_snapshot_is_rebuilt_identically() {
    assert_eq!(
        built(&common::fixture_bytes()),
        built(&common::fixture_bytes()),
        "two builds of the same snapshot differ"
    );
}

/// The error contract is found by what the operations point at. Nothing in the crate names the
/// schema, so this test does not name it either: it reads the answer out of the document and asks
/// the model for the same one.
#[test]
fn the_error_contract_of_the_committed_snapshot_is_discovered() {
    let document = fixture();
    let model = built(&common::fixture_bytes());

    let (declared_schema, declared_statuses) = declared_failures(&document);
    assert_eq!(model.errors.schema, declared_schema);
    assert_eq!(model.errors.statuses, declared_statuses);

    let (discriminant, values) = declared_catalogue(&document, &declared_schema);
    assert_eq!(model.errors.discriminant, discriminant);
    assert_eq!(
        model.errors.catalogue.len(),
        values.len(),
        "the catalogue does not list what the document declares"
    );
    assert!(!model.errors.catalogue.is_empty());
    assert_eq!(
        model.errors.catalogue.values().to_vec(),
        values.into_iter().collect::<Vec<_>>(),
        "the catalogue is not the enum the document declares"
    );

    assert_eq!(model.errors.shape.name, declared_schema);
    assert!(
        model
            .errors
            .shape
            .fields
            .iter()
            .any(|field| field.name == model.errors.discriminant),
        "the discriminant names no field of the error schema"
    );
}

/// The error schema is also answered on success by the operation that lists the problems the API
/// can report. Reading responses below 400 as part of the error contract would make that operation
/// look like a failure, and reading them as candidates would make every document ambiguous.
#[test]
fn a_success_answering_the_error_schema_is_not_read_as_a_failure() {
    let document = fixture();
    let model = built(&common::fixture_bytes());
    let schema = model.errors.schema.clone();

    let listing = model
        .entities
        .entities()
        .iter()
        .flat_map(|entity| entity.methods.iter())
        .find(|method| {
            matches!(
                method.success.as_ref(),
                Some((_, Some(Shape::Array(items)))) if **items == Shape::Named(schema.clone())
            )
        })
        .expect("the committed snapshot answers a list of problems on success somewhere");

    let (status, _) = listing.success.as_ref().expect("the method succeeds");
    assert!(
        u64::from(*status) < FAILURE_STATUS,
        "a success was read under a failing status"
    );
    assert!(
        !model.errors.statuses.contains(status),
        "the status of a success reached the error contract"
    );

    // The very same document still discovers one error schema, which is the one it answers
    // failures with rather than the one it happens to list on success.
    assert_eq!(model.errors.schema, declared_failures(&document).0);
}

/// Nothing about the type language may be applied by omission: a keyword the reader does not model
/// stops the read. This keeps that refusal from freezing into a list nobody revisits — every
/// keyword the committed document actually writes has to be one the reader knows about, so a
/// keyword that appears in a later snapshot fails here, by name.
#[test]
fn every_schema_keyword_of_the_committed_snapshot_is_modelled_or_knowingly_ignored() {
    let known: BTreeSet<String> = MODELLED_KEYWORDS
        .into_iter()
        .chain(IGNORED_KEYWORDS)
        .map(str::to_owned)
        .collect();
    let census = schema_keywords(&fixture());

    assert!(
        !census.is_empty(),
        "the committed snapshot declares no schema at all"
    );

    let unknown: Vec<&String> = census.difference(&known).collect();
    assert!(
        unknown.is_empty(),
        "the committed snapshot declares schema keywords the type language neither models nor \
         knowingly ignores: {unknown:?}"
    );
}

/// The census reads a parameter wherever a parameter may be written.
///
/// Asked of a document written for it rather than of the committed one, since the committed one
/// declares no shared parameter today and a guard proven only against a document that cannot
/// exercise it is proven against nothing.
#[test]
fn the_census_reads_a_parameter_written_on_the_path_item() {
    let document: Value = serde_json::from_slice(&common::spec_with_paths(json!({
        "/things/{thing_id}": {
            "parameters": [{
                "name": "thing_id",
                "in": "path",
                "required": true,
                "schema": {"type": "string", "deprecated": true},
            }],
            "get": {"operationId": "things.read"},
        },
    })))
    .expect("the document under test is JSON");

    let census = schema_keywords(&document);
    assert!(
        census.contains("deprecated"),
        "a keyword on a shared parameter went uncounted; the census saw {census:?}"
    );
}

/// A schema written where it is used is declared under the name of what owns it followed by the
/// member it sits in, so a language that has to name every type has one for it.
#[test]
fn a_schema_written_where_it_is_used_is_declared_under_a_derived_name() {
    let model = built(&common::fixture_bytes());
    let objects = declared_objects(&fixture());
    let components = declared_schemas(&fixture());

    let derived: Vec<&String> = objects
        .keys()
        .filter(|name| !components.contains_key(*name))
        .collect();
    assert!(
        !derived.is_empty(),
        "the committed snapshot writes no schema where it is used, so this proves nothing"
    );

    assert_eq!(
        objects.keys().collect::<BTreeSet<&String>>(),
        model.schemas.keys().collect::<BTreeSet<&String>>(),
        "the types the model carries are not the object schemas the document declares"
    );
}

/// Every format the document writes reaches the scalar that stands for it, and a format no target
/// has a type for falls back to the type it is written on rather than stopping the read.
#[test]
fn declared_formats_reach_the_scalar_that_stands_for_them() {
    let model = built(&common::fixture_bytes());
    let document = fixture();
    let expected: BTreeMap<&str, Scalar> = [
        ("uuid", Scalar::Uuid),
        ("date-time", Scalar::DateTime),
        ("date", Scalar::Date),
        ("url", Scalar::Url),
        ("int32", Scalar::Integer32),
        ("int64", Scalar::Integer64),
    ]
    .into_iter()
    .collect();

    let mut walked: BTreeSet<&str> = BTreeSet::new();
    for (owner, schema) in declared_objects(&document) {
        let object = model
            .schemas
            .get(&owner)
            .unwrap_or_else(|| panic!("`{owner}` reaches no type"));

        for (member, declared) in properties_of(&schema) {
            let Some(format) = declared.get("format").and_then(Value::as_str) else {
                continue;
            };
            let Some((format, scalar)) = expected.get_key_value(format) else {
                continue;
            };
            let field = object
                .fields
                .iter()
                .find(|field| field.name == member)
                .unwrap_or_else(|| panic!("`{owner}.{member}` reaches no field"));

            assert_eq!(
                field.shape,
                Shape::Scalar(*scalar),
                "`{owner}.{member}` declares the format `{format}` yet reaches another scalar"
            );
            walked.insert(format);
        }
    }

    assert_eq!(
        walked,
        expected.keys().copied().collect::<BTreeSet<&str>>(),
        "the committed snapshot no longer declares every format the type language maps"
    );
}

/// A field is optional when the document leaves it out of `required`, and nothing else says so.
#[test]
fn optionality_is_membership_in_required() {
    let model = built(&common::fixture_bytes());
    let document = fixture();

    for (owner, schema) in declared_objects(&document) {
        let object = model
            .schemas
            .get(&owner)
            .unwrap_or_else(|| panic!("`{owner}` reaches no type"));
        let required: BTreeSet<String> = schema
            .get("required")
            .and_then(Value::as_array)
            .map(|fields| {
                fields
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();

        for field in &object.fields {
            assert_eq!(
                field.required,
                required.contains(&field.name),
                "`{owner}.{}` is mandatory in the model and not in the document, or the reverse",
                field.name
            );
        }
    }
}

/// A body reached through a component keeps that component's name, which is the whole point of
/// carrying it: a language that names its argument types has one to name.
#[test]
fn a_body_reached_through_a_component_keeps_its_name() {
    let model = built(&common::fixture_bytes());
    let named: Vec<&Shape> = model
        .entities
        .entities()
        .iter()
        .flat_map(|entity| entity.methods.iter())
        .filter_map(|method| method.request.as_ref())
        .collect();

    assert!(
        !named.is_empty(),
        "no method of the committed snapshot reads a body"
    );
    for request in named {
        let Shape::Named(name) = request else {
            panic!("a body reached through a component lost its name: {request:?}");
        };
        assert!(
            model.schemas.contains_key(name),
            "`{name}` names no type the model carries"
        );
    }
}

/// The credential the API accepts travels the way the document says, and carries the prefix the
/// document cannot state.
#[test]
fn the_credential_of_the_committed_snapshot_travels_as_declared() {
    let model = built(&common::fixture_bytes());
    let document = fixture();
    let declared = document["components"]["securitySchemes"]
        .as_object()
        .expect("the committed snapshot declares security schemes");

    assert_eq!(model.security.schemes.len(), declared.len());

    for (name, scheme) in declared {
        let read = model
            .security
            .schemes
            .get(name)
            .unwrap_or_else(|| panic!("`{name}` reaches no scheme"));

        match scheme["in"].as_str() {
            Some("header") => {
                let Scheme::Header {
                    name: header,
                    prefix,
                } = read
                else {
                    panic!("`{name}` travels in a header yet was read as {read:?}");
                };
                assert_eq!(Some(header.as_str()), scheme["name"].as_str());
                assert!(
                    prefix.is_some(),
                    "the credential travels in `{header}` with nothing in front of it"
                );
            }
            Some("query") => assert!(matches!(read, Scheme::Query { .. })),
            _ => panic!("`{name}` travels somewhere this test does not describe"),
        }
    }

    for entity in model.entities.entities() {
        for method in &entity.methods {
            for required in &method.operation.security {
                assert!(
                    model.security.schemes.contains_key(required),
                    "`{}` requires `{required}`, which the snapshot does not declare",
                    method.operation_id
                );
            }
        }
    }
}

#[test]
fn operations_disagreeing_on_their_error_schema_are_refused() {
    let error = refused(&common::spec_with(
        json!({
            "/first": {"get": {
                "operationId": "first.get",
                "responses": {"400": {"description": "", "content": {"application/json": {
                    "schema": {"$ref": "#/components/schemas/Alpha"},
                }}}},
            }},
            "/second": {"get": {
                "operationId": "second.get",
                "responses": {"400": {"description": "", "content": {"application/json": {
                    "schema": {"$ref": "#/components/schemas/Beta"},
                }}}},
            }},
        }),
        json!({"schemas": {
            "Alpha": problem_like("alpha"),
            "Beta": problem_like("beta"),
        }}),
    ));

    let Error::DisagreeingErrorSchemas { candidates } = &error else {
        panic!("unexpected error: {error}");
    };
    assert!(candidates.contains("Alpha"), "unexpected error: {error}");
    assert!(candidates.contains("Beta"), "unexpected error: {error}");
}

#[test]
fn an_error_schema_declaring_two_closed_enums_is_refused() {
    let error = refused(&common::spec_with(
        json!({
            "/first": {"get": {
                "operationId": "first.get",
                "responses": {"400": {"description": "", "content": {"application/json": {
                    "schema": {"$ref": "#/components/schemas/Alpha"},
                }}}},
            }},
        }),
        json!({"schemas": {"Alpha": {
            "type": "object",
            "properties": {
                "id": {"type": "string", "enum": ["one", "two"]},
                "kind": {"type": "string", "enum": ["three", "four"]},
            },
            "required": ["id", "kind"],
        }}}),
    ));

    let Error::AmbiguousErrorCatalogue { schema, members } = &error else {
        panic!("unexpected error: {error}");
    };
    assert_eq!(schema, "Alpha");
    assert!(members.contains("id"), "unexpected error: {error}");
    assert!(members.contains("kind"), "unexpected error: {error}");
}

#[test]
fn an_error_schema_declaring_no_closed_enum_is_refused() {
    let error = refused(&common::spec_with(
        json!({
            "/first": {"get": {
                "operationId": "first.get",
                "responses": {"400": {"description": "", "content": {"application/json": {
                    "schema": {"$ref": "#/components/schemas/Alpha"},
                }}}},
            }},
        }),
        json!({"schemas": {"Alpha": {
            "type": "object",
            "properties": {"detail": {"type": "string"}},
            "required": ["detail"],
        }}}),
    ));

    assert!(
        matches!(&error, Error::ErrorSchemaWithoutCatalogue { schema } if schema == "Alpha"),
        "unexpected error: {error}"
    );
}

#[test]
fn a_document_answering_no_failure_has_no_error_contract_to_discover() {
    let error = refused(&common::spec_with_paths(json!({
        "/first": {"get": {"operationId": "first.get"}},
    })));

    assert!(
        matches!(error, Error::UndiscoverableErrorSchema { .. }),
        "unexpected error: {error}"
    );
}

/// Two schemas written where they are used can reach the same derived name. Merging them would
/// give a generated client one type where the API has two, so it stops the read instead.
#[test]
fn schemas_colliding_under_the_derived_name_are_refused() {
    let error = refused(&common::spec_with(
        json!({
            "/first": {"get": {
                "operationId": "first.get",
                "responses": {"400": {"description": "", "content": {"application/json": {
                    "schema": {"$ref": "#/components/schemas/Alpha"},
                }}}},
            }},
        }),
        // Everything else about this document holds, so the collision is the only thing that can
        // stop it: with the two objects merged instead, it would build and one of them would be
        // gone without a word.
        json!({"schemas": {
            "Alpha": {
                "type": "object",
                "properties": {
                    "id": {"type": "string", "enum": ["one"]},
                    "beta_gamma": {
                        "type": "object",
                        "properties": {"left": {"type": "string"}},
                    },
                },
                "required": ["id"],
            },
            "AlphaBeta": {
                "type": "object",
                "properties": {"gamma": {
                    "type": "object",
                    "properties": {"right": {"type": "string"}},
                }},
            },
        }}),
    ));

    let Error::SchemaNameCollision {
        name,
        first,
        second,
    } = &error
    else {
        panic!("unexpected error: {error}");
    };
    assert_eq!(name, "AlphaBetaGamma");
    assert_ne!(first, second, "a collision was reported against itself");
}

/// A keyword the type language does not model is never applied by omission.
#[test]
fn a_schema_keyword_the_type_language_does_not_model_is_refused() {
    let unmodelled = "nullable";
    assert!(
        !MODELLED_KEYWORDS.contains(&unmodelled) && !IGNORED_KEYWORDS.contains(&unmodelled),
        "this test no longer exercises an unmodelled keyword"
    );

    let error = refused(&common::spec_with(
        json!({
            "/first": {"get": {
                "operationId": "first.get",
                "responses": {"400": {"description": "", "content": {"application/json": {
                    "schema": {"$ref": "#/components/schemas/Alpha"},
                }}}},
            }},
        }),
        json!({"schemas": {"Alpha": {
            "type": "object",
            "properties": {
                "id": {"type": "string", "enum": ["one"]},
                "detail": {"type": "string", unmodelled: true},
            },
            "required": ["id"],
        }}}),
    ));

    assert!(
        matches!(&error, Error::UnmodelledSchemaKeyword { keyword, .. } if keyword == unmodelled),
        "unexpected error: {error}"
    );
}

/// The smallest schema that carries a catalogue, so a document under test only spells out what the
/// behaviour it exercises needs.
fn problem_like(value: &str) -> Value {
    json!({
        "type": "object",
        "properties": {"id": {"type": "string", "enum": [value]}},
        "required": ["id"],
    })
}

/// The schemas the raw document declares, read straight from JSON rather than from the crate.
fn declared_schemas(document: &Value) -> BTreeMap<String, Value> {
    document["components"]["schemas"]
        .as_object()
        .map(|schemas| {
            schemas
                .iter()
                .map(|(name, schema)| (name.clone(), schema.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn properties_of(schema: &Value) -> BTreeMap<String, Value> {
    schema
        .get("properties")
        .and_then(Value::as_object)
        .map(|properties| {
            properties
                .iter()
                .map(|(name, schema)| (name.clone(), schema.clone()))
                .collect()
        })
        .unwrap_or_default()
}

/// Every object schema the document declares, under the name it should be declared as: the name of
/// the component for the ones `components.schemas` names, and the name of what owns it followed by
/// the member it sits in for the ones written where they are used.
///
/// The rule is spelled out here rather than read from the crate, so what it produces is checked
/// against something other than itself.
fn declared_objects(document: &Value) -> BTreeMap<String, Value> {
    let mut objects = BTreeMap::new();

    for (name, schema) in declared_schemas(document) {
        collect_objects(&schema, &name, &mut objects);
    }

    objects
}

fn collect_objects(schema: &Value, name: &str, objects: &mut BTreeMap<String, Value>) {
    let Some(members) = schema.as_object() else {
        return;
    };

    // An array and a map carry the name of the member they sit in, so what they hold is declared
    // under it rather than under a name of its own.
    if let Some(items) = members.get("items") {
        collect_objects(items, name, objects);
        return;
    }
    if let Some(values) = members.get("additionalProperties") {
        collect_objects(values, name, objects);
        return;
    }

    if !members.contains_key("properties") {
        return;
    }

    objects.insert(name.to_owned(), schema.clone());
    for (member, property) in properties_of(schema) {
        collect_objects(
            &property,
            &format!("{name}{}", pascal_case(&member)),
            objects,
        );
    }
}

/// The schema every failing response of the document points at, and the statuses it answers them
/// under, read from the document.
fn declared_failures(document: &Value) -> (String, Vec<u16>) {
    let mut schemas = BTreeSet::new();
    let mut statuses = BTreeSet::new();

    for operation in operations(document) {
        for (status, response) in operation
            .get("responses")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            let Ok(status) = status.parse::<u64>() else {
                continue;
            };
            if status < FAILURE_STATUS {
                continue;
            }
            statuses.insert(status as u16);

            if let Some(reference) = response["content"]["application/json"]["schema"]["$ref"]
                .as_str()
                .and_then(|reference| reference.rsplit('/').next())
            {
                schemas.insert(reference.to_owned());
            }
        }
    }

    assert_eq!(
        schemas.len(),
        1,
        "the committed snapshot answers failures with more than one schema"
    );
    let schema = schemas
        .into_iter()
        .next()
        .expect("the committed snapshot answers failures with a schema");

    (schema, statuses.into_iter().collect())
}

/// The single closed string enum the named schema declares, and the field it sits in.
fn declared_catalogue(document: &Value, schema: &str) -> (String, BTreeSet<String>) {
    let declared = &document["components"]["schemas"][schema];
    let mut found = Vec::new();

    for (member, property) in properties_of(declared) {
        let Some(values) = property.get("enum").and_then(Value::as_array) else {
            continue;
        };
        found.push((
            member,
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<BTreeSet<String>>(),
        ));
    }

    assert_eq!(
        found.len(),
        1,
        "`{schema}` declares no single closed enum to list its problems"
    );
    found.into_iter().next().expect("one enum was found")
}

/// The path items the raw document declares, which are where a parameter shared by the operations
/// under one path is written.
fn path_items(document: &Value) -> Vec<Value> {
    let Some(paths) = document.get("paths").and_then(Value::as_object) else {
        return Vec::new();
    };
    paths
        .iter()
        .filter(|(path, _)| path.starts_with('/'))
        .filter_map(|(_, item)| item.as_object().map(|_| item.clone()))
        .collect()
}

/// The operations the raw document declares, whatever the tag says about them.
fn operations(document: &Value) -> Vec<Value> {
    let mut declared = Vec::new();
    let Some(paths) = document.get("paths").and_then(Value::as_object) else {
        return declared;
    };

    for (path, item) in paths {
        let Some(item) = item.as_object().filter(|_| path.starts_with('/')) else {
            continue;
        };
        for method in common::HTTP_METHODS {
            if let Some(operation) = item.get(method) {
                declared.push(operation.clone());
            }
        }
    }

    declared
}

/// Every keyword the document writes in a position the type language reads a schema from.
fn schema_keywords(document: &Value) -> BTreeSet<String> {
    let mut census = BTreeSet::new();

    for (_, schema) in declared_schemas(document) {
        walk_schema(&schema, &mut census);
    }

    // Both places a parameter may be written. The reader takes them from the path item as well as
    // from the operation, and this walked the operations alone, so a keyword written on a shared
    // parameter was invisible to the one guard whose whole job is to see every keyword.
    for item in path_items(document) {
        for parameter in item
            .get("parameters")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            walk_schema(&parameter["schema"], &mut census);
        }
    }

    for operation in operations(document) {
        for parameter in operation
            .get("parameters")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            walk_schema(&parameter["schema"], &mut census);
        }
        walk_bodies(&operation["requestBody"], &mut census);
        for (_, response) in operation
            .get("responses")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            walk_bodies(response, &mut census);
        }
    }

    census
}

fn walk_bodies(carrier: &Value, census: &mut BTreeSet<String>) {
    let Some(content) = carrier.get("content").and_then(Value::as_object) else {
        return;
    };
    for media in content.values() {
        walk_schema(&media["schema"], census);
    }
}

/// Collects the keywords of a schema, descending only where the type language descends.
fn walk_schema(schema: &Value, census: &mut BTreeSet<String>) {
    let Some(members) = schema.as_object() else {
        return;
    };

    for keyword in members.keys() {
        census.insert(keyword.clone());
    }
    for property in properties_of(schema).values() {
        walk_schema(property, census);
    }
    walk_schema(&schema["items"], census);
    walk_schema(&schema["additionalProperties"], census);
}

/// The fragment a derived type name is built from, written here rather than read from the crate so
/// the naming rule is asserted against something other than itself.
fn pascal_case(member: &str) -> String {
    member
        .split(|character: char| !character.is_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut characters = segment.chars();
            let first: String = characters
                .next()
                .into_iter()
                .flat_map(char::to_uppercase)
                .collect();
            format!("{first}{}", characters.as_str())
        })
        .collect()
}
