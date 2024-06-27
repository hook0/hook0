use actix::clock::sleep;
use anyhow::anyhow;
use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use clap::crate_version;
use hook0_client::{Hook0Client, Hook0ClientError};
use log::{error, info, trace, warn};
use reqwest::Url;
use serde::Serialize;
use serde_json::{to_string, to_value, Value};
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

use crate::handlers::subscriptions::Target;

const PERIOD_BETWEEN_EVENT_TYPES_UPSERTS_TRIES: Duration = Duration::from_secs(2);

pub const EVENT_TYPES: &[&str] = &[
    "api.organization.created",
    "api.organization.updated",
    "api.organization.invited",
    "api.organization.revoked",
    "api.organization.removed",
    "api.application.created",
    "api.application.updated",
    "api.application.removed",
    "api.application_secret.created",
    "api.application_secret.updated",
    "api.application_secret.removed",
    "api.service_token.created",
    "api.service_token.updated",
    "api.service_token.removed",
    "api.event_type.created",
    "api.event_type.removed",
    "api.subscription.created",
    "api.subscription.updated",
    "api.subscription.removed",
];

pub fn initialize(
    api_url: Option<Url>,
    application_id: Option<Uuid>,
    token: Option<String>,
) -> Option<Hook0Client> {
    match (api_url, application_id, token) {
        (Some(url), Some(id), Some(t)) => match Hook0Client::new(url, id, &t) {
            Ok(client) => {
                info!(
                    "Events from this Hook0 instance will be sent to {} [application ID = {}]",
                    client.api_url(),
                    client.application_id()
                );
                Some(client)
            }
            Err(_e) => {
                warn!("Could not initialize a Hook0 client that will receive events from this Hook0 instance");
                None
            }
        },
        _ => {
            info!("No Hook0 client was configured to receive events from this Hook0 instance");
            None
        }
    }
}

#[async_recursion]
pub async fn upsert_event_types(
    hook0_client: &Hook0Client,
    event_types: &[&str],
    retries: u16,
) -> () {
    fn log_error(e: anyhow::Error) {
        error!("Could not upsert event types, Hook0 client might not work: {e}")
    }

    info!("Starting upserting Hook0 client event types");
    match hook0_client.upsert_event_types(event_types).await {
        Ok(_added_event_types) => info!("Hook0 client event types upserting was successful"),
        Err(Hook0ClientError::GetAvailableEventTypes(e))
        | Err(Hook0ClientError::CreatingEventType {
            event_type_name: _,
            error: e,
        }) => {
            if e.is_connect()
                || e.is_timeout()
                || (e.status().is_some() && e.status().unwrap().is_server_error())
            {
                log_error(e.into());

                if retries != 0 {
                    trace!(
                        "Waiting {} seconds before retrying",
                        PERIOD_BETWEEN_EVENT_TYPES_UPSERTS_TRIES.as_secs()
                    );
                    sleep(PERIOD_BETWEEN_EVENT_TYPES_UPSERTS_TRIES).await;
                    upsert_event_types(hook0_client, event_types, retries - 1).await
                } else {
                    log_error(anyhow!("Too many retries"));
                }
            } else {
                log_error(e.into())
            }
        }
        Err(e) => log_error(e.into()),
    }
}

#[derive(Debug, Clone)]
pub enum Hook0ClientEvent {
    OrganizationCreated(EventOrganizationCreated),
    OrganizationUpdated(EventOrganizationUpdated),
    OrganizationInvited(EventOrganizationInvited),
    OrganizationRevoked(EventOrganizationRevoked),
    OrganizationRemoved(EventOrganizationRemoved),
    ApplicationCreated(EventApplicationCreated),
    ApplicationUpdated(EventApplicationUpdated),
    ApplicationRemoved(EventApplicationRemoved),
    ApplicationSecretCreated(EventApplicationSecretCreated),
    ApplicationSecretUpdated(EventApplicationSecretUpdated),
    ApplicationSecretRemoved(EventApplicationSecretRemoved),
    ServiceTokenCreated(EventServiceTokenCreated),
    ServiceTokenUpdated(EventServiceTokenUpdated),
    ServiceTokenRemoved(EventServiceTokenRemoved),
    EventTypeCreated(EventEventTypeCreated),
    EventTypeRemoved(EventEventTypeRemoved),
    SubscriptionCreated(EventSubscriptionCreated),
    SubscriptionUpdated(EventSubscriptionUpdated),
    SubscriptionRemoved(EventSubscriptionRemoved),
}

