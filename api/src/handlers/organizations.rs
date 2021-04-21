use actix_web_middleware_keycloak_auth::UnstructuredClaims;
use paperclip::actix::{
    api_v2_operation,
    web::{Json, ReqData},
    Apiv2Schema,
};
use serde::Serialize;
use uuid::Uuid;

use crate::errors::*;
use crate::iam::extract_organizations;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Organization {
    pub organization_id: Uuid,
    pub role: String,
}

/// List organizations
#[api_v2_operation]
pub async fn list(
    unstructured_claims: ReqData<UnstructuredClaims>,
) -> Result<Json<Vec<Organization>>, UnexpectedError> {
    let organizations = extract_organizations(&unstructured_claims)
        .iter()
        .map(|(id, role)| Organization {
            organization_id: *id,
            role: role.to_string(),
        })
        .collect();
    Ok(Json(organizations))
}
