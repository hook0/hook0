use log::{error, trace};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::problems::Hook0Problem;

#[derive(Debug, Clone)]
pub struct KeycloakApi {
    client: Client,
    api_url: Url,
    #[allow(dead_code)]
    access_token: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub enabled: bool,
    pub email_verified: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroupLookup {
    pub id: Uuid,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub sub_groups: Vec<Group>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMember {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct KcGroup<'a> {
    name: &'a str,
    path: &'a str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct KcUser<'a> {
    username: &'a str,
    email: &'a str,
    first_name: &'a str,
    last_name: &'a str,
    credentials: Vec<KcUserCredential<'a>>,
    enabled: bool,
    email_verified: bool,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct KcUserCredential<'a> {
    id: &'a str,
    value: &'a str,
    temporary: bool,
}

impl KeycloakApi {
    pub async fn new(
        keycloak_url: &Url,
        keycloak_realm: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<Self, Hook0Problem> {
        let url = append_url_segments(
            keycloak_url,
            &[
                "realms",
                keycloak_realm,
                "protocol",
                "openid-connect",
                "token",
            ],
        )
        .map_err(|e| {
            error!(
                "Could not create a valid URL to request Keycloak's API: {}",
                &e
            );
            Hook0Problem::InternalServerError
        })?;

        let body = [
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ];
        #[derive(Debug, Deserialize)]
        struct AuthResponse {
            access_token: String,
        }
        const OPERATION: &str = "getting an access token from Keycloak";

        trace!(
            "Requesting an access token to Keycloak (client_id = {})",
            client_id
        );

        let res = Client::new()
            .post(url)
            .form(&body)
            .send()
            .await
            .map_err(|e| {
                error!("Error while {}: {}", OPERATION, &e);
                Hook0Problem::InternalServerError
            })?
            .error_for_status()
            .map_err(|e| {
                error!("Error while {}: {}", OPERATION, &e);
                Hook0Problem::InternalServerError
            })?
            .json::<AuthResponse>()
            .await
            .map_err(|e| {
                error!("Error while {}: {}", OPERATION, &e);
                Hook0Problem::InternalServerError
            })?;

        let authenticated_client = HeaderValue::from_str(&format!("Bearer {}", &res.access_token))
            .map_err(|e| {
                error!("Could not build auth header: {}", &e);
                Hook0Problem::InternalServerError
            })
            .map(|hv| HeaderMap::from_iter([(AUTHORIZATION, hv)]))
            .and_then(|headers| {
                Client::builder()
                    .default_headers(headers)
                    .build()
                    .map_err(|e| {
                        error!("Could not build HTTP client: {}", &e);
                        Hook0Problem::InternalServerError
                    })
            })?;
        let api_url = append_url_segments(keycloak_url, &["admin", "realms", keycloak_realm])
            .map_err(|e| {
                error!(
                    "Could not create a valid URL to request Keycloak's API: {}",
                    &e
                );
                Hook0Problem::InternalServerError
            })?;

        Ok(Self {
            client: authenticated_client,
            api_url,
            access_token: res.access_token,
        })
    }

    fn mk_url(&self, segments: &[&str]) -> Result<Url, Hook0Problem> {
        append_url_segments(&self.api_url, segments).map_err(|e| {
            error!(
                "Could not create a valid URL to request Keycloak's API: {}",
                &e
            );
            Hook0Problem::InternalServerError
        })
    }

    pub async fn get_user_by_email(&self, user_email: &str) -> Result<Option<User>, Hook0Problem> {
        let operation = format!("getting user '{user_email}' from Keycloak");
        let user_url = self.mk_url(&["users"])?;

        let res = self
            .client
            .get(user_url.as_str())
            .query(&[("exact", "true"), ("email", user_email)])
            .send()
            .await
            .map_err(|e| {
                error!("Error while {}: {}", operation, &e);
                Hook0Problem::InternalServerError
            })?
            .error_for_status()
            .map_err(|e| {
                error!("Error while {}: {}", operation, &e);
                Hook0Problem::InternalServerError
            })?
            .json::<Vec<User>>()
            .await
            .map_err(|e| {
                error!("Error while {}: {}", operation, &e);
                Hook0Problem::InternalServerError
            })?;

        let user = res.iter().find(|u| u.email == user_email).cloned();
        Ok(user)
    }

    pub async fn get_user_groups(&self, user_id: &Uuid) -> Result<Vec<Group>, Hook0Problem> {
        let operation = format!("getting groups of user '{user_id}' from Keycloak");
        let group_url = self.mk_url(&["users", user_id.to_string().as_str(), "groups"])?;
        // let children_url = self.mk_url(&["users", user_id.to_string().as_str(), "children"])?;

        let groups = self
            .client
            .get(group_url.as_str())
            .send()
            .await
            .map_err(|e| {
                error!("Error while {}: {}", operation, &e);
                Hook0Problem::InternalServerError
            })?
            .error_for_status()
            .map_err(|e| {
                error!("Error while {}: {}", operation, &e);
                Hook0Problem::InternalServerError
            })?
            .json::<Vec<Group>>()
            .await
            .map_err(|e| {
                error!("Error while {}: {}", operation, &e);
                Hook0Problem::InternalServerError
            })?;

        Ok(groups)

        // At some point Keycloak stopped returning sub groups directly which requires doing a dedicated API call
        // if group.sub_groups.is_empty() {
        //     let children = self
        //         .client
        //         .get(children_url.as_str())
        //         .send()
        //         .await
        //         .map_err(|e| {
        //             error!("Error while {}: {}", operation, &e);
        //             Hook0Problem::InternalServerError
        //         })?
        //         .error_for_status()
        //         .map_err(|e| {
        //             error!("Error while {}: {}", operation, &e);
        //             Hook0Problem::InternalServerError
        //         })?
        //         .json::<Vec<Group>>()
        //         .await
        //         .map_err(|e| {
        //             error!("Error while {}: {}", operation, &e);
        //             Hook0Problem::InternalServerError
        //         })?;

        //     Ok(Group {
        //         sub_groups: children,
        //         ..group
        //     })
        // } else {
        //     Ok(group)
        // }
    }
}

pub fn append_url_segments(base_url: &Url, segments: &[&str]) -> Result<Url, url::ParseError> {
    const SEP: &str = "/";
    let segments_str = segments.join(SEP);

    let url = Url::parse(&format!("{base_url}/{segments_str}").replace("//", "/"))?;

    Ok(url)
}
