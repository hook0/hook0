use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::Utc;
use log::error;
use paperclip::actix::web::{Data, Json, Path};
use paperclip::actix::{Apiv2Schema, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, query_scalar};
use std::str::FromStr;
use uuid::Uuid;
use validator::Validate;

use crate::hook0_client::{
    EventOrganizationCreated, EventOrganizationInvited, EventOrganizationRemoved,
    EventOrganizationRevoked, EventOrganizationUpdated, Hook0ClientEvent,
};
use crate::iam::{
    Action, AuthorizeServiceToken, AuthorizedToken, AuthorizedUserToken, Role, authorize,
    authorize_only_user,
};
use crate::onboarding::{
    OnboardingStepStatus, OrganizationOnboardingSteps, get_organization_onboarding_steps,
};
use crate::openapi::{OaBiscuit, OaBiscuitUserAccess};
use crate::problems::Hook0Problem;
use crate::quotas::{Quota, QuotaValue};

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Organization {
    pub organization_id: Uuid,
    pub role: String,
    pub name: String,
    pub plan: Option<OrganizationInfoPlan>,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct OrganizationInfo {
    pub organization_id: Uuid,
    pub name: String,
    pub plan: Option<OrganizationInfoPlan>,
    pub users: Vec<OrganizationUser>,
    pub quotas: OrganizationQuotas,
    pub consumption: OrganizationConsumption,
    pub onboarding_steps: OrganizationOnboardingSteps,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct OrganizationConsumption {
    members: Option<i64>,
    applications: Option<i64>,
    events_per_day: Option<i64>,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct OrganizationInfoPlan {
    pub name: String,
    pub label: String,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct OrganizationQuotas {
    pub members_per_organization_limit: QuotaValue,
    pub applications_per_organization_limit: QuotaValue,
    pub events_per_day_limit: QuotaValue,
    pub days_of_events_retention_limit: QuotaValue,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct OrganizationUser {
    pub user_id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: Role,
}

#[api_v2_operation(
    summary = "List organizations",
    description = "Retrieves all organizations the current API token has access to, with the associated roles.",
    operation_id = "organizations.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
) -> Result<Json<Vec<Organization>>, Hook0Problem> {
    if let Ok(token) = authorize(
        &biscuit,
        None,
        Action::OrganizationList,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    ) {
        let (token_organizations, is_master) = match token {
            AuthorizedToken::User(AuthorizedUserToken { organizations, .. }) => {
                (organizations, false)
            }
            AuthorizedToken::Service(AuthorizeServiceToken { organization_id }) => {
                (vec![(organization_id, Role::Editor)], false)
            }
            AuthorizedToken::Master => (vec![], true),
        };

        struct OrganizationMetadata {
            organization_id: Uuid,
            name: String,
            plan_name: Option<String>,
            plan_label: Option<String>,
        }
        let metadatas = query_as!(
            OrganizationMetadata,
            r#"
                SELECT o.organization__id AS organization_id, o.name, p.name AS "plan_name?", p.label AS "plan_label?"
                FROM iam.organization AS o
                LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
                LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
                WHERE organization__id = ANY($1) OR $2
            "#,
            &token_organizations.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
            is_master,
        )
        .fetch_all(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

        let organizations = metadatas
            .into_iter()
            .map(|metadata| {
                let role = if is_master {
                    "master".to_owned()
                } else {
                    token_organizations
                        .iter()
                        .find(|(i, _)| i == &metadata.organization_id)
                        .map(|(_, r)| r.to_string())
                        .unwrap_or_else(|| "???".to_owned())
                };

                let plan = match (metadata.plan_name, metadata.plan_label) {
                    (Some(name), Some(label)) => Some(OrganizationInfoPlan { name, label }),
                    _ => None,
                };

                Organization {
                    organization_id: metadata.organization_id,
                    role,
                    name: metadata.name,
                    plan,
                }
            })
            .collect::<Vec<_>>();

        Ok(Json(organizations))
    } else {
        Err(Hook0Problem::Forbidden)
    }
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct OrganizationPost {
    #[validate(non_control_character, length(min = 2, max = 50))]
    name: String,
}

#[api_v2_operation(
    summary = "Create an organization",
    description = "Create a new organization. An organization contains applications, members and optionaly a plan. Note that you will need to regenerate an authentication token to be able to see/use the newly created organization.",
    operation_id = "organizations.create",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn create(
    state: Data<crate::State>,
    _: OaBiscuitUserAccess,
    biscuit: ReqData<Biscuit>,
    body: Json<OrganizationPost>,
) -> Result<Json<OrganizationInfo>, Hook0Problem> {
    if let Ok(token) = authorize_only_user(
        &biscuit,
        None,
        Action::OrganizationCreate,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    ) {
        if let Err(e) = body.validate() {
            return Err(Hook0Problem::Validation(e));
        }

        let mut tx = state.db.begin().await?;

        let organization_id = Uuid::new_v4();
        query!(
            "
                INSERT INTO iam.organization (organization__id, name, created_by)
                VALUES ($1, $2, $3)
            ",
            &organization_id,
            &body.name,
            &token.user_id,
        )
        .execute(&mut *tx)
        .await?;

        query!(
            "
                INSERT INTO iam.user__organization (user__id, organization__id, role)
                VALUES ($1, $2, $3)
            ",
            &token.user_id,
            &organization_id,
            Role::Editor.as_ref(),
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        if let Some(hook0_client) = state.hook0_client.as_ref() {
            let hook0_client_event: Hook0ClientEvent = EventOrganizationCreated {
                organization_id,
                name: body.name.to_owned(),
                created_at: Utc::now(),
                created_by: token.user_id,
            }
            .into();
            if let Err(e) = hook0_client
                .send_event(&hook0_client_event.mk_hook0_event())
                .await
            {
                error!("Hook0ClientError: {e}");
            };
        }

        let quotas = OrganizationQuotas {
            members_per_organization_limit: state
                .quotas
                .get_limit_for_organization(
                    &state.db,
                    Quota::MembersPerOrganization,
                    &organization_id,
                )
                .await?,
            applications_per_organization_limit: state
                .quotas
                .get_limit_for_organization(
                    &state.db,
                    Quota::ApplicationsPerOrganization,
                    &organization_id,
                )
                .await?,
            events_per_day_limit: state
                .quotas
                .get_limit_for_organization(&state.db, Quota::EventsPerDay, &organization_id)
                .await?,
            days_of_events_retention_limit: state
                .quotas
                .get_limit_for_organization(
                    &state.db,
                    Quota::DaysOfEventsRetention,
                    &organization_id,
                )
                .await?,
        };

        Ok(Json(OrganizationInfo {
            organization_id,
            name: body.name.to_owned(),
            plan: None,
            users: vec![OrganizationUser {
                user_id: token.user_id,
                email: token.email,
                first_name: token.first_name,
                last_name: token.last_name,
                role: Role::Editor,
            }],
            quotas,
            consumption: OrganizationConsumption {
                members: Some(1),
                applications: Some(0),
                events_per_day: Some(0),
            },
            onboarding_steps: OrganizationOnboardingSteps {
                application: OnboardingStepStatus::ToDo,
                event_type: OnboardingStepStatus::ToDo,
                subscription: OnboardingStepStatus::ToDo,
                event: OnboardingStepStatus::ToDo,
            },
        }))
    } else {
        Err(Hook0Problem::Forbidden)
    }
}

#[api_v2_operation(
    summary = "Get organization's info by its ID",
    description = "Retrieves details about a specific organization, including quotas and consumption statistics.",
    operation_id = "organizations.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn get(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    organization_id: Path<Uuid>,
) -> Result<Json<OrganizationInfo>, Hook0Problem> {
    let organization_id = organization_id.into_inner();

    if authorize(
        &biscuit,
        Some(organization_id),
        Action::OrganizationGet,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    struct OrganizationMetadata {
        name: String,
        plan_name: Option<String>,
        plan_label: Option<String>,
    }
    let metadata = query_as!(
        OrganizationMetadata,
        r#"
            SELECT o.name, p.name AS "plan_name?", p.label AS "plan_label?"
            FROM iam.organization AS o
            LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
            LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
            WHERE organization__id = $1
        "#,
        &organization_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    if let Some(OrganizationMetadata {
        name,
        plan_name,
        plan_label,
    }) = metadata
    {
        let plan = match (plan_name, plan_label) {
            (Some(name), Some(label)) => Some(OrganizationInfoPlan { name, label }),
            _ => None,
        };

        #[derive(Debug, Clone)]
        struct UserWithRole {
            pub user_id: Uuid,
            pub email: String,
            pub first_name: String,
            pub last_name: String,
            pub role: String,
        }
        let users = query_as!(
            UserWithRole,
            r#"
                SELECT u.user__id AS user_id, u.email, u.first_name, u.last_name, uo.role
                FROM iam.user AS u
                INNER JOIN iam.user__organization AS uo ON uo.user__id = u.user__id
                WHERE uo.organization__id = $1
            "#,
            &organization_id
        )
        .fetch_all(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

        let org_users = users
            .into_iter()
            .flat_map(|u| {
                if let Ok(role) = Role::from_str(&u.role) {
                    vec![OrganizationUser {
                        user_id: u.user_id,
                        email: u.email,
                        first_name: u.first_name,
                        last_name: u.last_name,
                        role,
                    }]
                } else {
                    vec![]
                }
            })
            .collect::<Vec<_>>();

        let quotas = OrganizationQuotas {
            members_per_organization_limit: state
                .quotas
                .get_limit_for_organization(
                    &state.db,
                    Quota::MembersPerOrganization,
                    &organization_id,
                )
                .await?,
            applications_per_organization_limit: state
                .quotas
                .get_limit_for_organization(
                    &state.db,
                    Quota::ApplicationsPerOrganization,
                    &organization_id,
                )
                .await?,
            events_per_day_limit: state
                .quotas
                .get_limit_for_organization(&state.db, Quota::EventsPerDay, &organization_id)
                .await?,
            days_of_events_retention_limit: state
                .quotas
                .get_limit_for_organization(
                    &state.db,
                    Quota::DaysOfEventsRetention,
                    &organization_id,
                )
                .await?,
        };

        let consumption = query_as!(
            OrganizationConsumption,
            r#"
                WITH members AS (
                    SELECT COUNT(user__id) AS total
                    FROM iam.user__organization
                    WHERE organization__id = $1
                ), applications AS (
                    SELECT COUNT(application__id) AS total
                    FROM event.application
                    WHERE organization__id = $1
                        AND deleted_at IS NULL
                ), events_per_day AS (
                    SELECT SUM(e.amount) AS total
                    FROM event.events_per_day AS e
                    INNER JOIN event.application AS a ON a.application__id = e.application__id
                    WHERE a.organization__id = $1
                        AND e.date = CURRENT_DATE
                )
                SELECT
                    COALESCE(members.total, 0) AS members,
                    COALESCE(applications.total, 0) AS applications,
                    COALESCE(events_per_day.total, 0) AS events_per_day
                FROM
                    members, applications, events_per_day
            "#,
            &organization_id
        )
        .fetch_one(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

        let onboarding_steps =
            get_organization_onboarding_steps(&state.db, &organization_id).await?;

        Ok(Json(OrganizationInfo {
            organization_id,
            name,
            plan,
            users: org_users,
            quotas,
            consumption,
            onboarding_steps,
        }))
    } else {
        Err(Hook0Problem::NotFound)
    }
}

#[api_v2_operation(
    summary = "Edit an organization",
    description = "Note that you will need to regenerate a JWT to be able to see the updated name of the organization.",
    operation_id = "organizations.edit",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn edit(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    organization_id: Path<Uuid>,
    body: Json<OrganizationPost>,
) -> Result<Json<OrganizationInfo>, Hook0Problem> {
    if authorize(
        &biscuit,
        Some(organization_id.as_ref().to_owned()),
        Action::OrganizationEdit,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    query!(
        "
            UPDATE iam.organization
            SET name = $2
            WHERE organization__id = $1
        ",
        organization_id.as_ref(),
        &body.name,
    )
    .execute(&state.db)
    .await?;

    if let Some(hook0_client) = state.hook0_client.as_ref() {
        let hook0_client_event: Hook0ClientEvent = EventOrganizationUpdated {
            organization_id: organization_id.as_ref().to_owned(),
            name: body.name.to_owned(),
        }
        .into();
        if let Err(e) = hook0_client
            .send_event(&hook0_client_event.mk_hook0_event())
            .await
        {
            error!("Hook0ClientError: {e}");
        };
    }

    let org = get(state, OaBiscuit, biscuit, organization_id).await?;
    Ok(org)
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct UserInvitation {
    #[validate(non_control_character, email, length(max = 100))]
    email: String,
    role: String,
}

#[api_v2_operation(
    summary = "Invite a user to an organization",
    description = "Give permission to a user to access an organization you have write access to. The user must already have a Hook0 account otherwise this will fail.",
    operation_id = "organizations.invite",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn invite(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    organization_id: Path<Uuid>,
    body: Json<UserInvitation>,
) -> Result<Json<UserInvitation>, Hook0Problem> {
    let organization_id = organization_id.into_inner();

    if authorize(
        &biscuit,
        Some(organization_id),
        Action::OrganizationInvite,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    match Role::from_str(&body.role) {
        Ok(role) => {
            let user_id = query_scalar!(
                "
                    SELECT user__id
                    FROM iam.user
                    WHERE email = $1
                ",
                &body.email,
            )
            .fetch_optional(&state.db)
            .await?;

            match user_id {
                Some(uid) => {
                    query!(
                        "
                            INSERT INTO iam.user__organization (user__id, organization__id, role)
                            VALUES ($1, $2, $3)
                        ",
                        &uid,
                        &organization_id,
                        role.as_ref(),
                    )
                    .execute(&state.db)
                    .await?;

                    if let Some(hook0_client) = state.hook0_client.as_ref() {
                        let hook0_client_event: Hook0ClientEvent = EventOrganizationInvited {
                            organization_id,
                            user_id: uid,
                            email: body.email.to_owned(),
                            role: role.to_string(),
                        }
                        .into();
                        if let Err(e) = hook0_client
                            .send_event(&hook0_client_event.mk_hook0_event())
                            .await
                        {
                            error!("Hook0ClientError: {e}");
                        };
                    }

                    Ok(body)
                }
                None => Err(Hook0Problem::InvitedUserDoesNotExist),
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
    description = "Remove permission of a user to access an organization you have write access to.",
    operation_id = "organizations.revoke",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn revoke(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    organization_id: Path<Uuid>,
    body: Json<Revoke>,
) -> Result<Json<Revoke>, Hook0Problem> {
    let organization_id = organization_id.into_inner();

    if authorize(
        &biscuit,
        Some(organization_id),
        Action::OrganizationRevoke,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let user_role = query_scalar!(
        "
            SELECT role
            FROM iam.user__organization
            WHERE user__id = $1
                AND organization__id = $2
        ",
        &body.user_id,
        &organization_id,
    )
    .fetch_optional(&state.db)
    .await?;

    match user_role {
        Some(_) => {
            query!(
                "
                    DELETE FROM iam.user__organization
                    WHERE user__id = $1
                    AND organization__id = $2
                ",
                &body.user_id,
                &organization_id,
            )
            .execute(&state.db)
            .await?;

            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventOrganizationRevoked {
                    organization_id,
                    user_id: body.user_id,
                }
                .into();
                if let Err(e) = hook0_client
                    .send_event(&hook0_client_event.mk_hook0_event())
                    .await
                {
                    error!("Hook0ClientError: {e}");
                };
            }

            Ok(body)
        }
        None => Err(Hook0Problem::NotFound),
    }
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct OrganizationEditRole {
    user_id: Uuid,
    role: String,
}

#[api_v2_operation(
    summary = "Edit a user's role in an organization",
    description = "Change the role of a user that has already access to an organization you have write access to.",
    operation_id = "organizations.edit_role",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn edit_role(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    organization_id: Path<Uuid>,
    body: Json<OrganizationEditRole>,
) -> Result<Json<OrganizationEditRole>, Hook0Problem> {
    let organization_id = organization_id.into_inner();

    if let Ok(token) = authorize(
        &biscuit,
        Some(organization_id),
        Action::OrganizationEditRole,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    ) {
        if let AuthorizedToken::User(user_token) = token
            && user_token.user_id == body.user_id
        {
            return Err(Hook0Problem::Forbidden);
        }

        query!(
            "
                UPDATE iam.user__organization
                SET role = $1
                WHERE user__id = $2
                    AND organization__id = $3
            ",
            &body.role,
            &body.user_id,
            &organization_id,
        )
        .execute(&state.db)
        .await?;

        Ok(body)
    } else {
        Err(Hook0Problem::Forbidden)
    }
}

#[api_v2_operation(
    summary = "Delete an organization",
    description = "Remove an organization you have write access to. Organization must not contain any application otherwise this will fail.",
    operation_id = "organizations.delete",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn delete(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    organization_id: Path<Uuid>,
) -> Result<NoContent, Hook0Problem> {
    let organization_id = organization_id.into_inner();

    if authorize(
        &biscuit,
        Some(organization_id),
        Action::OrganizationDelete,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let organization_is_empty = query_scalar!(
        r#"
            SELECT COUNT(application__id) = 0 AS "empty!"
            FROM event.application
            WHERE organization__id = $1 AND deleted_at IS NULL
        "#,
        &organization_id,
    )
    .fetch_one(&state.db)
    .await?;

    if organization_is_empty {
        query!(
            "
                DELETE FROM iam.organization
                WHERE organization__id = $1
            ",
            &organization_id,
        )
        .execute(&state.db)
        .await?;

        if let Some(hook0_client) = state.hook0_client.as_ref() {
            let hook0_client_event: Hook0ClientEvent =
                EventOrganizationRemoved { organization_id }.into();
            if let Err(e) = hook0_client
                .send_event(&hook0_client_event.mk_hook0_event())
                .await
            {
                error!("Hook0ClientError: {e}");
            };
        }

        Ok(NoContent)
    } else {
        Err(Hook0Problem::OrganizationIsNotEmpty)
    }
}
