//! Shared helpers for the black-box suites: fixture access, snapshot building and the invariants
//! every model is expected to hold.
#![allow(dead_code)]

use std::collections::BTreeSet;

use hook0_sdkgen::{
    EntityModel, HttpMethod, Limits, Nonconformity, Operation, PUBLIC_TAG, Snapshot,
};
use serde_json::{Value, json};

/// The snapshot the Hook0 API serves, committed so the suites never reach the network.
///
/// The generator reads the very file the API crate regenerates and guards, rather than a copy of
/// it: there is one document in the repository, so there is nothing for the two to drift apart on.
pub const FIXTURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../api/openapi.snapshot.json"
);

pub fn fixture_bytes() -> Vec<u8> {
    std::fs::read(FIXTURE_PATH).expect("the committed snapshot fixture is readable")
}

/// The operations the document marks as the surface generated clients are built from.
pub fn declared_public_operations(bytes: &[u8]) -> Vec<DeclaredOperation> {
    let mut declared = declared_operations(bytes);
    declared.retain(|operation| operation.tags.iter().any(|tag| tag == PUBLIC_TAG));
    declared
}

/// Wraps a `paths` object into the smallest document an OpenAPI parser accepts.
///
/// Every operation gets the `responses` object OpenAPI requires, so a document under test only
/// spells out what the behaviour it exercises needs.
pub fn spec_with_paths(paths: Value) -> Vec<u8> {
    spec_with(paths, json!({}))
}

/// Same, for a document that also needs components to point its references at.
pub fn spec_with(mut paths: Value, components: Value) -> Vec<u8> {
    fill_responses(&mut paths);

    document(json!({
        "openapi": "3.0.0",
        "info": {"title": "spec under test", "version": "0.0.0"},
        "paths": paths,
        "components": components,
    }))
}

fn fill_responses(paths: &mut Value) {
    let Some(paths) = paths.as_object_mut() else {
        return;
    };

    for item in paths.values_mut() {
        let Some(item) = item.as_object_mut() else {
            continue;
        };
        for method in HTTP_METHODS {
            let Some(operation) = item.get_mut(method).and_then(Value::as_object_mut) else {
                continue;
            };
            operation.entry("responses").or_insert_with(|| json!({}));
        }
    }
}

pub fn document(value: Value) -> Vec<u8> {
    value.to_string().into_bytes()
}

/// What identifies an operation inside a document: no two operations share a method and a path.
pub fn identity(operation: &Operation) -> (HttpMethod, String) {
    (operation.method, operation.path.clone())
}

pub fn identities(operations: &[Operation]) -> BTreeSet<(HttpMethod, String)> {
    operations.iter().map(identity).collect()
}

/// The methods an OpenAPI path item may hold an operation under.
pub const HTTP_METHODS: [&str; 8] = [
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

/// An operation as the raw document declares it, before the crate reads anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredOperation {
    pub method: String,
    pub path: String,
    pub operation_id: Option<String>,
    pub tags: Vec<String>,
}

impl DeclaredOperation {
    pub fn identity(&self) -> (HttpMethod, String) {
        let method = match self.method.as_str() {
            "PUT" => HttpMethod::Put,
            "POST" => HttpMethod::Post,
            "DELETE" => HttpMethod::Delete,
            "OPTIONS" => HttpMethod::Options,
            "HEAD" => HttpMethod::Head,
            "PATCH" => HttpMethod::Patch,
            "TRACE" => HttpMethod::Trace,
            _ => HttpMethod::Get,
        };
        (method, self.path.clone())
    }
}