impl Hook0ClientEvent {
    pub fn mk_hook0_event<'a>(self) -> hook0_client::Event<'a> {
        fn to_event<'a, E: 'a + Event>(
            event: E,
            occurred_at: Option<DateTime<Utc>>,
        ) -> hook0_client::Event<'a> {
            hook0_client::Event {
                event_id: &None,
                event_type: event.event_type(),
                payload: Cow::from(to_string(&event).unwrap()),
                payload_content_type: "application/json",
                metadata: Some(vec![(
                    "hook0_version".to_owned(),
                    to_value(crate_version!()).unwrap(),
                )]),
                occurred_at,
                labels: event.labels(),
            }
        }

        match self {
            Self::OrganizationCreated(e) => to_event(e, None),
            Self::OrganizationUpdated(e) => to_event(e, None),
            Self::OrganizationInvited(e) => to_event(e, None),
            Self::OrganizationRevoked(e) => to_event(e, None),
            Self::OrganizationRemoved(e) => to_event(e, None),
            Self::ApplicationCreated(e) => to_event(e, None),
            Self::ApplicationUpdated(e) => to_event(e, None),
            Self::ApplicationRemoved(e) => to_event(e, None),
            Self::ApplicationSecretCreated(
                e @ EventApplicationSecretCreated { created_at, .. },
            ) => to_event(e, Some(created_at)),
            Self::ApplicationSecretUpdated(e) => to_event(e, None),
            Self::ApplicationSecretRemoved(e) => to_event(e, None),
            Self::ServiceTokenCreated(e @ EventServiceTokenCreated { created_at, .. }) => {
                to_event(e, Some(created_at))
            }
            Self::ServiceTokenUpdated(e) => to_event(e, None),
            Self::ServiceTokenRemoved(e) => to_event(e, None),
            Self::EventTypeCreated(e @ EventEventTypeCreated { created_at, .. }) => {
                to_event(e, Some(created_at))
            }
            Self::EventTypeRemoved(e) => to_event(e, None),
            Self::SubscriptionCreated(e @ EventSubscriptionCreated { created_at, .. }) => {
                to_event(e, Some(created_at))
            }
            Self::SubscriptionUpdated(e) => to_event(e, None),
            Self::SubscriptionRemoved(e) => to_event(e, None),
        }
    }
}

trait Event: std::fmt::Debug + Clone + Serialize {
    fn event_type(&self) -> &'static str;
    fn labels(&self) -> Vec<(String, Value)>;
}

const INSTANCE_LABEL: &str = "instance";
const INSTANCE_VALUE: &str = "1";
const ORGANIZATION_LABEL: &str = "organization";
const APPLICATION_LABEL: &str = "application";

#[derive(Debug, Clone, Serialize)]
pub struct EventOrganizationCreated {
    pub organization_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

impl Event for EventOrganizationCreated {
    fn event_type(&self) -> &'static str {
        "api.organization.created"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![(
            INSTANCE_LABEL.to_owned(),
            Value::String(INSTANCE_VALUE.to_owned()),
        )]
    }
}

