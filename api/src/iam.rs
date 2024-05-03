use biscuit_auth::builder::Fact;
use biscuit_auth::builder_ext::AuthorizerExt;
use biscuit_auth::macros::{authorizer, authorizer_merge, biscuit, fact, rule};
use biscuit_auth::{AuthorizerLimits, Biscuit, KeyPair, PrivateKey};
use chrono::{DateTime, Utc};
use log::{error, trace, warn};
use paperclip::v2::schema::TypedData;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, PgPool};
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use strum::{AsRefStr, EnumIter, EnumString, VariantNames};
use uuid::Uuid;

#[cfg(feature = "migrate-users-from-keycloak")]
const GROUP_SEP: &str = "/";
#[cfg(feature = "migrate-users-from-keycloak")]
const ORGA_GROUP_PREFIX: &str = "orga_";
#[cfg(feature = "migrate-users-from-keycloak")]
const ROLE_GROUP_PREFIX: &str = "role_";

pub async fn get_owner_organization(db: &PgPool, application_id: &Uuid) -> Option<Uuid> {
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

    match org {
        Ok(Organization { id }) => Some(id),
        Err(e) => {
            error!("{e}");
            None
        }
    }
}

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
    VariantNames,
    AsRefStr,
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
    #[cfg(feature = "migrate-users-from-keycloak")]
    pub fn from_string_with_prefix(str: &str) -> Option<Self> {
        str.strip_prefix(ROLE_GROUP_PREFIX)
            .and_then(|s| Self::from_str(s).ok())
    }
}

