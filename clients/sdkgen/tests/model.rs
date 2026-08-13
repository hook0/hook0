//! Black-box suite over the committed snapshot and over documents written for a single behaviour.

use std::path::Path;

use hook0_sdkgen::{
    EntityModel, Error, HttpMethod, Limits, MCP_TAG, Nonconformity, PUBLIC_TAG, ParameterLocation,
    Snapshot, Verb,
};
use serde_json::json;

mod common;

fn model_of(bytes: &[u8], limits: &Limits) -> Result<(Snapshot, EntityModel), Error> {
    let snapshot = Snapshot::from_bytes(bytes, PUBLIC_TAG, limits)?;
    let model = EntityModel::from_snapshot(&snapshot, limits)?;
    Ok((snapshot, model))
}

fn built(bytes: &[u8]) -> (Snapshot, EntityModel) {
    let limits = Limits::default();
    let (snapshot, model) = model_of(bytes, &limits).expect("the document under test builds");
    common::assert_model_invariants(&snapshot, &model, &limits);
    (snapshot, model)
}

#[test]
fn every_public_operation_of_the_committed_snapshot_reaches_the_model() {
    let bytes = common::fixture_bytes();
    let public = common::declared_public_operations(&bytes);
    let (snapshot, model) = built(&bytes);

    assert!(
        !public.is_empty(),
        "the committed snapshot marks no operation as public"
    );
    assert_eq!(
        common::identities(snapshot.operations()),
        public
            .iter()
            .map(common::DeclaredOperation::identity)
            .collect(),
        "the snapshot does not carry exactly the operations the document marks public"
    );
    assert_eq!(
        model.method_count() + model.unconventional().len(),
        public.len()
    );
    assert!(
        !model.entities().is_empty(),
        "the committed snapshot yields no entity"
    );
}

#[test]
fn the_committed_snapshot_is_rebuilt_identically() {
    let bytes = common::fixture_bytes();
    let limits = Limits::default();

    assert_eq!(
        model_of(&bytes, &limits),
        model_of(&bytes, &limits),
        "two builds of the same snapshot differ"
    );
}

#[test]
fn a_snapshot_read_from_disk_matches_the_one_read_from_memory() {
    let limits = Limits::default();
    let from_disk = Snapshot::from_path(Path::new(common::FIXTURE_PATH), PUBLIC_TAG, &limits)
        .expect("the fixture parses");
    let from_memory = Snapshot::from_bytes(&common::fixture_bytes(), PUBLIC_TAG, &limits)
        .expect("the fixture parses");

    assert_eq!(from_disk, from_memory);
}

#[test]
fn canonical_verbs_are_recognised() {
    let (_, model) = built(&common::fixture_bytes());
    let applications = model
        .entity("applications")
        .expect("the snapshot declares an applications entity");

    for (verb_text, expected) in [
        ("list", Verb::List),
        ("get", Verb::Get),
        ("create", Verb::Create),
        ("update", Verb::Update),
        ("delete", Verb::Delete),
    ] {
        let method = applications
            .method(verb_text)
            .unwrap_or_else(|| panic!("applications carries no `{verb_text}` method"));
        assert_eq!(method.verb, expected);
        assert!(method.verb.is_canonical());
    }
}

#[test]
fn verbs_outside_the_vocabulary_stay_named_methods() {
    let (_, model) = built(&common::fixture_bytes());

    // Starts with a canonical verb without being one, so a generator matching on a prefix
    // rather than on the whole verb would wrongly render this as a plain list.
    let list_for_application = model
        .entity("events_per_day")
        .and_then(|entity| entity.method("list_for_application"))
        .expect("the snapshot declares events_per_day.list_for_application");
    assert_eq!(
        list_for_application.verb,
        Verb::Named("list_for_application".to_owned())
    );
    assert!(!list_for_application.verb.is_canonical());

    let ingest = model
        .entity("events")
        .and_then(|entity| entity.method("ingest"))
        .expect("the snapshot declares events.ingest");
    assert_eq!(ingest.verb, Verb::Named("ingest".to_owned()));
    assert_eq!(ingest.operation.method, HttpMethod::Post);
}

#[test]
fn alias_verbs_carry_the_meaning_of_the_verb_they_stand_for() {
    let (_, model) = built(&common::spec_with_paths(json!({
        "/things/{id}": {
            "get": {"operationId": "things.load"},
            "delete": {"operationId": "things.remove"},
        },
    })));

    let things = model.entity("things").expect("things is modelled");
    assert_eq!(
        things.method("load").expect("things.load is modelled").verb,
        Verb::Get
    );
    assert_eq!(
        things
            .method("remove")
            .expect("things.remove is modelled")
            .verb,
        Verb::Delete
    );
}

