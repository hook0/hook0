use actix_web_middleware_keycloak_auth::UnstructuredClaims;
use paperclip::actix::{
    api_v2_operation,
    web::{Json, ReqData},
    Apiv2Schema,
};
use serde::Serialize;
use uuid::Uuid;

use crate::iam::extract_organizations;
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Organization {
    pub organization_id: Uuid,
    pub role: String,
}

#[api_v2_operation(
    summary = "List organizations",
    description = "",
    operation_id = "organizations.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Identity and Access Management")
)]
pub async fn list(
    unstructured_claims: ReqData<UnstructuredClaims>,
) -> Result<Json<Vec<Organization>>, Hook0Problem> {
    let organizations = extract_organizations(&unstructured_claims)
        .iter()
        .map(|(id, role)| Organization {
            organization_id: *id,
            role: role.to_string(),
        })
        .collect();
    Ok(Json(organizations))
}
