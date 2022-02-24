use actix_web::{FromRequest, HttpMessage, ResponseError};
use futures_util::future::{ready, Ready};
use lazy_static::lazy_static;
use paperclip::actix::OperationModifier;
use paperclip::v2::schema::{Apiv2Schema, TypedData};
use regex::{escape, Regex};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, PgPool};
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
use strum::{EnumIter, EnumString, EnumVariantNames};
use uuid::Uuid;

pub const GROUP_SEP: &str = "/";
pub const ORGA_GROUP_PREFIX: &str = "orga_";
const ROLE_GROUP_PREFIX: &str = "role_";

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    strum::Display,
    EnumString,
    EnumIter,
    EnumVariantNames,
)]
#[strum(serialize_all = "snake_case")]
pub enum Role {
    Viewer,
    Editor,
}

impl Default for Role {
    fn default() -> Self {
        Self::Viewer
    }
}

impl TypedData for Role {
    fn data_type() -> paperclip::v2::models::DataType {
        paperclip::v2::models::DataType::String
    }

    fn format() -> Option<paperclip::v2::models::DataTypeFormat> {
        None
    }
}

impl Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl Role {
    pub fn from_string_with_prefix(str: &str) -> Option<Self> {
        str.strip_prefix(ROLE_GROUP_PREFIX)
            .and_then(|s| Self::from_str(s).ok())
    }

    pub fn string_with_prefix(&self) -> String {
        format!("{ROLE_GROUP_PREFIX}{self}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Hook0Claims {
    pub sub: Uuid,
    pub groups: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AuthProof {
    Jwt {
        claims: Hook0Claims,
    },
    ApplicationSecret {
        secret: Uuid,
        name: Option<String>,
        application_id: Uuid,
    },
}

impl AuthProof {
    pub fn organizations(&self) -> HashMap<Uuid, Role> {
        if let Self::Jwt { claims } = self {
            lazy_static! {
                static ref RE: Regex = Regex::new(&format!(
                    "^{}{}([0-9a-f-]+)(?:{}([0-9a-zA-Z_]+))?$",
                    escape(GROUP_SEP),
                    escape(ORGA_GROUP_PREFIX),
                    escape(GROUP_SEP)
                ))
                .unwrap();
            }

            claims
                .groups
                .as_ref()
                .map(|strings| {
                    let mut organizations = HashMap::new();
                    for str in strings {
                        let matches = RE.captures(str.as_str());
                        if let Some(m) = matches {
                            let org_id_str = m.get(1).unwrap().as_str();
                            let role = m
                                .get(2)
                                .map(|regex_match| regex_match.as_str())
                                .and_then(Role::from_string_with_prefix)
                                .unwrap_or_default();
                            if let Ok(org_id) = Uuid::from_str(org_id_str) {
                                organizations.insert(org_id, role);
                            }
                        }
                    }
                    organizations
                })
                .unwrap_or_else(HashMap::new)
        } else {
            HashMap::new()
        }
    }

    pub async fn can_access_organization(
        &self,
        organization_id: &Uuid,
        minimum_required_role: &Role,
    ) -> Option<&Self> {
        let available_organizations = self.organizations();
        match available_organizations.get(organization_id) {
            Some(role) if role >= minimum_required_role => Some(self),
            _ => None,
        }
    }

    pub async fn can_access_application(
        &self,
        db: &PgPool,
        application_id: &Uuid,
        minimum_required_role: &Role,
    ) -> Option<&Self> {
        match self {
            Self::ApplicationSecret {
                application_id: provided_application_id,
                name: _,
                secret: _,
            } => {
                // Providing an application secret implies having the Editor role on the application
                if provided_application_id == application_id {
                    Some(self)
                } else {
                    None
                }
            }
            Self::Jwt { claims: _ } => {
                struct Organization {
                    pub id: Uuid,
                }

                let org = query_as!(
                    Organization,
                    "SELECT organization__id AS id FROM event.application WHERE application__id = $1",
                    application_id
                )
                .fetch_one(db)
                .await;

                if let Ok(Organization { id }) = org {
                    let available_organizations = self.organizations();
                    match available_organizations.get(&id) {
                        Some(role) if role >= minimum_required_role => Some(self),
                        _ => None,
                    }
                } else {
                    None
                }
            }
        }
    }
}

impl Apiv2Schema for AuthProof {}
impl OperationModifier for AuthProof {}

impl FromRequest for AuthProof {
    type Error = AuthProofExtractorError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let req_data = req.extensions();
        ready(
            req_data
                .get::<Self>()
                .map(|auth_proof| auth_proof.to_owned())
                .ok_or(AuthProofExtractorError),
        )
    }
}

#[derive(Debug)]
pub struct AuthProofExtractorError;

impl Display for AuthProofExtractorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AuthProof cannot be extracted from ReqData")
    }
}