#[test]
fn an_operation_id_without_a_separator_is_reported_rather_than_dropped() {
    let (_, model) = built(&common::spec_with_paths(json!({
        "/register": {"post": {"operationId": "register"}},
    })));

    assert!(model.entities().is_empty());
    assert_eq!(model.unconventional().len(), 1);
    assert_eq!(
        model.unconventional()[0].reason,
        Nonconformity::MissingVerbSeparator
    );
    assert_eq!(
        model.unconventional()[0].operation.operation_id.as_deref(),
        Some("register")
    );
}

#[test]
fn an_operation_without_an_id_is_reported_rather_than_dropped() {
    let (_, model) = built(&common::spec_with_paths(json!({
        "/anonymous": {"get": {"summary": "no operation id here"}},
    })));

    assert_eq!(model.unconventional().len(), 1);
    assert_eq!(
        model.unconventional()[0].reason,
        Nonconformity::MissingOperationId
    );
}

#[test]
fn an_operation_id_missing_its_entity_or_its_verb_is_reported() {
    let (_, model) = built(&common::spec_with_paths(json!({
        "/headless": {"get": {"operationId": ".list"}},
        "/tailless": {"get": {"operationId": "things."}},
    })));

    let reasons: Vec<_> = model
        .unconventional()
        .iter()
        .map(|unconventional| unconventional.reason)
        .collect();

    assert!(reasons.contains(&Nonconformity::EmptyEntityName));
    assert!(reasons.contains(&Nonconformity::EmptyVerb));
    assert!(model.entities().is_empty());
}

#[test]
fn the_public_tag_narrows_the_snapshot_to_what_it_marks() {
    let (snapshot, model) = built(&common::spec_with_paths(json!({
        "/things": {
            "get": {"operationId": "things.list", "tags": ["public"]},
            "post": {"operationId": "things.create", "tags": ["mcp"]},
        },
        "/secrets": {"get": {"operationId": "secrets.list"}},
    })));

    assert!(snapshot.tag_applied());
    assert_eq!(snapshot.filtered_out(), 2);
    assert_eq!(snapshot.operations().len(), 1);
    assert_eq!(model.method_count(), 1);
    assert!(model.entity("secrets").is_none());
}

/// Each target names the tag it is built from, so the same document yields a different surface
/// depending on who reads it.
#[test]
fn a_snapshot_is_narrowed_to_whichever_tag_a_target_names() {
    let document = common::spec_with_paths(json!({
        "/things": {
            "get": {"operationId": "things.list", "tags": ["public"]},
            "post": {"operationId": "things.create", "tags": ["mcp"]},
        },
        "/secrets": {"get": {"operationId": "secrets.list", "tags": ["public", "mcp"]}},
    }));
    let limits = Limits::default();

    let selected = |tag: &str| {
        let snapshot =
            Snapshot::from_bytes(&document, tag, &limits).expect("the document under test parses");
        snapshot
            .operations()
            .iter()
            .filter_map(|operation| operation.operation_id.clone())
            .collect::<Vec<_>>()
    };

    assert_eq!(selected(PUBLIC_TAG), ["secrets.list", "things.list"]);
    assert_eq!(selected(MCP_TAG), ["secrets.list", "things.create"]);
    assert!(
        selected("absent").len() == 3,
        "an unused tag narrows nothing"
    );
}

#[test]
fn a_snapshot_without_the_public_tag_keeps_every_operation() {
    let (snapshot, _) = built(&common::spec_with_paths(json!({
        "/things": {"get": {"operationId": "things.list"}},
        "/secrets": {"get": {"operationId": "secrets.list"}},
    })));

    assert!(!snapshot.tag_applied());
    assert_eq!(snapshot.filtered_out(), 0);
    assert_eq!(snapshot.operations().len(), 2);
}

/// The committed snapshot carries the tag, so the generator sees the curated surface rather than
/// the whole API: the private control plane the dashboard drives stays out of generated clients.
#[test]
fn the_committed_snapshot_narrows_down_to_its_public_tag() {
    let bytes = common::fixture_bytes();
    let declared = common::declared_operations(&bytes);
    let public = common::declared_public_operations(&bytes);
    let (snapshot, _) = built(&bytes);

    assert!(
        snapshot.tag_applied(),
        "the committed snapshot marks nothing public, so nothing narrows the generated surface"
    );
    assert_eq!(
        snapshot.filtered_out(),
        declared.len() - public.len(),
        "the snapshot leaves out something other than the untagged operations"
    );
    assert!(
        snapshot.filtered_out() > 0,
        "the committed snapshot exposes its whole surface to generated clients"
    );
}

