use actix_web_middleware_keycloak_auth::UnstructuredClaims;
use lazy_static::lazy_static;
use regex::{escape, Regex};
use sqlx::{query_as, PgPool};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

const GROUPS_CLAIM_NAME: &str = "groups";
const GROUP_SEP: &str = "/";
const ORGA_GROUP_PREFIX: &str = "orga_";
const ROLE_GROUP_PREFIX: &str = "role_";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    Viewer,
    Editor,
}

impl Default for Role {
    fn default() -> Self {
        Self::Viewer
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Editor => f.write_str("editor"),
            Self::Viewer => f.write_str("viewer"),
        }
    }
}

impl FromStr for Role {
    type Err = ();

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str.to_lowercase().as_str() {
            s if s == format!("{}{}", ROLE_GROUP_PREFIX, "editor") => Ok(Self::Editor),
            s if s == format!("{}{}", ROLE_GROUP_PREFIX, "viewer") => Ok(Self::Viewer),
            _ => Err(()),
        }
    }
}

pub fn extract_organizations(unstructured_claims: &UnstructuredClaims) -> HashMap<Uuid, Role> {
    lazy_static! {
        static ref RE: Regex = Regex::new(&format!(
            "^{}{}([0-9a-f-]+)(?:{}([0-9a-zA-Z_]+))?$",
            escape(GROUP_SEP),
            escape(ORGA_GROUP_PREFIX),
            escape(GROUP_SEP)
        ))
        .unwrap();
    }

    unstructured_claims
        .get::<Vec<String>>(GROUPS_CLAIM_NAME)
        .map(|strings| {
            let mut organizations = HashMap::new();
            for str in strings {
                let matches = RE.captures(str.as_str());
                if let Some(m) = matches {
                    let org_id_str = m.get(1).unwrap().as_str();
                    let role = m
                        .get(2)
                        .map(|regex_match| regex_match.as_str())
                        .and_then(|role_str| Role::from_str(role_str).ok())
                        .unwrap_or_default();
                    if let Ok(org_id) = Uuid::from_str(org_id_str) {
                        organizations.insert(org_id, role);
                    }
                }
            }
            organizations
        })
        .unwrap_or_else(|_| HashMap::new())
}

pub async fn can_access_organization(
    unstructured_claims: &UnstructuredClaims,
    organization_id: &Uuid,
    minimum_required_role: &Role,
) -> bool {
    let available_organizations = extract_organizations(unstructured_claims);
    match available_organizations.get(&organization_id) {
        Some(role) => role >= minimum_required_role,
        None => false,
    }
}

pub async fn can_access_application(
    db: &PgPool,
    unstructured_claims: &UnstructuredClaims,
    application_id: &Uuid,
    minimum_required_role: &Role,
) -> bool {
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
        let available_organizations = extract_organizations(unstructured_claims);
        match available_organizations.get(&id) {
            Some(role) => role >= minimum_required_role,
            None => false,
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::Value;
    use std::collections::HashMap;
    use std::iter::FromIterator;

    #[test]
    fn extract_all_organizations() {
        let groups_array = Value::Array(vec![
            Value::String(format!(
                "{}{}{}{}{}{}",
                GROUP_SEP,
                ORGA_GROUP_PREFIX,
                "1cd43b73-a5f0-4683-9961-cbd0c28ba565",
                GROUP_SEP,
                ROLE_GROUP_PREFIX,
                "viewer"
            )),
            Value::String(format!(
                "{}{}{}{}{}{}",
                GROUP_SEP,
                ORGA_GROUP_PREFIX,
                "7d41a6ad-de79-4990-8cb4-770f1c8c8974",
                GROUP_SEP,
                ROLE_GROUP_PREFIX,
                "editor"
            )),
        ]);
        let input = UnstructuredClaims(HashMap::from_iter(vec![(
            GROUPS_CLAIM_NAME.to_owned(),
            groups_array,
        )]));

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
        let found = extract_organizations(&input);
        assert_eq!(found, expected);
    }

    #[test]
    fn extract_only_valid_organizations() {
        let groups_array = Value::Array(vec![
            Value::String(format!(
                "{}{}{}{}{}{}",
                GROUP_SEP,
                ORGA_GROUP_PREFIX,
                "1cd43b73-a5f0-4683-9961-cbd0c28ba565",
                GROUP_SEP,
                ROLE_GROUP_PREFIX,
                "editor"
            )),
            Value::String(format!(
                "{}{}{}{}{}",
                GROUP_SEP,
                ORGA_GROUP_PREFIX,
                "782904f5-3122-4bea-9c21-88e5047037d5",
                GROUP_SEP,
                "unknown"
            )),
            Value::String(format!(
                "{}{}{}",
                GROUP_SEP, ORGA_GROUP_PREFIX, "37ae1500-0893-4123-9ba3-a2021586c40b"
            )),
            Value::String("d3e1116a-9733-4522-9831-7f8dc7509825".to_owned()),
            Value::String(format!(
                "{}{}{}",
                "cc8066dc-9f12-49cc-95d1-1e0723355162", GROUP_SEP, "role2"
            )),
        ]);
        let input = UnstructuredClaims(HashMap::from_iter(vec![(
            GROUPS_CLAIM_NAME.to_owned(),
            groups_array,
        )]));

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
        let found = extract_organizations(&input);
        assert_eq!(found, expected);
    }
}