/// The operations a raw document declares, read straight from JSON rather than from the crate.
pub fn declared_operations(bytes: &[u8]) -> Vec<DeclaredOperation> {
    let document: Value = serde_json::from_slice(bytes).expect("the document under test is JSON");
    let mut declared = Vec::new();

    let Some(paths) = document.get("paths").and_then(Value::as_object) else {
        return declared;
    };

    for (path, item) in paths {
        let Some(item) = item.as_object().filter(|_| path.starts_with('/')) else {
            continue;
        };
        for method in HTTP_METHODS {
            let Some(operation) = item.get(method).and_then(Value::as_object) else {
                continue;
            };
            declared.push(DeclaredOperation {
                method: method.to_uppercase(),
                path: path.clone(),
                operation_id: operation
                    .get("operationId")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                tags: operation
                    .get("tags")
                    .and_then(Value::as_array)
                    .map(|tags| {
                        tags.iter()
                            .filter_map(Value::as_str)
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default(),
            });
        }
    }

    declared
}

/// The names an emitted tool table gives its tools, read back from the source it wrote.
pub fn tool_names(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("name: \""))
        .filter_map(|line| line.strip_suffix("\","))
        .map(str::to_owned)
        .collect()
}

/// Everything a model holds true, whatever snapshot it came from.
pub fn assert_model_invariants(snapshot: &Snapshot, model: &EntityModel, limits: &Limits) {
    assert!(
        model.entities().len() <= limits.max_entities,
        "the model carries {} entities, above the {} accepted",
        model.entities().len(),
        limits.max_entities
    );

    let mut entity_names = BTreeSet::new();
    for entity in model.entities() {
        assert!(!entity.name.is_empty(), "an entity carries no name");
        assert!(
            entity_names.insert(entity.name.as_str()),
            "entity `{}` appears twice",
            entity.name
        );
        assert!(
            entity.methods.len() <= limits.max_methods_per_entity,
            "entity `{}` carries {} methods, above the {} accepted",
            entity.name,
            entity.methods.len(),
            limits.max_methods_per_entity
        );

        let mut verbs = BTreeSet::new();
        for method in &entity.methods {
            assert!(
                !method.verb_text.is_empty(),
                "entity `{}` carries a method with no verb",
                entity.name
            );
            assert!(
                verbs.insert(method.verb_text.as_str()),
                "entity `{}` carries `{}` twice",
                entity.name,
                method.verb_text
            );
            assert_eq!(
                method.operation_id,
                format!("{}.{}", entity.name, method.verb_text),
                "the method does not name the operation it came from"
            );
        }
    }

    for set_aside in model.unconventional() {
        let expected = match set_aside.operation.operation_id.as_deref() {
            None => Nonconformity::MissingOperationId,
            Some(operation_id) => match operation_id.split_once('.') {
                None => Nonconformity::MissingVerbSeparator,
                Some(("", _)) => Nonconformity::EmptyEntityName,
                Some((_, "")) => Nonconformity::EmptyVerb,
                Some(_) => panic!("`{operation_id}` follows the convention yet was set aside"),
            },
        };
        assert_eq!(
            set_aside.reason, expected,
            "an operation was set aside for the wrong reason"
        );
    }

    let mut modelled: Vec<(HttpMethod, String)> = model
        .entities()
        .iter()
        .flat_map(|entity| {
            entity
                .methods
                .iter()
                .map(|method| identity(&method.operation))
        })
        .chain(
            model
                .unconventional()
                .iter()
                .map(|unconventional| identity(&unconventional.operation)),
        )
        .collect();

    assert_eq!(
        modelled.len(),
        snapshot.operations().len(),
        "the model and the snapshot do not carry the same number of operations"
    );
    assert_eq!(
        model.method_count() + model.unconventional().len(),
        snapshot.operations().len(),
        "some operation of the snapshot reached neither an entity nor the unconventional list"
    );

    let counted = modelled.len();
    modelled.sort();
    modelled.dedup();
    assert_eq!(
        modelled.len(),
        counted,
        "an operation of the snapshot appears more than once in the model"
    );

    assert_eq!(
        modelled.into_iter().collect::<BTreeSet<_>>(),
        identities(snapshot.operations()),
        "the model does not carry exactly the operations of the snapshot"
    );
}