#[test]
fn an_operation_id_declared_twice_is_rejected() {
    let error = model_of(
        &common::spec_with_paths(json!({
            "/things": {"get": {"operationId": "things.list"}},
            "/other-things": {"get": {"operationId": "things.list"}},
        })),
        &Limits::default(),
    )
    .expect_err("a repeated operation id is rejected");

    assert!(
        matches!(error, Error::DuplicateOperationId { ref operation_id, .. } if operation_id == "things.list"),
        "unexpected error: {error}"
    );
}

#[test]
fn path_level_parameters_reach_every_operation_of_the_path() {
    let (snapshot, _) = built(&common::spec_with_paths(json!({
        "/things/{id}": {
            "parameters": [
                {"in": "path", "name": "id", "required": true, "schema": {"type": "string"}},
                {"in": "query", "name": "verbose", "required": false, "schema": {"type": "boolean"}},
            ],
            "get": {
                "operationId": "things.get",
                "parameters": [
                    {"in": "query", "name": "verbose", "required": true, "schema": {"type": "boolean"}},
                ],
            },
        },
    })));

    let parameters = &snapshot.operations()[0].parameters;
    assert_eq!(parameters.len(), 2, "parameters were lost or duplicated");

    let id = parameters
        .iter()
        .find(|parameter| parameter.name == "id")
        .expect("the path parameter reaches the operation");
    assert_eq!(id.location, ParameterLocation::Path);
    assert!(id.required);

    let verbose = parameters
        .iter()
        .find(|parameter| parameter.name == "verbose")
        .expect("the query parameter reaches the operation");
    assert!(
        verbose.required,
        "the operation-level parameter does not win over the path-level one"
    );
}

#[test]
fn a_referenced_parameter_is_resolved() {
    let (snapshot, _) = built(&common::spec_with(
        json!({
            "/things/{id}": {
                "get": {
                    "operationId": "things.get",
                    "parameters": [{"$ref": "#/components/parameters/first"}],
                },
            },
        }),
        json!({
            "parameters": {
                "first": {"$ref": "#/components/parameters/second"},
                "second": {"in": "path", "name": "id", "required": true, "schema": {"type": "string"}},
            },
        }),
    ));

    let parameters = &snapshot.operations()[0].parameters;
    assert_eq!(parameters.len(), 1);
    assert_eq!(parameters[0].name, "id");
    assert_eq!(parameters[0].location, ParameterLocation::Path);
}

#[test]
fn a_reference_chain_above_the_ceiling_is_rejected() {
    let bytes = common::spec_with(
        json!({
            "/things/{id}": {
                "get": {
                    "operationId": "things.get",
                    "parameters": [{"$ref": "#/components/parameters/first"}],
                },
            },
        }),
        json!({
            "parameters": {
                "first": {"$ref": "#/components/parameters/second"},
                "second": {"in": "path", "name": "id", "required": true, "schema": {"type": "string"}},
            },
        }),
    );

    let limits = Limits {
        max_reference_depth: 1,
        ..Limits::DEFAULT
    };
    let error = model_of(&bytes, &limits).expect_err("the chain crosses the ceiling");

    assert!(
        matches!(error, Error::ReferenceTooDeep { limit: 1, .. }),
        "unexpected error: {error}"
    );
}

#[test]
fn a_reference_cycle_is_stopped_by_the_ceiling() {
    let bytes = common::spec_with(
        json!({
            "/things/{id}": {
                "get": {
                    "operationId": "things.get",
                    "parameters": [{"$ref": "#/components/parameters/loop"}],
                },
            },
        }),
        json!({"parameters": {"loop": {"$ref": "#/components/parameters/loop"}}}),
    );

    let error = model_of(&bytes, &Limits::default()).expect_err("the cycle is stopped");

    assert!(
        matches!(error, Error::ReferenceTooDeep { .. }),
        "unexpected error: {error}"
    );
}

#[test]
fn a_reference_pointing_outside_the_snapshot_is_rejected() {
    let error = model_of(
        &common::spec_with_paths(json!({
            "/things/{id}": {
                "get": {
                    "operationId": "things.get",
                    "parameters": [{"$ref": "#/components/parameters/absent"}],
                },
            },
        })),
        &Limits::default(),
    )
    .expect_err("a dangling reference is rejected");

    assert!(
        matches!(error, Error::UnresolvableReference { .. }),
        "unexpected error: {error}"
    );
}

#[test]
fn a_referenced_path_item_is_rejected() {
    let error = model_of(
        &common::spec_with_paths(json!({
            "/things": {"$ref": "#/paths/~1other"},
        })),
        &Limits::default(),
    )
    .expect_err("a referenced path item is rejected");

    assert!(
        matches!(error, Error::UnresolvableReference { .. }),
        "unexpected error: {error}"
    );
}