#[cfg(feature = "migrate-users-from-keycloak")]
pub fn kc_group_paths_to_roles(groups: &[String]) -> std::collections::HashMap<Uuid, Role> {
    lazy_static::lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new(&format!(
            "^{}{}([0-9a-f-]+)(?:{}([0-9a-zA-Z_]+))?$",
            regex::escape(GROUP_SEP),
            regex::escape(ORGA_GROUP_PREFIX),
            regex::escape(GROUP_SEP)
        ))
        .unwrap();
    }

    let mut organizations = std::collections::HashMap::new();

    for str in groups {
        let matches = RE.captures(str);
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Hook0Claims {
    pub sub: Uuid,
    pub email: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub groups: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Hook0UserInfo {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RootToken {
    pub biscuit: Biscuit,
    pub serialized_biscuit: String,
    pub revocation_id: Vec<u8>,
    pub expired_at: Option<DateTime<Utc>>,
}

const USER_ACCESS_TOKEN_VERSION: i64 = 1;
const USER_ACCESS_TOKEN_EXPIRATION: Duration = Duration::from_secs(60 * 5);

#[allow(clippy::too_many_arguments)]
pub fn create_user_access_token(
    private_key: &PrivateKey,
    token_id: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    email: &str,
    first_name: &str,
    last_name: &str,
    roles: Vec<(Uuid, String)>,
) -> Result<RootToken, biscuit_auth::error::Token> {
    let keypair = KeyPair::from(private_key);
    let created_at = SystemTime::now();
    let expired_at = created_at + USER_ACCESS_TOKEN_EXPIRATION;

    let biscuit = {
        let mut biscuit = biscuit!(
            r#"
                type("user_access");
                version({USER_ACCESS_TOKEN_VERSION});
                session_id({session_id});
                token_id({token_id});
                created_at({created_at});
                user_id({user_id});
                email({email});
                first_name({first_name});
                last_name({last_name});

                check if time($t), $t < {expired_at};
            "#,
        );
        for (organization_id, role) in roles {
            biscuit.add_fact(fact!("organization_role({organization_id}, {role})"))?;
        }
        biscuit.build(&keypair)?
    };
    let serialized_biscuit = biscuit.to_base64()?;
    let revocation_id = biscuit
        .revocation_identifiers()
        .first()
        .map(|rid| rid.to_owned())
        .ok_or(biscuit_auth::error::Token::InternalError)?;

    Ok(RootToken {
        biscuit,
        serialized_biscuit,
        revocation_id,
        expired_at: Some(DateTime::from(expired_at)),
    })
}

const REFRESH_TOKEN_VERSION: i64 = 1;
const REFRESH_TOKEN_EXPIRATION: Duration = Duration::from_secs(60 * 30);

pub fn create_refresh_token(
    private_key: &PrivateKey,
    token_id: Uuid,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<RootToken, biscuit_auth::error::Token> {
    let keypair = KeyPair::from(private_key);
    let created_at = SystemTime::now();
    let expired_at = created_at + REFRESH_TOKEN_EXPIRATION;

    let biscuit = biscuit!(
        r#"
            type("refresh");
            version({REFRESH_TOKEN_VERSION});
            token_id({token_id});
            session_id({session_id});
            user_id({user_id});
            created_at({created_at});

            check if time($t), $t < {expired_at};
        "#,
    )
    .build(&keypair)?;
    let serialized_biscuit = biscuit.to_base64()?;
    let revocation_id = biscuit
        .revocation_identifiers()
        .first()
        .map(|rid| rid.to_owned())
        .ok_or(biscuit_auth::error::Token::InternalError)?;

    Ok(RootToken {
        biscuit,
        serialized_biscuit,
        revocation_id,
        expired_at: Some(DateTime::from(expired_at)),
    })
}

const SERVICE_ACCESS_TOKEN_VERSION: i64 = 1;

pub fn create_service_access_token(
    private_key: &PrivateKey,
    token_id: Uuid,
    organization_id: Uuid,
) -> Result<RootToken, biscuit_auth::error::Token> {
    let keypair = KeyPair::from(private_key);
    let created_at = SystemTime::now();

    let biscuit = biscuit!(
        r#"
            type("service_access");
            version({SERVICE_ACCESS_TOKEN_VERSION});
            token_id({token_id});
            created_at({created_at});
            organization_id({organization_id});
        "#,
    )
    .build(&keypair)?;
    let serialized_biscuit = biscuit.to_base64()?;
    let revocation_id = biscuit
        .revocation_identifiers()
        .first()
        .map(|rid| rid.to_owned())
        .ok_or(biscuit_auth::error::Token::InternalError)?;

    Ok(RootToken {
        biscuit,
        serialized_biscuit,
        revocation_id,
        expired_at: None,
    })
}

const EMAIL_VERIFICATION_TOKEN_VERSION: i64 = 1;

pub fn create_email_verification_token(
    private_key: &PrivateKey,
    user_id: Uuid
) -> Result<RootToken, biscuit_auth::error::Token> {
    let keypair = KeyPair::from(private_key);
    let created_at = SystemTime::now();

    let biscuit = biscuit!(
        r#"
            type("email_verification");
            version({EMAIL_VERIFICATION_TOKEN_VERSION});
            user_id({user_id});
            created_at({created_at});
        "#,
    )
    .build(&keypair)?;
    let serialized_biscuit = biscuit.to_base64()?;
    let revocation_id = biscuit
        .revocation_identifiers()
        .first()
        .map(|rid| rid.to_owned())
        .ok_or(biscuit_auth::error::Token::InternalError)?;

    Ok(RootToken {
        biscuit,
        serialized_biscuit,
        revocation_id,
        expired_at: None,
    })
}

pub fn create_master_access_token(
    private_key: &PrivateKey,
) -> Result<Biscuit, biscuit_auth::error::Token> {
    let keypair = KeyPair::from(private_key);
    let created_at = SystemTime::now();

    let biscuit = biscuit!(
        r#"
            type("master_access");
            created_at({created_at});
        "#,
    )
    .build(&keypair)?;

    Ok(biscuit)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action<'a> {
    #[cfg(test)]
    TestSimple,
    #[cfg(test)]
    TestWithApplication {
        application_id: &'a Uuid,
    },
    #[cfg(test)]
    TestNoOrganization,
    //
    AuthLogout,
    //
    OrganizationList,
    OrganizationCreate,
    OrganizationGet,
    OrganizationEdit,
    OrganizationInvite,
    OrganizationRevoke,
    OrganizationDelete,
    //
    ServiceTokenList,
    ServiceTokenCreate,
    ServiceTokenEdit {
        service_token_id: &'a Uuid,
    },
    ServiceTokenDelete {
        service_token_id: &'a Uuid,
    },
    //
    ApplicationList,
    ApplicationCreate,
    ApplicationGet {
        application_id: &'a Uuid,
    },
    ApplicationEdit {
        application_id: &'a Uuid,
    },
    ApplicationDelete {
        application_id: &'a Uuid,
    },
    //
    #[cfg(feature = "application-secret-compatibility")]
    ApplicationSecretList {
        application_id: &'a Uuid,
    },
    #[cfg(feature = "application-secret-compatibility")]
    ApplicationSecretCreate {
        application_id: &'a Uuid,
    },
    #[cfg(feature = "application-secret-compatibility")]
    ApplicationSecretEdit {
        application_id: &'a Uuid,
    },
    #[cfg(feature = "application-secret-compatibility")]
    ApplicationSecretDelete {
        application_id: &'a Uuid,
    },
    //
    EventTypeList {
        application_id: &'a Uuid,
    },
    EventTypeCreate {
        application_id: &'a Uuid,
    },
    EventTypeGet {
        application_id: &'a Uuid,
    },
    EventTypeDelete {
        application_id: &'a Uuid,
    },
    //
    SubscriptionList {
        application_id: &'a Uuid,
    },
    SubscriptionCreate {
        application_id: &'a Uuid,
        label_key: &'a str,
        label_value: &'a str,
    },
    SubscriptionGet {
        application_id: &'a Uuid,
        subscription_id: &'a Uuid,
    },
    SubscriptionEdit {
        application_id: &'a Uuid,
        subscription_id: &'a Uuid,
    },
    SubscriptionDelete {
        application_id: &'a Uuid,
        subscription_id: &'a Uuid,
    },
    //
    EventList {
        application_id: &'a Uuid,
    },
    EventGet {
        application_id: &'a Uuid,
    },
    EventIngest {
        application_id: &'a Uuid,
    },
    //
    RequestAttemptList {
        application_id: &'a Uuid,
    },
    //
    ResponseGet {
        application_id: &'a Uuid,
    },
}

impl<'a> Action<'a> {
    fn action_name(&self) -> &'static str {
        match self {
            #[cfg(test)]
            Self::TestSimple => "test:simple",
            #[cfg(test)]
            Self::TestWithApplication { .. } => "test:with_application",
            #[cfg(test)]
            Self::TestNoOrganization => "test:no_organization",
            //
            Self::AuthLogout => "auth:logout",
            //
            Self::OrganizationList => "organization:list",
            Self::OrganizationCreate => "organization:create",
            Self::OrganizationGet => "organization:get",
            Self::OrganizationEdit => "organization:edit",
            Self::OrganizationInvite => "organization:invite",
            Self::OrganizationRevoke => "organization:revoke",
            Self::OrganizationDelete => "organization:delete",
            //
            Self::ServiceTokenList => "service_token:list",
            Self::ServiceTokenCreate => "service_token:create",
            Self::ServiceTokenEdit { .. } => "service_token:edit",
            Self::ServiceTokenDelete { .. } => "service_token:delete",
            //
            Self::ApplicationList { .. } => "application:list",
            Self::ApplicationCreate { .. } => "application:create",
            Self::ApplicationGet { .. } => "application:get",
            Self::ApplicationEdit { .. } => "application:edit",
            Self::ApplicationDelete { .. } => "application:delete",
            //
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretList { .. } => "application_secret:list",
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretCreate { .. } => "application_secret:create",
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretEdit { .. } => "application_secret:edit",
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretDelete { .. } => "application_secret:delete",
            //
            Self::EventTypeList { .. } => "event_type:list",
            Self::EventTypeCreate { .. } => "event_type:create",
            Self::EventTypeGet { .. } => "event_type:get",
            Self::EventTypeDelete { .. } => "event_type:delete",
            //
            Self::SubscriptionList { .. } => "subscription:list",
            Self::SubscriptionCreate { .. } => "subscription:create",
            Self::SubscriptionGet { .. } => "subscription:get",
            Self::SubscriptionEdit { .. } => "subscription:edit",
            Self::SubscriptionDelete { .. } => "subscription:delete",
            //
            Self::EventList { .. } => "event:list",
            Self::EventGet { .. } => "event:get",
            Self::EventIngest { .. } => "event:ingest",
            //
            Self::RequestAttemptList { .. } => "request_attempt:list",
            //
            Self::ResponseGet { .. } => "response:get",
        }
    }

    fn allowed_roles(&self) -> Vec<Role> {
        let mut roles = vec![Role::Editor];

        let mut per_action_roles = match self {
            #[cfg(test)]
            Self::TestSimple => vec![Role::Viewer],
            #[cfg(test)]
            Self::TestWithApplication { .. } => vec![],
            #[cfg(test)]
            Self::TestNoOrganization => vec![],
            //
            Self::AuthLogout => vec![],
            //
            Self::OrganizationList => vec![],
            Self::OrganizationCreate => vec![],
            Self::OrganizationGet => vec![Role::Viewer],
            Self::OrganizationEdit => vec![],
            Self::OrganizationInvite => vec![],
            Self::OrganizationRevoke => vec![],
            Self::OrganizationDelete => vec![],
            //
            Self::ServiceTokenList => vec![Role::Viewer],
            Self::ServiceTokenCreate => vec![],
            Self::ServiceTokenEdit { .. } => vec![],
            Self::ServiceTokenDelete { .. } => vec![],
            //
            Self::ApplicationList { .. } => vec![Role::Viewer],
            Self::ApplicationCreate { .. } => vec![],
            Self::ApplicationGet { .. } => vec![Role::Viewer],
            Self::ApplicationEdit { .. } => vec![],
            Self::ApplicationDelete { .. } => vec![],
            //
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretList { .. } => vec![Role::Viewer],
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretCreate { .. } => vec![],
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretEdit { .. } => vec![],
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretDelete { .. } => vec![],
            //
            Self::EventTypeList { .. } => vec![Role::Viewer],
            Self::EventTypeCreate { .. } => vec![],
            Self::EventTypeGet { .. } => vec![Role::Viewer],
            Self::EventTypeDelete { .. } => vec![],
            //
            Self::SubscriptionList { .. } => vec![Role::Viewer],
            Self::SubscriptionCreate { .. } => vec![],
            Self::SubscriptionGet { .. } => vec![Role::Viewer],
            Self::SubscriptionEdit { .. } => vec![],
            Self::SubscriptionDelete { .. } => vec![],
            //
            Self::EventList { .. } => vec![Role::Viewer],
            Self::EventGet { .. } => vec![Role::Viewer],
            Self::EventIngest { .. } => vec![],
            //
            Self::RequestAttemptList { .. } => vec![Role::Viewer],
            //
            Self::ResponseGet { .. } => vec![Role::Viewer],
        };

        roles.append(&mut per_action_roles);
        roles
    }

    pub fn can_work_without_organization(&self) -> bool {
        match self {
            #[cfg(test)]
            Self::TestNoOrganization => true,
            //
            Self::AuthLogout => true,
            //
            Self::OrganizationList => true,
            Self::OrganizationCreate => true,
            //
            _ => false,
        }
    }

    pub fn application_id(&self) -> Option<Uuid> {
        match self {
            #[cfg(test)]
            Self::TestSimple => None,
            #[cfg(test)]
            Self::TestWithApplication { application_id, .. } => Some(**application_id),
            #[cfg(test)]
            Self::TestNoOrganization => None,
            //
            Self::AuthLogout => None,
            //
            Self::OrganizationList => None,
            Self::OrganizationCreate => None,
            Self::OrganizationGet => None,
            Self::OrganizationEdit => None,
            Self::OrganizationInvite => None,
            Self::OrganizationRevoke => None,
            Self::OrganizationDelete => None,
            //
            Self::ServiceTokenList => None,
            Self::ServiceTokenCreate => None,
            Self::ServiceTokenEdit { .. } => None,
            Self::ServiceTokenDelete { .. } => None,
            //
            Self::ApplicationList => None,
            Self::ApplicationCreate => None,
            Self::ApplicationGet { application_id, .. } => Some(**application_id),
            Self::ApplicationEdit { application_id, .. } => Some(**application_id),
            Self::ApplicationDelete { application_id, .. } => Some(**application_id),
            //
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretList { application_id, .. } => Some(**application_id),
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretCreate { application_id, .. } => Some(**application_id),
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretEdit { application_id, .. } => Some(**application_id),
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretDelete { application_id, .. } => Some(**application_id),
            //
            Self::EventTypeList { application_id, .. } => Some(**application_id),
            Self::EventTypeCreate { application_id, .. } => Some(**application_id),
            Self::EventTypeGet { application_id, .. } => Some(**application_id),
            Self::EventTypeDelete { application_id, .. } => Some(**application_id),
            //
            Self::SubscriptionList { application_id, .. } => Some(**application_id),
            Self::SubscriptionCreate { application_id, .. } => Some(**application_id),
            Self::SubscriptionGet { application_id, .. } => Some(**application_id),
            Self::SubscriptionEdit { application_id, .. } => Some(**application_id),
            Self::SubscriptionDelete { application_id, .. } => Some(**application_id),
            //
            Self::EventList { application_id, .. } => Some(**application_id),
            Self::EventGet { application_id, .. } => Some(**application_id),
            Self::EventIngest { application_id, .. } => Some(**application_id),
            //
            Self::RequestAttemptList { application_id, .. } => Some(**application_id),
            //
            Self::ResponseGet { application_id, .. } => Some(**application_id),
        }
    }

    pub fn generate_facts(self) -> Vec<Fact> {
        let mut facts = match self {
            #[cfg(test)]
            Self::TestSimple => vec![],
            #[cfg(test)]
            Self::TestWithApplication { .. } => vec![],
            #[cfg(test)]
            Self::TestNoOrganization => vec![],
            //
            Self::AuthLogout => vec![],
            //
            Self::OrganizationList => vec![],
            Self::OrganizationCreate => vec![],
            Self::OrganizationGet => vec![],
            Self::OrganizationEdit => vec![],
            Self::OrganizationInvite => vec![],
            Self::OrganizationRevoke => vec![],
            Self::OrganizationDelete => vec![],
            //
            Self::ServiceTokenList => vec![],
            Self::ServiceTokenCreate => vec![],
            Self::ServiceTokenEdit { service_token_id } => vec![fact!(
                "service_token_id({service_token_id})",
                service_token_id = *service_token_id
            )],
            Self::ServiceTokenDelete { service_token_id } => vec![fact!(
                "service_token_id({service_token_id})",
                service_token_id = *service_token_id
            )],
            //
            Self::ApplicationList => vec![],
            Self::ApplicationCreate => vec![],
            Self::ApplicationGet { .. } => vec![],
            Self::ApplicationEdit { .. } => vec![],
            Self::ApplicationDelete { .. } => vec![],
            //
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretList { .. } => vec![],
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretCreate { .. } => vec![],
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretEdit { .. } => vec![],
            #[cfg(feature = "application-secret-compatibility")]
            Self::ApplicationSecretDelete { .. } => vec![],
            //
            Self::EventTypeList { .. } => vec![],
            Self::EventTypeCreate { .. } => vec![],
            Self::EventTypeGet { .. } => vec![],
            Self::EventTypeDelete { .. } => vec![],
            //
            Self::SubscriptionList { .. } => vec![],
            Self::SubscriptionCreate {
                label_key,
                label_value,
                ..
            } => vec![
                fact!("label_key({label_key})", label_key = label_key),
                fact!("label_value({label_value})", label_value = label_value),
            ],
            Self::SubscriptionGet {
                subscription_id, ..
            } => vec![fact!(
                "subscription_id({subscription_id})",
                subscription_id = *subscription_id
            )],
            Self::SubscriptionEdit {
                subscription_id, ..
            } => vec![fact!(
                "subscription_id({subscription_id})",
                subscription_id = *subscription_id
            )],
            Self::SubscriptionDelete {
                subscription_id, ..
            } => vec![fact!(
                "subscription_id({subscription_id})",
                subscription_id = *subscription_id
            )],
            //
            Self::EventList { .. } => vec![],
            Self::EventGet { .. } => vec![],
            Self::EventIngest { .. } => vec![],
            //
            Self::RequestAttemptList { .. } => vec![],
            //
            Self::ResponseGet { .. } => vec![],
        };

        facts.push(fact!("action({action})", action = self.action_name()));
        if let Some(application_id) = self.application_id() {
            facts.push(fact!(
                "application_id({application_id})",
                application_id = application_id
            ));
        }

        for role in self.allowed_roles() {
            facts.push(fact!("allowed_role({role})", role = role.as_ref()));
        }

        facts
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizedToken {
    User(AuthorizedUserToken),
    EmailVerification(AuthorizedEmailVerificationToken),
    Service(AuthorizeServiceToken),
    Master,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedUserToken {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub organizations: Vec<(Uuid, Role)>,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizeServiceToken {
    pub organization_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedEmailVerificationToken {
    pub user_id: Uuid,
}

pub fn authorize(
    biscuit: &Biscuit,
    organization_id: Option<Uuid>,
    action: Action,
) -> Result<AuthorizedToken, biscuit_auth::error::Token> {
    let mut authorizer = authorizer!(
        r#"
            valid_types(["master_access", "service_access", "user_access"]);
            valid_type($t) <- type($t), valid_types($vt), $vt.contains($t);
            check if valid_type($t);

            supported_version("service_access", 1);
            supported_version("user_access", 1);
            valid_version($t, $v) <- type($t), version($v), supported_version($t, $v);
            valid_version("master_access", 0) <- type("master_access");
            check if valid_version($t, $v);

            expired($t) <- expired_at($exp), time($t), $exp < $t;
            deny if expired($t);
        "#
    );

    if let Some(organization_id) = organization_id {
        authorizer_merge!(
            &mut authorizer,
            r#"
                organization_id($id) <- type("user_access"), organization_role($id, $r), $id == {organization_id};
                organization_id({organization_id}) <- type("master_access");

                role($r) <- type("user_access"), organization_id($id), organization_role($id, $r);
                valid_role($r) <- role($r), allowed_role($r);
                valid_role("service") <- type("service_access");
                valid_role("master") <- type("master_access");
                check if valid_role($r);

                check if organization_id({organization_id});
            "#
        );
    } else {
        // If no organization ID was provided, we double-check that the action we used allows this
        // This is not supposed to happen, that is why we display a big error and fail if it does
        if !action.can_work_without_organization() {
            error!("Action '{}' cannot be used without providing an organization ID. This is probably a bug.", action.action_name());
            return Err(biscuit_auth::error::Token::InternalError);
        }
    }

    authorizer.set_time();
    for fact in action.generate_facts() {
        authorizer.add_fact(fact)?;
    }
    authorizer.add_allow_all();

    authorizer.set_limits(AuthorizerLimits {
        max_time: Duration::from_millis(5),
        ..Default::default()
    });
    authorizer.add_token(biscuit)?;
    let result = authorizer.authorize();
    trace!("Authorizer state:\n{}", authorizer.print_world());
    result?;

    let raw_type: Vec<(String,)> = authorizer.query(rule!("data($id) <- type($id)"))?;
    let token_type = raw_type
        .first()
        .map(|(str,)| str)
        .ok_or(biscuit_auth::error::Token::InternalError)?;

    match token_type.as_str() {
        "user_access" => {
            let raw_session_id: Vec<(Vec<u8>,)> =
                authorizer.query(rule!("data($id) <- session_id($id)"))?;
            let session_id = raw_session_id
                .first()
                .and_then(|(str,)| Uuid::from_slice(str).ok())
                .ok_or(biscuit_auth::error::Token::InternalError)?;
            let raw_user_id: Vec<(Vec<u8>,)> =
                authorizer.query(rule!("data($id) <- user_id($id)"))?;
            let user_id = raw_user_id
                .first()
                .and_then(|(str,)| Uuid::from_slice(str).ok())
                .ok_or(biscuit_auth::error::Token::InternalError)?;
            let raw_organizations: Vec<(Vec<u8>, String)> =
                authorizer.query(rule!("data($id, $role) <- organization_role($id, $role)"))?;
            let organizations = raw_organizations.into_iter().flat_map(|(id, role)| {
                let organization_id = Uuid::from_slice(id.as_slice()).ok();
                let organization_role = Role::from_str(&role).ok();

                match (organization_id, organization_role) {
                    (Some(i), Some(r)) => vec![(i, r)],
                    (i, r) => {
                        if i.is_none() {
                            warn!("Could not parse organization ID from Biscuit as UUID: '{id:x?}'");
                        }
                        if r.is_none() {
                            warn!("Could not parse role from Biscuit as Role: '{role}'");
                        }
                        vec![]
                    }
                }
            }).collect::<Vec<_>>();
            let raw_email: Vec<(String,)> = authorizer.query(rule!("data($str) <- email($str)"))?;
            let email = raw_email
                .first()
                .ok_or(biscuit_auth::error::Token::InternalError)?
                .0
                .to_owned();
            let raw_first_name: Vec<(String,)> =
                authorizer.query(rule!("data($str) <- first_name($str)"))?;
            let first_name = raw_first_name
                .first()
                .ok_or(biscuit_auth::error::Token::InternalError)?
                .0
                .to_owned();
            let raw_last_name: Vec<(String,)> =
                authorizer.query(rule!("data($str) <- last_name($str)"))?;
            let last_name = raw_last_name
                .first()
                .ok_or(biscuit_auth::error::Token::InternalError)?
                .0
                .to_owned();

            Ok(AuthorizedToken::User(AuthorizedUserToken {
                session_id,
                user_id,
                organizations,
                email,
                first_name,
                last_name,
            }))
        }
        "service_access" => {
            let raw_organization_id: Vec<(Vec<u8>,)> =
                authorizer.query(rule!("data($id) <- organization_id($id)"))?;
            let organization_id = raw_organization_id
                .first()
                .and_then(|(str,)| Uuid::from_slice(str).ok())
                .ok_or(biscuit_auth::error::Token::InternalError)?;

            Ok(AuthorizedToken::Service(AuthorizeServiceToken {
                organization_id,
            }))
        }
        "master_access" => Ok(AuthorizedToken::Master),
        _ => {
            error!("Invalid token type: {token_type}");
            Err(biscuit_auth::error::Token::InternalError)
        }
    }
}

pub fn authorize_only_user(
    biscuit: &Biscuit,
    organization_id: Option<Uuid>,
    action: Action,
) -> Result<AuthorizedUserToken, biscuit_auth::error::Token> {
    match authorize(biscuit, organization_id, action) {
        Ok(AuthorizedToken::User(aut)) => Ok(aut),
        Ok(_) => {
            trace!("Authorization was denied because a user_access token was required");
            Err(biscuit_auth::error::Token::InternalError)
        }
        Err(e) => Err(e),
    }
}

pub fn authorize_email_verification(
    biscuit: &Biscuit,
) -> Result<AuthorizedEmailVerificationToken, biscuit_auth::error::Token> {
    let mut authorizer = authorizer!(
        r#"
            supported_version("email_verification", 1);
            valid_version($t, $v) <- type($t), version($v), supported_version($t, $v);
            check if valid_version($t, $v);

            expired($t) <- expired_at($exp), time($t), $exp < $t;
            deny if expired($t);
        "#
    );
    authorizer.set_time();
    authorizer.add_allow_all();

    authorizer.set_limits(AuthorizerLimits {
        max_time: Duration::from_secs(1800),
        ..Default::default()
    });
    authorizer.add_token(biscuit)?;
    let result = authorizer.authorize();
    trace!("Authorizer state:\n{}", authorizer.print_world());
    result?;

    let raw_user_id: Vec<(Vec<u8>,)> = authorizer.query(rule!("data($id) <- user_id($id)"))?;
    let user_id = raw_user_id
        .first()
        .and_then(|(str,)| Uuid::from_slice(str).ok())
        .ok_or(biscuit_auth::error::Token::InternalError)?;

    Ok(AuthorizedEmailVerificationToken { user_id })
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedRefreshToken {
    pub token_id: Uuid,
    pub session_id: Uuid,
    pub user_id: Uuid,
}

pub fn authorize_refresh_token(
    biscuit: &Biscuit,
) -> Result<AuthorizedRefreshToken, biscuit_auth::error::Token> {
    let mut authorizer = authorizer!(
        r#"
            supported_version("refresh", 1);
            valid_version($t, $v) <- type($t), version($v), supported_version($t, $v);
            check if valid_version($t, $v);

            expired($t) <- expired_at($exp), time($t), $exp < $t;
            deny if expired($t);
        "#
    );
    authorizer.set_time();
    authorizer.add_allow_all();

    authorizer.set_limits(AuthorizerLimits {
        max_time: Duration::from_millis(5),
        ..Default::default()
    });
    authorizer.add_token(biscuit)?;
    let result = authorizer.authorize();
    trace!("Authorizer state:\n{}", authorizer.print_world());
    result?;

    let raw_token_id: Vec<(Vec<u8>,)> = authorizer.query(rule!("data($id) <- token_id($id)"))?;
    let token_id = raw_token_id
        .first()
        .and_then(|(str,)| Uuid::from_slice(str).ok())
        .ok_or(biscuit_auth::error::Token::InternalError)?;
    let raw_session_id: Vec<(Vec<u8>,)> =
        authorizer.query(rule!("data($id) <- session_id($id)"))?;
    let session_id = raw_session_id
        .first()
        .and_then(|(str,)| Uuid::from_slice(str).ok())
        .ok_or(biscuit_auth::error::Token::InternalError)?;
    let raw_user_id: Vec<(Vec<u8>,)> = authorizer.query(rule!("data($id) <- user_id($id)"))?;
    let user_id = raw_user_id
        .first()
        .and_then(|(str,)| Uuid::from_slice(str).ok())
        .ok_or(biscuit_auth::error::Token::InternalError)?;

    Ok(AuthorizedRefreshToken {
        token_id,
        session_id,
        user_id,
    })
}

pub async fn authorize_for_application<'a>(
    db: &PgPool,
    biscuit: &Biscuit,
    action: Action<'a>,
) -> Result<AuthorizedToken, String> {
    let application_id = action.application_id().ok_or_else(|| {
        let e = format!("The following action is not application-scoped (please report the issue, this is most likely a bug): {action:?}");
        trace!("{e}");
        e
    })?;

    get_owner_organization(db, &application_id)
        .await
        .ok_or_else(|| {
            format!(
                "Could not find owner organization of application {}",
                &application_id
            )
        })
        .and_then(|organization_id| {
            authorize(biscuit, Some(organization_id), action).map_err(|e| format!("{e:?}"))
        })
        .map_err(|e| {
            trace!("{e}");
            e
        })
}

#[cfg(test)]
mod tests {
    use biscuit_auth::builder::BlockBuilder;
    use biscuit_auth::builder_ext::BuilderExt;
    use biscuit_auth::macros::block;
    use std::time::{Duration, SystemTime};

    use super::*;

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn service_access_token_authorization_root_token() {
        let keypair = KeyPair::new();
        let token_id = Uuid::new_v4();
        let organization_id = Uuid::new_v4();
        let RootToken { biscuit, .. } =
            create_service_access_token(&keypair.private(), token_id, organization_id).unwrap();

        assert_eq!(
            dbg!(authorize(
                &biscuit,
                Some(organization_id),
                Action::TestSimple
            )),
            Ok(AuthorizedToken::Service(AuthorizeServiceToken {
                organization_id
            }))
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn service_access_token_authorization_expiration() {
        let keypair = KeyPair::new();
        let token_id = Uuid::new_v4();
        let organization_id = Uuid::new_v4();
        let RootToken { biscuit, .. } =
            create_service_access_token(&keypair.private(), token_id, organization_id).unwrap();

        let not_yet_expired_biscuit = biscuit
            .append({
                let mut block = BlockBuilder::new();
                block.check_expiration_date(SystemTime::now() + Duration::from_secs(1));
                block
            })
            .unwrap();
        let expired_biscuit = biscuit
            .append({
                let mut block = BlockBuilder::new();
                block.check_expiration_date(SystemTime::now() - Duration::from_secs(1));
                block
            })
            .unwrap();

        assert!(dbg!(authorize(
            &not_yet_expired_biscuit,
            Some(organization_id),
            Action::TestSimple,
        ))
        .is_ok());
        assert!(dbg!(authorize(
            &expired_biscuit,
            Some(organization_id),
            Action::TestSimple,
        ))
        .is_err());
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn service_access_token_authorization_wrong_organization() {
        let keypair = KeyPair::new();
        let token_id = Uuid::new_v4();
        let organization_id = Uuid::new_v4();
        let RootToken { biscuit, .. } =
            create_service_access_token(&keypair.private(), token_id, organization_id).unwrap();

        let other_organization_id = Uuid::new_v4();
        assert!(dbg!(authorize(
            &biscuit,
            Some(other_organization_id),
            Action::TestSimple,
        ))
        .is_err());
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn service_access_token_authorization_application() {
        let keypair = KeyPair::new();
        let token_id = Uuid::new_v4();
        let organization_id = Uuid::new_v4();
        let application_id = Uuid::new_v4();
        let RootToken { biscuit, .. } =
            create_service_access_token(&keypair.private(), token_id, organization_id).unwrap();

        let application_restricted_biscuit = biscuit
            .append(block!("check if application_id({application_id})"))
            .unwrap();
        let other_application_id = Uuid::new_v4();
        let wrong_application_restricted_biscuit = biscuit
            .append(block!("check if application_id({other_application_id})"))
            .unwrap();

        assert!(dbg!(authorize(
            &biscuit,
            Some(organization_id),
            Action::TestWithApplication {
                application_id: &application_id
            },
        ))
        .is_ok());
        assert!(dbg!(authorize(
            &application_restricted_biscuit,
            Some(organization_id),
            Action::TestWithApplication {
                application_id: &application_id
            },
        ))
        .is_ok());
        assert!(dbg!(authorize(
            &wrong_application_restricted_biscuit,
            Some(organization_id),
            Action::TestWithApplication {
                application_id: &application_id
            },
        ))
        .is_err());
        // Using an application-restricted Biscuit on an organization-only endpoint
        assert!(dbg!(authorize(
            &application_restricted_biscuit,
            Some(organization_id),
            Action::TestSimple,
        ))
        .is_err());
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn service_access_token_authorization_no_organization_id() {
        let keypair = KeyPair::new();
        let token_id = Uuid::new_v4();
        let organization_id = Uuid::new_v4();
        let RootToken { biscuit, .. } =
            create_service_access_token(&keypair.private(), token_id, organization_id).unwrap();

        assert!(dbg!(authorize(&biscuit, None, Action::TestSimple)).is_err());
        assert!(dbg!(authorize(&biscuit, None, Action::TestNoOrganization)).is_ok());
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn user_access_token_authorization_editor() {
        let keypair = KeyPair::new();
        let token_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let organization_id = Uuid::new_v4();
        let roles = vec![(organization_id, "editor".to_owned())];
        let RootToken { biscuit, .. } = create_user_access_token(
            &keypair.private(),
            token_id,
            session_id,
            user_id,
            "email",
            "first_name",
            "last_name",
            roles,
        )
        .unwrap();

        assert_eq!(
            dbg!(authorize(
                &biscuit,
                Some(organization_id),
                Action::TestSimple
            )),
            Ok(AuthorizedToken::User(AuthorizedUserToken {
                session_id,
                user_id,
                organizations: vec![(organization_id, Role::Editor)],
                email: "email".to_owned(),
                first_name: "first_name".to_owned(),
                last_name: "last_name".to_owned(),
            }))
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn user_access_token_authorization_roles() {
        let keypair = KeyPair::new();
        let token_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let organization_id1 = Uuid::new_v4();
        let organization_id2 = Uuid::new_v4();
        let organization_id3 = Uuid::new_v4();
        let roles = vec![
            (organization_id1, "editor".to_owned()),
            (organization_id2, "viewer".to_owned()),
        ];
        let RootToken { biscuit, .. } = create_user_access_token(
            &keypair.private(),
            token_id,
            session_id,
            user_id,
            "",
            "",
            "",
            roles,
        )
        .unwrap();

        assert!(dbg!(authorize(
            &biscuit,
            Some(organization_id1),
            Action::TestSimple
        ))
        .is_ok());
        assert!(dbg!(authorize(
            &biscuit,
            Some(organization_id2),
            Action::TestSimple
        ))
        .is_ok());
        assert!(dbg!(authorize(
            &biscuit,
            Some(organization_id3),
            Action::TestSimple
        ))
        .is_err());
        assert!(dbg!(authorize(
            &biscuit,
            Some(organization_id1),
            Action::TestWithApplication {
                application_id: &Uuid::new_v4()
            }
        ))
        .is_ok());
        assert!(dbg!(authorize(
            &biscuit,
            Some(organization_id2),
            Action::TestWithApplication {
                application_id: &Uuid::new_v4()
            }
        ))
        .is_err());
        assert!(dbg!(authorize(
            &biscuit,
            Some(organization_id3),
            Action::TestWithApplication {
                application_id: &Uuid::new_v4()
            }
        ))
        .is_err());
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn refresh_token_authorization() {
        let keypair = KeyPair::new();
        let token_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let RootToken { biscuit, .. } =
            create_refresh_token(&keypair.private(), token_id, session_id, user_id).unwrap();

        let rt = dbg!(authorize_refresh_token(&biscuit)).unwrap();

        assert_eq!(rt.token_id, token_id);
        assert_eq!(rt.session_id, session_id);
        assert_eq!(rt.user_id, user_id);
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn refreshh_token_authorization_expiration() {
        let keypair = KeyPair::new();
        let token_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let RootToken { biscuit, .. } =
            create_refresh_token(&keypair.private(), token_id, session_id, user_id).unwrap();

        let not_yet_expired_biscuit = biscuit
            .append({
                let mut block = BlockBuilder::new();
                block.check_expiration_date(SystemTime::now() + Duration::from_secs(1));
                block
            })
            .unwrap();
        let expired_biscuit = biscuit
            .append({
                let mut block = BlockBuilder::new();
                block.check_expiration_date(SystemTime::now() - Duration::from_secs(1));
                block
            })
            .unwrap();

        assert!(dbg!(authorize_refresh_token(&not_yet_expired_biscuit)).is_ok());
        assert!(dbg!(authorize_refresh_token(&expired_biscuit)).is_err());
    }
}
