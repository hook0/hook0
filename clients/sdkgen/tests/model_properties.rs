//! Invariants the entity model holds whatever document it was built from.

use std::collections::{BTreeSet, HashSet};

use hook0_sdkgen::{EntityModel, Error, HttpMethod, Limits, PUBLIC_TAG, Snapshot};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Map, Value, json};

mod common;

/// Seeds of past failures, replayed before anything random is drawn.
const REGRESSIONS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/proptest-regressions/model_properties.txt"
);

/// Distinct paths a generated document spreads its operations over.
const PATH_SLOTS: usize = 6;

/// Largest number of operations a generated document declares.
const OPERATION_SLOTS: usize = 24;

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSIONS))),
        ..ProptestConfig::default()
    }
}

#[derive(Debug, Clone)]
struct GeneratedOperation {
    operation_id: Option<String>,
    method: usize,
    path: usize,
    public: bool,
}

/// Operation ids spanning what the convention accepts and what it does not.
fn operation_id() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just(".".to_owned()),
        "[a-z]{1,8}",
        "[a-z]{1,8}\\.[a-z]{1,8}",
        "[a-z]{1,6}\\.(list|get|load|create|update|delete|remove)",
        "\\.[a-z]{1,6}",
        "[a-z]{1,6}\\.",
        "[a-z]{1,4}\\.[a-z]{1,4}\\.[a-z]{1,4}",
        "[\\p{L}\\p{N}_. -]{0,12}",
    ]
}

fn generated_operation() -> impl Strategy<Value = GeneratedOperation> {
    (
        prop::option::of(operation_id()),
        0..common::HTTP_METHODS.len(),
        0..PATH_SLOTS,
        any::<bool>(),
    )
        .prop_map(|(operation_id, method, path, public)| GeneratedOperation {
            operation_id,
            method,
            path,
            public,
        })
}

fn generated_operations() -> impl Strategy<Value = Vec<GeneratedOperation>> {
    prop::collection::vec(generated_operation(), 0..OPERATION_SLOTS)
}

/// Ceilings tight enough that a generated document regularly crosses them.
fn tight_limits() -> impl Strategy<Value = Limits> {
    (0usize..8, 0usize..8, 0usize..OPERATION_SLOTS).prop_map(|(entities, methods, operations)| {
        Limits {
            max_entities: entities,
            max_methods_per_entity: methods,
            max_operations: operations,
            ..Limits::DEFAULT
        }
    })
}

/// Lays the generated operations out as an OpenAPI document; later ones take over their slot.
fn document(operations: &[GeneratedOperation]) -> Vec<u8> {
    let mut paths = Map::new();

    for operation in operations {
        let mut body = Map::new();
        if let Some(operation_id) = operation.operation_id.as_ref() {
            body.insert("operationId".to_owned(), json!(operation_id));
        }
        if operation.public {
            body.insert("tags".to_owned(), json!([PUBLIC_TAG]));
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

    common::spec_with_paths(Value::Object(paths))
}

fn build(bytes: &[u8], limits: &Limits) -> Result<(Snapshot, EntityModel), Error> {
    let snapshot = Snapshot::from_bytes(bytes, limits)?;
    let model = EntityModel::from_snapshot(&snapshot, limits)?;
    Ok((snapshot, model))
}

/// The operations a document exposes once the public tag has had its say.
fn expected_surface(declared: &[common::DeclaredOperation]) -> BTreeSet<(HttpMethod, String)> {
    let public_tag_applied = declared
        .iter()
        .any(|operation| operation.tags.iter().any(|tag| tag == PUBLIC_TAG));

    declared
        .iter()
        .filter(|operation| {
            !public_tag_applied || operation.tags.iter().any(|tag| tag == PUBLIC_TAG)
        })
        .map(common::DeclaredOperation::identity)
        .collect()
}

fn carries_a_repeated_id(declared: &[common::DeclaredOperation]) -> bool {
    let mut seen = HashSet::new();
    declared
        .iter()
        .filter_map(|operation| operation.operation_id.as_ref())
        .any(|operation_id| !seen.insert(operation_id))
}

proptest! {
    #![proptest_config(config())]

    /// Building twice from the same bytes gives the same thing, down to the errors.
    #[test]
    fn a_model_is_rebuilt_identically(operations in generated_operations()) {
        let bytes = document(&operations);
        let limits = Limits::default();

        prop_assert_eq!(build(&bytes, &limits), build(&bytes, &limits));
    }

    /// The model carries the operations the document declares, no more and no fewer.
    #[test]
    fn every_declared_operation_lands_in_the_model_exactly_once(
        operations in generated_operations()
    ) {
        let bytes = document(&operations);
        let declared = common::declared_operations(&bytes);
        prop_assume!(!carries_a_repeated_id(&declared));

        let limits = Limits::default();
        let (snapshot, model) = build(&bytes, &limits)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(
            common::identities(snapshot.operations()),
            expected_surface(&declared)
        );
        common::assert_model_invariants(&snapshot, &model, &limits);
    }

    /// Entity and method names are never empty, and never collide inside their scope.
    #[test]
    fn a_model_holds_its_invariants_under_any_ceiling(
        operations in generated_operations(),
        limits in tight_limits(),
    ) {
        let bytes = document(&operations);

        if let Ok((snapshot, model)) = build(&bytes, &limits) {
            common::assert_model_invariants(&snapshot, &model, &limits);
        }
    }

    /// An operation id declared twice breaks the model, so it is refused outright.
    #[test]
    fn a_repeated_operation_id_is_rejected(
        operations in generated_operations(),
        repeated in "[a-z]{1,6}\\.[a-z]{1,6}",
    ) {
        let mut document: Value = serde_json::from_slice(&document(&operations))
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        let paths = document
            .get_mut("paths")
            .and_then(Value::as_object_mut)
            .ok_or_else(|| TestCaseError::fail("the generated document carries paths"))?;
        for slot in ["/repeated-first", "/repeated-second"] {
            paths.insert(
                slot.to_owned(),
                json!({"get": {"operationId": repeated, "responses": {}}}),
            );
        }

        let error = build(&common::document(document), &Limits::default())
            .expect_err("a repeated operation id is rejected");
        let rejected_as_repeated = matches!(error, Error::DuplicateOperationId { .. });

        prop_assert!(rejected_as_repeated, "unexpected error: {}", error);
    }

    /// A document above the operation ceiling is rejected, and one below it never is.
    #[test]
    fn the_operation_ceiling_is_enforced(
        operations in generated_operations(),
        ceiling in 0usize..OPERATION_SLOTS,
    ) {
        let bytes = document(&operations);
        let declared = common::declared_operations(&bytes);
        let limits = Limits { max_operations: ceiling, ..Limits::DEFAULT };
        let result = Snapshot::from_bytes(&bytes, &limits);

        if declared.len() > ceiling {
            prop_assert_eq!(
                result,
                Err(Error::TooManyOperations { count: declared.len(), limit: ceiling })
            );
        } else {
            let refused_for_count = matches!(result, Err(Error::TooManyOperations { .. }));
            prop_assert!(!refused_for_count, "a document below the ceiling was refused");
        }
    }
}
