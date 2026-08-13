use paperclip::actix::web::Json;
use paperclip::actix::{Apiv2Schema, api_v2_operation};
use serde::Serialize;
use serde_json::Value;
use strum::IntoEnumIterator;

use crate::problems::{Hook0Problem, Hook0ProblemId, ProblemDetails};

/// The body every Hook0 error answers with, in RFC 7807 problem format.
///
/// Its members are exactly the ones the API writes on the wire, so a client generated from the
/// published schema can read an error response whole rather than the part of it somebody
/// remembered to declare.
#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Problem {
    /// Documentation page of this problem, which is also what distinguishes one problem type from
    /// another as RFC 7807 asks. Prefer matching on `id`: it says the same thing without parsing
    /// a URL.
    #[serde(rename = "type")]
    type_url: String,
    id: Hook0ProblemId,
    /// Which submitted values were rejected and why, keyed by the name of the field they were
    /// submitted under. Carried by validation failures (`Validation`, status 422) and `null` for
    /// every other problem, as nothing else has a field to point at.
    ///
    /// The value of a key is either the list of failures on that field — each one a `code`, an
    /// optional `message` and the `params` the check was run with — or the same structure again
    /// for a nested object, or a map from index to that structure for a list. Nesting follows the
    /// shape of the submitted body and is therefore left free-form rather than modelled.
    validation: Option<Value>,
    title: String,
    detail: String,
    status: u16,
}

impl From<ProblemDetails> for Problem {
    fn from(internal_problem: ProblemDetails) -> Self {
        let id = internal_problem.id.id();
        Problem {
            type_url: id.type_url(),
            id,
            validation: internal_problem.validation,
            title: internal_problem.title.to_string(),
            detail: internal_problem.detail.to_string(),
            status: internal_problem.status.as_u16(),
        }
    }
}

/// List errors
#[api_v2_operation(
    summary = "List errors",
    description = "List of every possible errors that Hook0 can return. Each error is in RFC7807 problem format.",
    operation_id = "errors.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Hook0", "public")
)]
pub async fn list() -> Result<Json<Vec<Problem>>, Hook0Problem> {
    Ok(Json(
        Hook0Problem::iter()
            .map(|problem: Hook0Problem| Problem::from(ProblemDetails::from(problem)))
            .collect(),
    ))
}