#[test]
fn a_snapshot_above_the_byte_ceiling_is_rejected() {
    let bytes = common::fixture_bytes();
    let limits = Limits {
        max_snapshot_bytes: bytes.len() - 1,
        ..Limits::DEFAULT
    };

    assert!(
        matches!(
            Snapshot::from_bytes(&bytes, PUBLIC_TAG, &limits),
            Err(Error::SnapshotTooLarge { .. })
        ),
        "an oversized snapshot was accepted"
    );
    assert!(
        matches!(
            Snapshot::from_path(Path::new(common::FIXTURE_PATH), PUBLIC_TAG, &limits),
            Err(Error::SnapshotTooLarge { .. })
        ),
        "an oversized snapshot was accepted from disk"
    );
}

#[test]
fn more_operations_than_the_ceiling_accepts_is_rejected() {
    let bytes = common::spec_with_paths(json!({
        "/things": {
            "get": {"operationId": "things.list"},
            "post": {"operationId": "things.create"},
        },
    }));
    let limits = Limits {
        max_operations: 1,
        ..Limits::DEFAULT
    };

    assert_eq!(
        model_of(&bytes, &limits),
        Err(Error::TooManyOperations { count: 2, limit: 1 })
    );
}

#[test]
fn more_entities_than_the_ceiling_accepts_is_rejected() {
    let bytes = common::spec_with_paths(json!({
        "/things": {"get": {"operationId": "things.list"}},
        "/others": {"get": {"operationId": "others.list"}},
    }));
    let limits = Limits {
        max_entities: 1,
        ..Limits::DEFAULT
    };

    assert_eq!(
        model_of(&bytes, &limits),
        Err(Error::TooManyEntities { count: 2, limit: 1 })
    );
}

#[test]
fn more_methods_on_one_entity_than_the_ceiling_accepts_is_rejected() {
    let bytes = common::spec_with_paths(json!({
        "/things": {
            "get": {"operationId": "things.list"},
            "post": {"operationId": "things.create"},
        },
    }));
    let limits = Limits {
        max_methods_per_entity: 1,
        ..Limits::DEFAULT
    };

    let error = model_of(&bytes, &limits).expect_err("the entity crosses the ceiling");
    assert!(
        matches!(error, Error::TooManyMethods { ref entity, limit: 1, .. } if entity == "things"),
        "unexpected error: {error}"
    );
}

#[test]
fn more_parameters_than_the_ceiling_accepts_is_rejected() {
    let bytes = common::spec_with_paths(json!({
        "/things": {
            "get": {
                "operationId": "things.list",
                "parameters": [
                    {"in": "query", "name": "a", "schema": {"type": "string"}},
                    {"in": "query", "name": "b", "schema": {"type": "string"}},
                ],
            },
        },
    }));
    let limits = Limits {
        max_parameters_per_operation: 1,
        ..Limits::DEFAULT
    };

    let error = model_of(&bytes, &limits).expect_err("the operation crosses the ceiling");
    assert!(
        matches!(
            error,
            Error::TooManyParameters {
                count: 2,
                limit: 1,
                ..
            }
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn an_identifier_above_the_ceiling_is_rejected() {
    let bytes = common::spec_with_paths(json!({
        "/things": {"get": {"operationId": "things.list"}},
    }));
    let limits = Limits {
        max_identifier_bytes: 4,
        ..Limits::DEFAULT
    };

    let error = model_of(&bytes, &limits).expect_err("the operation id crosses the ceiling");
    assert!(
        matches!(error, Error::IdentifierTooLong { limit: 4, .. }),
        "unexpected error: {error}"
    );
}

#[test]
fn a_malformed_snapshot_is_rejected() {
    for bytes in [
        b"".as_slice(),
        b"not json".as_slice(),
        br#"{"openapi": "3.0.0""#.as_slice(),
        br#"{"openapi": "3.0.0", "info": {"title": "t"}}"#.as_slice(),
    ] {
        assert!(
            matches!(
                Snapshot::from_bytes(bytes, PUBLIC_TAG, &Limits::default()),
                Err(Error::MalformedSnapshot(_))
            ),
            "a malformed snapshot was accepted"
        );
    }
}

#[test]
fn a_snapshot_that_cannot_be_read_is_reported() {
    let error = Snapshot::from_path(
        Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/absent.json"
        )),
        PUBLIC_TAG,
        &Limits::default(),
    )
    .expect_err("an absent file is reported");

    assert!(
        matches!(error, Error::SnapshotUnreadable { .. }),
        "unexpected error: {error}"
    );
}