impl ResponseError for AuthProofExtractorError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter::FromIterator;

    use super::*;

    #[test]
    fn jwt_all_organizations() {
        let groups = vec![
            format!(
                "{}{}{}{}{}{}",
                GROUP_SEP,
                ORGA_GROUP_PREFIX,
                "1cd43b73-a5f0-4683-9961-cbd0c28ba565",
                GROUP_SEP,
                ROLE_GROUP_PREFIX,
                "viewer"
            ),
            format!(
                "{}{}{}{}{}{}",
                GROUP_SEP,
                ORGA_GROUP_PREFIX,
                "7d41a6ad-de79-4990-8cb4-770f1c8c8974",
                GROUP_SEP,
                ROLE_GROUP_PREFIX,
                "editor"
            ),
        ];
        let auth = AuthProof::Jwt {
            claims: Hook0Claims {
                sub: Uuid::new_v4(),
                groups: Some(groups),
            },
        };

        let expected: HashMap<Uuid, Role> = HashMap::from_iter(vec![
            (
                Uuid::from_str("1cd43b73-a5f0-4683-9961-cbd0c28ba565").unwrap(),
                Role::Viewer,
            ),
            (
                Uuid::from_str("7d41a6ad-de79-4990-8cb4-770f1c8c8974").unwrap(),
                Role::Editor,
            ),
        ]);
        let found = auth.organizations();
        assert_eq!(found, expected);
    }

    #[test]
    fn jwt_only_valid_organizations() {
        let groups = vec![
            format!(
                "{}{}{}{}{}{}",
                GROUP_SEP,
                ORGA_GROUP_PREFIX,
                "1cd43b73-a5f0-4683-9961-cbd0c28ba565",
                GROUP_SEP,
                ROLE_GROUP_PREFIX,
                "editor"
            ),
            format!(
                "{}{}{}{}{}",
                GROUP_SEP,
                ORGA_GROUP_PREFIX,
                "782904f5-3122-4bea-9c21-88e5047037d5",
                GROUP_SEP,
                "unknown"
            ),
            format!(
                "{}{}{}",
                GROUP_SEP, ORGA_GROUP_PREFIX, "37ae1500-0893-4123-9ba3-a2021586c40b"
            ),
            "d3e1116a-9733-4522-9831-7f8dc7509825".to_owned(),
            format!(
                "{}{}{}",
                "cc8066dc-9f12-49cc-95d1-1e0723355162", GROUP_SEP, "role2"
            ),
        ];
        let auth = AuthProof::Jwt {
            claims: Hook0Claims {
                sub: Uuid::new_v4(),
                groups: Some(groups),
            },
        };

        let expected: HashMap<Uuid, Role> = HashMap::from_iter(vec![
            (
                Uuid::from_str("1cd43b73-a5f0-4683-9961-cbd0c28ba565").unwrap(),
                Role::Editor,
            ),
            (
                Uuid::from_str("782904f5-3122-4bea-9c21-88e5047037d5").unwrap(),
                Role::Viewer,
            ),
            (
                Uuid::from_str("37ae1500-0893-4123-9ba3-a2021586c40b").unwrap(),
                Role::Viewer,
            ),
        ]);
        let found = auth.organizations();
        assert_eq!(found, expected);
    }

    #[test]
    fn jwt_no_organization() {
        let auth = AuthProof::Jwt {
            claims: Hook0Claims {
                sub: Uuid::new_v4(),
                groups: None,
            },
        };
        let expected = HashMap::new();
        let found = auth.organizations();

        assert_eq!(found, expected);
    }
}
