//! Guards over the slice of the API that generated clients are built from.
//!
//! An operation joins that slice only when someone writes the `public` tag on
//! it, so a handler added later stays out of generated clients until somebody
//! decides otherwise. Generators split each `operation_id` into an entity and
//! one of its methods, and map error responses onto a typed error hierarchy, so
//! an opted-in operation has to spell its identifier as `entity.verb` and
//! answer every error with a `Problem`.
//!
//! Both invariants are checked against the OpenAPI document the application
//! actually serves, never against a list kept by hand here.

use serde_json::Value;

/// Tag that opts an operation into the surface exposed to generated clients.
const PUBLIC_TAG: &str = "public";

/// Schema every error response of an opted-in operation answers with.
const PROBLEM_SCHEMA_REF: &str = "#/components/schemas/Problem";

/// Media type generated clients read an error body as.
const PROBLEM_MEDIA_TYPE: &str = "application/json";

/// Keys of a path item that describe an operation. The rest of a path item
/// (`parameters`, `summary`, …) describes the path itself.
const HTTP_METHODS: [&str; 7] = ["get", "put", "post", "delete", "options", "head", "patch"];

/// Lowest status code that makes a response an error.
const FIRST_ERROR_STATUS: u16 = 400;

/// One operation of the served document.
struct Operation<'a> {
    path: &'a str,
    method: &'a str,
    body: &'a Value,
}

impl<'a> Operation<'a> {
    fn is_public(&self) -> bool {
        match self.body["tags"].as_array() {
            Some(tags) => tags.iter().any(|tag| tag == PUBLIC_TAG),
            None => false,
        }
    }

    fn operation_id(&self) -> Option<&'a str> {
        self.body["operationId"].as_str()
    }

    /// Every response the operation declares for a status of 400 or above.
    fn error_responses(&self) -> Vec<(&'a str, &'a Value)> {
        match self.body["responses"].as_object() {
            Some(responses) => responses
                .iter()
                .filter(|(status, _)| {
                    status
                        .parse::<u16>()
                        .is_ok_and(|status| status >= FIRST_ERROR_STATUS)
                })
                .map(|(status, response)| (status.as_str(), response))
                .collect(),
            None => Vec::new(),
        }
    }
}

impl std::fmt::Display for Operation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.method.to_uppercase(), self.path)
    }
}

/// Every operation the served document describes.
fn operations(spec: &Value) -> Vec<Operation<'_>> {
    let paths = match spec["paths"].as_object() {
        Some(paths) => paths,
        None => return Vec::new(),
    };

    paths
        .iter()
        .flat_map(|(path, item)| {
            HTTP_METHODS.iter().filter_map(move |method| {
                item.get(method).map(|body| Operation {
                    path: path.as_str(),
                    method,
                    body,
                })
            })
        })
        .collect()
}

/// `entity.verb`: one dot, an entity and a verb that both read as identifiers.
fn follows_entity_verb_convention(operation_id: &str) -> bool {
    match operation_id.split_once('.') {
        Some((entity, verb)) => is_identifier(entity) && is_identifier(verb),
        None => false,
    }
}

fn is_identifier(segment: &str) -> bool {
    let mut characters = segment.chars();
    match characters.next() {
        Some(first) if first.is_ascii_alphabetic() => {
            characters.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_support::openapi_spec;
    use serde_json::json;

    /// Generators split an `operation_id` on its dot to name an entity and one
    /// of its methods. An opted-in operation that spells its identifier any
    /// other way has no entity to hang off, and breaks generation for every
    /// language at once.
    #[actix_web::test]
    async fn opted_in_operations_spell_their_identifier_as_entity_and_verb() {
        let spec = openapi_spec().await;
        let all = operations(&spec);
        let opted_in = all
            .iter()
            .filter(|operation| operation.is_public())
            .collect::<Vec<_>>();

        assert!(
            !opted_in.is_empty(),
            "no served operation carries the `{PUBLIC_TAG}` tag"
        );

        let offenders = opted_in
            .iter()
            .filter(|operation| match operation.operation_id() {
                Some(operation_id) => !follows_entity_verb_convention(operation_id),
                None => true,
            })
            .map(|operation| format!("{operation} declares {:?}", operation.operation_id()))
            .collect::<Vec<_>>();

        assert!(
            offenders.is_empty(),
            "these `{PUBLIC_TAG}` operations do not spell their identifier as `entity.verb`: {}",
            offenders.join(", ")
        );
    }

    /// Generated clients turn an error body into a typed error. An opted-in
    /// operation whose error response carries no schema leaves them with a
    /// status code and nothing to read.
    #[actix_web::test]
    async fn opted_in_operations_answer_their_errors_with_a_problem() {
        let spec = openapi_spec().await;
        let all = operations(&spec);
        let opted_in = all
            .iter()
            .filter(|operation| operation.is_public())
            .collect::<Vec<_>>();

        assert!(
            !opted_in.is_empty(),
            "no served operation carries the `{PUBLIC_TAG}` tag"
        );

        let mut checked = 0_usize;
        for operation in &opted_in {
            let error_responses = operation.error_responses();
            assert!(
                !error_responses.is_empty(),
                "{operation} declares no error response"
            );

            for (status, response) in error_responses {
                assert_eq!(
                    response["content"][PROBLEM_MEDIA_TYPE]["schema"]["$ref"],
                    json!(PROBLEM_SCHEMA_REF),
                    "{operation} answers {status} with {response}"
                );
                checked += 1;
            }
        }

        assert!(
            spec["components"]["schemas"]["Problem"].is_object(),
            "the schema those {checked} error responses point at is missing from the components"
        );
    }

    /// The tag is opt-in: the control plane the dashboard drives — signing in,
    /// registering, inviting — stays out until somebody writes the tag on it.
    #[actix_web::test]
    async fn the_tag_selects_a_subset_of_the_served_operations() {
        let spec = openapi_spec().await;
        let all = operations(&spec);

        assert!(
            all.iter().any(|operation| operation.is_public()),
            "the served document exposes no operation at all"
        );
        assert!(
            all.iter().any(|operation| !operation.is_public()),
            "every served operation is opted in, so the tag no longer selects anything"
        );
    }

    #[test]
    fn an_identifier_needs_an_entity_and_a_verb() {
        assert!(follows_entity_verb_convention("applications.create"));
        assert!(follows_entity_verb_convention("applicationSecrets.list"));
        assert!(follows_entity_verb_convention(
            "events_per_day.list_for_application"
        ));

        assert!(!follows_entity_verb_convention("register"));
        assert!(!follows_entity_verb_convention("applications."));
        assert!(!follows_entity_verb_convention(".create"));
        assert!(!follows_entity_verb_convention(
            "api.v1.applications.create"
        ));
        assert!(!follows_entity_verb_convention("2applications.create"));
    }

    #[test]
    fn a_path_item_yields_one_operation_per_method_it_declares() {
        let spec = json!({
            "paths": {
                "/applications": {
                    "parameters": [{"name": "organization_id"}],
                    "get": {"operationId": "applications.list", "tags": ["public"]},
                    "post": {"operationId": "applications.create", "tags": ["mcp"]},
                }
            }
        });

        let found = operations(&spec);
        assert_eq!(found.len(), 2, "the `parameters` key is not an operation");
        assert_eq!(
            found
                .iter()
                .map(|operation| (operation.to_string(), operation.is_public()))
                .collect::<Vec<_>>(),
            vec![
                ("GET /applications".to_owned(), true),
                ("POST /applications".to_owned(), false),
            ]
        );
    }
}
