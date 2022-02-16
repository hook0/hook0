use log::error;
use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json, Path},
    Apiv2Schema,
};
use serde::Serialize;
use sqlx::query_as;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

use crate::iam::{AuthProof, Role, GROUP_SEP, ORGA_GROUP_PREFIX};
use crate::keycloak_api::KeycloakApi;
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Organization {
    pub organization_id: Uuid,
    pub role: String,
    pub name: String,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct OrganizationInfo {
    pub organization_id: Uuid,
    pub name: String,
    pub users: Vec<OrganizationUser>,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct OrganizationUser {
    pub user_id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
}

#[api_v2_operation(
    summary = "List organizations",
    description = "",
    operation_id = "organizations.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn list(
    state: Data<crate::State>,
    auth: AuthProof,
) -> Result<Json<Vec<Organization>>, Hook0Problem> {
    struct OrganizationMetadata {
        name: String,
    }
    let mut organizations = vec![];

    for (organization_id, role) in auth.organizations() {
        let metadata = query_as!(
            OrganizationMetadata,
            "
                SELECT name
                FROM event.organization
                WHERE organization__id = $1
            ",
            &organization_id
        )
        .fetch_optional(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

        let name = metadata.map(|om| om.name).unwrap_or_else(|| {
            error!(
                "Could not find organization {} in database",
                &organization_id
            );
            organization_id.to_string()
        });

        let org = Organization {
            organization_id,
            role: role.to_string(),
            name,
        };

        organizations.push(org);
    }

    Ok(Json(organizations))
}

#[api_v2_operation(
    summary = "Get organization's info by its ID",
    description = "",
    operation_id = "organizations.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn get(
    state: Data<crate::State>,
    auth: AuthProof,
    organization_id: Path<Uuid>,
) -> Result<Json<OrganizationInfo>, Hook0Problem> {
    if auth
        .can_access_organization(&organization_id, &Role::Viewer)
        .await
        .is_none()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let organization_id = organization_id.into_inner();
    struct OrganizationMetadata {
        name: String,
    }
    let metadata = query_as!(
        OrganizationMetadata,
        "
            SELECT name
            FROM event.organization
            WHERE organization__id = $1
        ",
        &organization_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    let name = metadata.map(|om| om.name).unwrap_or_else(|| {
        error!(
            "Could not find organization {} in database",
            &organization_id
        );
        organization_id.to_string()
    });

    let keycloak_api = KeycloakApi::new(
        &state.keycloak_url,
        &state.keycloak_realm,
        &state.keycloak_client_id,
        &state.keycloak_client_secret,
    )
    .await?;

    match keycloak_api
        .lookup_group_by_name(&format!("{ORGA_GROUP_PREFIX}{organization_id}"))
        .await?
    {
        Some(group) => {
            #[derive(Debug, Clone)]
            struct RoleGroup<'a> {
                pub id: &'a Uuid,
                pub role: Role,
            }
            let root_group = keycloak_api.get_group(&group.id).await?;
            let root_group_with_role = [RoleGroup {
                id: &group.id,
                role: Role::Viewer,
            }]
            .into_iter();
            let sub_groups_with_role = root_group.sub_groups.iter().map(|g| {
                let role = g
                    .path
                    .rsplit_once(GROUP_SEP)
                    .and_then(|(_, str)| Role::from_str(str).ok())
                    .unwrap_or(Role::Viewer);
                RoleGroup { id: &g.id, role }
            });
            let groups = root_group_with_role.chain(sub_groups_with_role);

            #[derive(Debug, Clone)]
            struct UserWithRole {
                pub id: Uuid,
                pub email: String,
                pub first_name: String,
                pub last_name: String,
                pub role: Role,
            }
            impl From<UserWithRole> for OrganizationUser {
                fn from(u: UserWithRole) -> Self {
                    Self {
                        user_id: u.id,
                        email: u.email,
                        first_name: u.first_name,
                        last_name: u.last_name,
                        role: u.role.to_string(),
                    }
                }
            }

            let mut users: HashMap<Uuid, UserWithRole> = HashMap::new();
            for group in groups {
                let role = group.role;
                let members = keycloak_api.get_group_members(group.id).await?;
                for member in members.iter().filter(|m| m.enabled) {
                    if users.get(&member.id).map(|u| role > u.role).unwrap_or(true) {
                        let user = UserWithRole {
                            id: member.id,
                            email: member.email.to_owned(),
                            first_name: member.first_name.to_owned(),
                            last_name: member.last_name.to_owned(),
                            role,
                        };
                        users.insert(user.id, user);
                    }
                }
            }
            let org_users = users.into_values().map(|u| u.into()).collect::<Vec<_>>();

            Ok(Json(OrganizationInfo {
                organization_id,
                name,
                users: org_users,
            }))
        }
        None => Err(Hook0Problem::NotFound),
    }
}