impl From<EventOrganizationCreated> for Hook0ClientEvent {
    fn from(e: EventOrganizationCreated) -> Self {
        Self::OrganizationCreated(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventOrganizationUpdated {
    pub organization_id: Uuid,
    pub name: String,
}

impl Event for EventOrganizationUpdated {
    fn event_type(&self) -> &'static str {
        "api.organization.updated"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
        ]
    }
}

impl From<EventOrganizationUpdated> for Hook0ClientEvent {
    fn from(e: EventOrganizationUpdated) -> Self {
        Self::OrganizationUpdated(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventOrganizationInvited {
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
}

impl Event for EventOrganizationInvited {
    fn event_type(&self) -> &'static str {
        "api.organization.invited"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
        ]
    }
}

impl From<EventOrganizationInvited> for Hook0ClientEvent {
    fn from(e: EventOrganizationInvited) -> Self {
        Self::OrganizationInvited(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventOrganizationRevoked {
    pub organization_id: Uuid,
    pub user_id: Uuid,
}

impl Event for EventOrganizationRevoked {
    fn event_type(&self) -> &'static str {
        "api.organization.revoked"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
        ]
    }
}

impl From<EventOrganizationRevoked> for Hook0ClientEvent {
    fn from(e: EventOrganizationRevoked) -> Self {
        Self::OrganizationRevoked(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventOrganizationRemoved {
    pub organization_id: Uuid,
}

impl Event for EventOrganizationRemoved {
    fn event_type(&self) -> &'static str {
        "api.organization.removed"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
        ]
    }
}

impl From<EventOrganizationRemoved> for Hook0ClientEvent {
    fn from(e: EventOrganizationRemoved) -> Self {
        Self::OrganizationRemoved(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventApplicationCreated {
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub name: String,
}

impl Event for EventApplicationCreated {
    fn event_type(&self) -> &'static str {
        "api.application.created"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
            (
                APPLICATION_LABEL.to_owned(),
                to_value(self.application_id).unwrap(),
            ),
        ]
    }
}

impl From<EventApplicationCreated> for Hook0ClientEvent {
    fn from(e: EventApplicationCreated) -> Self {
        Self::ApplicationCreated(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventApplicationUpdated {
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub name: String,
}

impl Event for EventApplicationUpdated {
    fn event_type(&self) -> &'static str {
        "api.application.updated"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
            (
                APPLICATION_LABEL.to_owned(),
                to_value(self.application_id).unwrap(),
            ),
        ]
    }
}

impl From<EventApplicationUpdated> for Hook0ClientEvent {
    fn from(e: EventApplicationUpdated) -> Self {
        Self::ApplicationUpdated(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventApplicationRemoved {
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub name: String,
}

impl Event for EventApplicationRemoved {
    fn event_type(&self) -> &'static str {
        "api.application.removed"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
            (
                APPLICATION_LABEL.to_owned(),
                to_value(self.application_id).unwrap(),
            ),
        ]
    }
}

impl From<EventApplicationRemoved> for Hook0ClientEvent {
    fn from(e: EventApplicationRemoved) -> Self {
        Self::ApplicationRemoved(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventApplicationSecretCreated {
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Event for EventApplicationSecretCreated {
    fn event_type(&self) -> &'static str {
        "api.application_secret.created"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
            (
                APPLICATION_LABEL.to_owned(),
                to_value(self.application_id).unwrap(),
            ),
        ]
    }
}

impl From<EventApplicationSecretCreated> for Hook0ClientEvent {
    fn from(e: EventApplicationSecretCreated) -> Self {
        Self::ApplicationSecretCreated(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventApplicationSecretUpdated {
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub name: Option<String>,
}

impl Event for EventApplicationSecretUpdated {
    fn event_type(&self) -> &'static str {
        "api.application_secret.updated"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
            (
                APPLICATION_LABEL.to_owned(),
                to_value(self.application_id).unwrap(),
            ),
        ]
    }
}

impl From<EventApplicationSecretUpdated> for Hook0ClientEvent {
    fn from(e: EventApplicationSecretUpdated) -> Self {
        Self::ApplicationSecretUpdated(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventApplicationSecretRemoved {
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub name: Option<String>,
    pub token: Uuid,
}

impl Event for EventApplicationSecretRemoved {
    fn event_type(&self) -> &'static str {
        "api.application_secret.removed"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
            (
                APPLICATION_LABEL.to_owned(),
                to_value(self.application_id).unwrap(),
            ),
        ]
    }
}

impl From<EventApplicationSecretRemoved> for Hook0ClientEvent {
    fn from(e: EventApplicationSecretRemoved) -> Self {
        Self::ApplicationSecretRemoved(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventServiceTokenCreated {
    pub token_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl Event for EventServiceTokenCreated {
    fn event_type(&self) -> &'static str {
        "api.service_token.created"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
        ]
    }
}

impl From<EventServiceTokenCreated> for Hook0ClientEvent {
    fn from(e: EventServiceTokenCreated) -> Self {
        Self::ServiceTokenCreated(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventServiceTokenUpdated {
    pub token_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
}

impl Event for EventServiceTokenUpdated {
    fn event_type(&self) -> &'static str {
        "api.service_token.updated"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
        ]
    }
}

impl From<EventServiceTokenUpdated> for Hook0ClientEvent {
    fn from(e: EventServiceTokenUpdated) -> Self {
        Self::ServiceTokenUpdated(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventServiceTokenRemoved {
    pub token_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
}

impl Event for EventServiceTokenRemoved {
    fn event_type(&self) -> &'static str {
        "api.service_token.removed"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
        ]
    }
}

impl From<EventServiceTokenRemoved> for Hook0ClientEvent {
    fn from(e: EventServiceTokenRemoved) -> Self {
        Self::ServiceTokenRemoved(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventEventTypeCreated {
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub service_name: String,
    pub resource_type_name: String,
    pub verb_name: String,
    pub event_type_name: String,
    pub created_at: DateTime<Utc>,
}

impl Event for EventEventTypeCreated {
    fn event_type(&self) -> &'static str {
        "api.event_type.created"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
            (
                APPLICATION_LABEL.to_owned(),
                to_value(self.application_id).unwrap(),
            ),
        ]
    }
}

impl From<EventEventTypeCreated> for Hook0ClientEvent {
    fn from(e: EventEventTypeCreated) -> Self {
        Self::EventTypeCreated(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventEventTypeRemoved {
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub event_type_name: String,
}

impl Event for EventEventTypeRemoved {
    fn event_type(&self) -> &'static str {
        "api.event_type.removed"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
            (
                APPLICATION_LABEL.to_owned(),
                to_value(self.application_id).unwrap(),
            ),
        ]
    }
}

impl From<EventEventTypeRemoved> for Hook0ClientEvent {
    fn from(e: EventEventTypeRemoved) -> Self {
        Self::EventTypeRemoved(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventSubscriptionCreated {
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub subscription_id: Uuid,
    pub is_enabled: bool,
    pub event_types: Vec<String>,
    pub description: Option<String>,
    pub metadata: HashMap<String, Value>,
    pub label_key: String,
    pub label_value: String,
    pub target: Target,
    pub created_at: DateTime<Utc>,
}

impl Event for EventSubscriptionCreated {
    fn event_type(&self) -> &'static str {
        "api.subscription.created"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
            (
                APPLICATION_LABEL.to_owned(),
                to_value(self.application_id).unwrap(),
            ),
        ]
    }
}

impl From<EventSubscriptionCreated> for Hook0ClientEvent {
    fn from(e: EventSubscriptionCreated) -> Self {
        Self::SubscriptionCreated(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventSubscriptionUpdated {
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub subscription_id: Uuid,
    pub is_enabled: bool,
    pub event_types: Vec<String>,
    pub description: Option<String>,
    pub metadata: HashMap<String, Value>,
    pub label_key: String,
    pub label_value: String,
    pub target: Target,
    pub created_at: DateTime<Utc>,
}

impl Event for EventSubscriptionUpdated {
    fn event_type(&self) -> &'static str {
        "api.subscription.updated"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
            (
                APPLICATION_LABEL.to_owned(),
                to_value(self.application_id).unwrap(),
            ),
        ]
    }
}

impl From<EventSubscriptionUpdated> for Hook0ClientEvent {
    fn from(e: EventSubscriptionUpdated) -> Self {
        Self::SubscriptionUpdated(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventSubscriptionRemoved {
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub subscription_id: Uuid,
}

impl Event for EventSubscriptionRemoved {
    fn event_type(&self) -> &'static str {
        "api.subscription.removed"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![
            (
                INSTANCE_LABEL.to_owned(),
                Value::String(INSTANCE_VALUE.to_owned()),
            ),
            (
                ORGANIZATION_LABEL.to_owned(),
                Value::String(self.organization_id.to_string()),
            ),
            (
                APPLICATION_LABEL.to_owned(),
                to_value(self.application_id).unwrap(),
            ),
        ]
    }
}

impl From<EventSubscriptionRemoved> for Hook0ClientEvent {
    fn from(e: EventSubscriptionRemoved) -> Self {
        Self::SubscriptionRemoved(e)
    }
}
