use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{NaiveDate, Utc};
use paperclip::actix::web::{Data, Json, Query};
use paperclip::actix::{Apiv2Schema, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use uuid::Uuid;

use crate::iam::{Action, authorize, authorize_for_application};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct EventsPerDayEntry {
    application_id: Uuid,
    application_name: String,
    date: NaiveDate,
    amount: i32,
    is_provisional: bool,
}

impl From<EventsPerDayEntryRaw> for EventsPerDayEntry {
    fn from(value: EventsPerDayEntryRaw) -> Self {
        EventsPerDayEntry {
            application_id: value.application__id,
            application_name: value.application_name,
            date: value.date,
            amount: value.amount,
            is_provisional: value.is_provisional,
        }
    }
}

#[allow(non_snake_case)]
struct EventsPerDayEntryRaw {
    application__id: Uuid,
    application_name: String,
    date: NaiveDate,
    amount: i32,
    is_provisional: bool,
}

#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct ApplicationQs {
    application_id: Uuid,
    /// Start of date range (inclusive). Defaults to 30 days before `to`.
    from: Option<NaiveDate>,
    /// End of date range (inclusive). Defaults to today.
    to: Option<NaiveDate>,
}

#[api_v2_operation(
    summary = "List events per day for an application",
    description = "Retrieves the number of events ingested per day for a specific application.",
    operation_id = "events_per_day.list_for_application",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management")
)]
pub async fn application(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<ApplicationQs>,
) -> Result<Json<Vec<EventsPerDayEntry>>, Hook0Problem> {
    let today = Utc::now().date_naive();
    let to = qs.to.unwrap_or(today);
    let from = qs.from.unwrap_or_else(|| to - chrono::Duration::days(30));

    if from > to {
        return Err(Hook0Problem::InvalidDateRange);
    }

    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::EventsPerDayApplication {
            application_id: &qs.application_id,
        },
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let entries = query_as!(
        EventsPerDayEntryRaw,
        r#"
            SELECT
                epd.application__id AS "application__id!",
                a.name AS application_name,
                epd.date AS "date!",
                epd.amount AS "amount!",
                epd.is_provisional AS "is_provisional!"
            FROM (
                SELECT application__id, date, amount, false::bool AS is_provisional
                FROM event.all_time_events_per_day
                WHERE date < $1
                UNION ALL
                SELECT application__id, date, amount, true::bool AS is_provisional
                FROM event.events_per_day
                WHERE date = $1
            ) AS epd
            INNER JOIN event.application AS a ON a.application__id = epd.application__id
            WHERE epd.application__id = $2
                AND epd.date >= $3
                AND epd.date <= $4
                AND a.deleted_at IS NULL
            ORDER BY epd.date ASC
        "#,
        today,
        &qs.application_id,
        &from,
        &to,
    )
    .fetch_all(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .into_iter()
    .map(|e| e.into())
    .collect();

    Ok(Json(entries))
}

#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct OrganizationQs {
    organization_id: Uuid,
    /// Start of date range (inclusive). Defaults to 30 days before `to`.
    from: Option<NaiveDate>,
    /// End of date range (inclusive). Defaults to today.
    to: Option<NaiveDate>,
}

#[api_v2_operation(
    summary = "List events per day for an organization",
    description = "Retrieves the number of events ingested per day across all applications in an organization.",
    operation_id = "events_per_day.list_for_organization",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management")
)]
pub async fn organization(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<OrganizationQs>,
) -> Result<Json<Vec<EventsPerDayEntry>>, Hook0Problem> {
    let today = Utc::now().date_naive();
    let to = qs.to.unwrap_or(today);
    let from = qs.from.unwrap_or_else(|| to - chrono::Duration::days(30));

    if from > to {
        return Err(Hook0Problem::InvalidDateRange);
    }

    if authorize(
        &biscuit,
        Some(qs.organization_id),
        Action::EventsPerDayOrganization,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let entries = query_as!(
        EventsPerDayEntryRaw,
        r#"
            SELECT
                epd.application__id AS "application__id!",
                a.name AS application_name,
                epd.date AS "date!",
                epd.amount AS "amount!",
                epd.is_provisional AS "is_provisional!"
            FROM (
                SELECT application__id, date, amount, false::bool AS is_provisional
                FROM event.all_time_events_per_day
                WHERE date < $1
                UNION ALL
                SELECT application__id, date, amount, true::bool AS is_provisional
                FROM event.events_per_day
                WHERE date = $1
            ) AS epd
            INNER JOIN event.application AS a ON a.application__id = epd.application__id
            WHERE a.organization__id = $2
                AND epd.date >= $3
                AND epd.date <= $4
            ORDER BY epd.date ASC, a.name ASC
        "#,
        today,
        &qs.organization_id,
        &from,
        &to,
    )
    .fetch_all(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .into_iter()
    .map(|e| e.into())
    .collect();

    Ok(Json(entries))
}
