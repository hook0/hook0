use paperclip::actix::web::Json;
use paperclip::actix::{api_v2_operation, Apiv2Schema};
use serde::Serialize;
use strum::IntoEnumIterator;

use crate::problems::{Hook0Problem, Problem as InternalProblem};

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Problem {
    id: String,
    title: String,
    detail: String,
    status: u16,
}

impl From<InternalProblem> for Problem {
    fn from(internal_problem: InternalProblem) -> Self {
        Problem {
            id: internal_problem.id.to_string(),
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
    tags("Hook0")
)]
pub async fn list() -> Result<Json<Vec<Problem>>, Hook0Problem> {
    Ok(Json(
        Hook0Problem::iter()
            .map(|problem: Hook0Problem| Problem::from(InternalProblem::from(problem)))
            .collect(),
    ))
}
