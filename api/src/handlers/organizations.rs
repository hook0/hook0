use log::error;
use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json, Path},
    Apiv2Schema,
};
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;
use validator::Validate;

use crate::iam::{AuthProof, Role, GROUP_SEP, ORGA_GROUP_PREFIX};
use crate::keycloak_api::{Group, KeycloakApi};
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
                    .and_then(|(_, str)| Role::from_string_with_prefix(str))
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

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct Grant {
    #[validate(non_control_character, email, length(max = 100))]
    email: String,
    role: String,
}

#[api_v2_operation(
    summary = "Invite a user to an organization",
    description = "",
    operation_id = "organizations.grant",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn grant(
    state: Data<crate::State>,
    auth: AuthProof,
    organization_id: Path<Uuid>,
    body: Json<Grant>,
) -> Result<Json<Grant>, Hook0Problem> {
    if auth
        .can_access_organization(&organization_id, &Role::Editor)
        .await
        .is_none()
    {
        return Err(Hook0Problem::Forbidden);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    match Role::from_str(&body.role) {
        Ok(role) => {
            let role_group_name = role.string_with_prefix();

            let keycloak_api = KeycloakApi::new(
                &state.keycloak_url,
                &state.keycloak_realm,
                &state.keycloak_client_id,
                &state.keycloak_client_secret,
            )
            .await?;

            let u = keycloak_api.get_user_by_email(&body.email).await?;
            let g = keycloak_api
                .lookup_group_by_name(&format!("{ORGA_GROUP_PREFIX}{organization_id}"))
                .await?;
            match (u, g) {
                (Some(user), Some(group)) => {
                    let root_group = keycloak_api.get_group(&group.id).await?;
                    let role_group = root_group
                        .sub_groups
                        .iter()
                        .find(|g| g.name == role_group_name)
                        .ok_or(Hook0Problem::NotFound)?;

                    remove_user_from_all_sub_groups(&keycloak_api, &user.id, &root_group).await?;
                    keycloak_api
                        .add_user_to_group(&user.id, &role_group.id)
                        .await?;
                    Ok(body)
                }
                _ => Err(Hook0Problem::NotFound),
            }
        }
        Err(_) => Err(Hook0Problem::InvalidRole),
    }
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct Revoke {
    user_id: Uuid,
}

#[api_v2_operation(
    summary = "Revoke a user's access to an organization",
    description = "",
    operation_id = "organizations.revoke",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn revoke(
    state: Data<crate::State>,
    auth: AuthProof,
    organization_id: Path<Uuid>,
    body: Json<Revoke>,
) -> Result<Json<Revoke>, Hook0Problem> {
    if auth
        .can_access_organization(&organization_id, &Role::Editor)
        .await
        .is_none()
    {
        return Err(Hook0Problem::Forbidden);
    }

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
            let root_group = keycloak_api.get_group(&group.id).await?;
            remove_user_from_all_sub_groups(&keycloak_api, &body.user_id, &root_group).await?;

            Ok(body)
        }
        None => Err(Hook0Problem::NotFound),
    }
}

async fn remove_user_from_all_sub_groups(
    keycloak_api: &KeycloakApi,
    user_id: &Uuid,
    root_group: &Group,
) -> Result<(), Hook0Problem> {
    let root_group_id = [&root_group.id].into_iter();
    let sub_groups_ids = root_group.sub_groups.iter().map(|g| &g.id);
    let groups_ids = root_group_id.chain(sub_groups_ids);

    for group_id in groups_ids {
        let members = keycloak_api.get_group_members(group_id).await?;
        for member in members.iter() {
            if &member.id == user_id {
                keycloak_api
                    .remove_user_from_group(user_id, group_id)
                    .await?;
            }
        }
    }

    Ok(())
}
