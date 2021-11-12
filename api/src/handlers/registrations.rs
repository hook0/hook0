use log::{debug, error, trace};
use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json},
    Apiv2Schema, CreatedJson,
};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{Client, Response, Url};
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool, Postgres, Transaction};
use strum::IntoEnumIterator;
use uuid::Uuid;

use crate::iam::{Role, GROUP_SEP, ORGA_GROUP_PREFIX, ROLE_GROUP_PREFIX};
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Registration {
    organization_id: Uuid,
    user_id: Uuid,
    temporary_password: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct RegistrationPost {
    organization_name: String,
    first_name: String,
    last_name: String,
    email: String,
}

#[api_v2_operation(
    summary = "Create a new user account and a new organization",
    description = "",
    operation_id = "register",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn register(
    state: Data<crate::State>,
    body: Json<RegistrationPost>,
) -> Result<CreatedJson<Registration>, Hook0Problem> {
    do_register(
        body.into_inner(),
        &state.db,
        &state.keycloak_url,
        &state.keycloak_realm,
        &state.keycloak_client_id,
        &state.keycloak_client_secret,
    )
    .await
    .map(CreatedJson)
}

async fn do_register(
    registration_req: RegistrationPost,
    db: &PgPool,
    keycloak_url: &Url,
    keycloak_realm: &str,
    keycloak_client_id: &str,
    keycloak_client_secret: &str,
) -> Result<Registration, Hook0Problem> {
    debug!("Starting registration for {}", &registration_req.email);

    check_organization_name(&registration_req.organization_name)?;

    let client = Client::new();

    let token = get_keycloak_access_token(
        &client,
        keycloak_url,
        keycloak_realm,
        keycloak_client_id,
        keycloak_client_secret,
    )
    .await?;
    let authenticated_client = HeaderValue::from_str(&format!("Bearer {}", &token))
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
    let api_url = keycloak_url
        .join(&format!(
            "{}/admin/realms/{}",
            keycloak_url.path(),
            keycloak_realm
        ))
        .map_err(|e| {
            error!(
                "Could not create a valid URL to request Keycloak's API: {}",
                &e
            );
            Hook0Problem::InternalServerError
        })?;

    check_user_email(&authenticated_client, &api_url, &registration_req.email).await?;

    // Let's start a transaction so DB operations can be rollback if something fails
    let mut tx = db.begin().await?;

    let organization_id =
        create_organization_in_db(&mut tx, &registration_req.organization_name).await?;

    let editor_group_id =
        create_organization_in_keycloak(&authenticated_client, &api_url, &organization_id).await?;

    let (user_id, temporary_password) = create_user_in_keycloak(
        &authenticated_client,
        &api_url,
        &editor_group_id,
        &registration_req.first_name,
        &registration_req.last_name,
        &registration_req.email,
    )
    .await?;

    tx.commit().await?;
    Ok(Registration {
        organization_id,
        user_id,
        temporary_password,
    })
}

fn check_organization_name(organization_name: &str) -> Result<(), Hook0Problem> {
    if organization_name.len() <= 1 {
        Err(Hook0Problem::OrganizationNameMissing)
    } else {
        Ok(())
    }
}

async fn get_keycloak_access_token(
    client: &Client,
    keycloak_url: &Url,
    keycloak_realm: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<String, Hook0Problem> {
    let url = keycloak_url
        .join(&format!(
            "{}/realms/{}/protocol/openid-connect/token",
            keycloak_url.path(),
            keycloak_realm
        ))
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

    let res = client
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

    Ok(res.access_token)
}

async fn check_user_email(client: &Client, api_url: &Url, email: &str) -> Result<(), Hook0Problem> {
    let url = api_url
        .join(&format!("{}/users/count", api_url.path()))
        .map_err(|e| {
            error!(
                "Could not create a valid URL to request Keycloak's API: {}",
                &e
            );
            Hook0Problem::InternalServerError
        })?;

    let query = [("email", email)];
    const OPERATION: &str = "checking if user already exists in Keycloak";

    trace!("Checking if a Keycloak user with this email address already exists");

    let res = client
        .get(url)
        .query(&query)
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
        .text()
        .await
        .map_err(|e| {
            error!("Error while {}: {}", OPERATION, &e);
            Hook0Problem::InternalServerError
        })?;

    let count = res.parse::<i32>();

    match count {
        Ok(c) if c == 0 => Ok(()),
        Ok(c) if c < 0 => {
            error!(
                "Error while parsing result given by Keycloak API: search found {} users",
                &c
            );
            Err(Hook0Problem::InternalServerError)
        }
        Ok(_) => Err(Hook0Problem::UserAlreadyExist),
        Err(e) => {
            error!("Error while parsing result given by Keycloak API: {}", &e);
            Err(Hook0Problem::InternalServerError)
        }
    }
}

async fn create_organization_in_db(
    tx: &mut Transaction<'_, Postgres>,
    name: &str,
) -> Result<Uuid, Hook0Problem> {
    let organization_id = Uuid::new_v4();
    query!(
        "
            INSERT INTO event.organization (organization__id, name)
            VALUES ($1, $2)
        ",
        &organization_id,
        name
    )
    .execute(tx)
    .await
    .map_err(|e| {
        error!("Error while creating organization in DB: {}", &e);
        Hook0Problem::InternalServerError
    })?;

    Ok(organization_id)
}

fn extract_resource_id_from_redirection(res: &Response) -> Result<Uuid, Hook0Problem> {
    res.headers()
        .get("location")
        .ok_or_else(|| {
            error!("Could not find the Location header in the response");
            Hook0Problem::InternalServerError
        })
        .and_then(|hv| {
            hv.to_str().map_err(|e| {
                error!("Could not convert response header to string: {}", &e);
                Hook0Problem::InternalServerError
            })
        })
        .and_then(|location| {
            Url::parse(location).map_err(|e| {
                error!("Could not parse Location header as a URL: {}", &e);
                Hook0Problem::InternalServerError
            })
        })
        .and_then(|url| {
            url.path_segments()
                .ok_or_else(|| {
                    error!("Could not split Location header's URL into segments",);
                    Hook0Problem::InternalServerError
                })
                .and_then(|segments| {
                    segments.last().map(|s| s.to_owned()).ok_or_else(|| {
                        error!("Could not get last segment of Location header's URL");
                        Hook0Problem::InternalServerError
                    })
                })
        })
        .and_then(|last| {
            Uuid::parse_str(&last).map_err(|e| {
                error!(
                    "Could not parse last segment of Location header's URL into UUID: {}",
                    &e
                );
                Hook0Problem::InternalServerError
            })
        })
}

async fn create_organization_in_keycloak(
    client: &Client,
    api_url: &Url,
    organization_id: &Uuid,
) -> Result<Uuid, Hook0Problem> {
    let group_url = api_url
        .join(&format!("{}/groups", api_url.path()))
        .map_err(|e| {
            error!(
                "Could not create a valid URL to request Keycloak's API: {}",
                &e
            );
            Hook0Problem::InternalServerError
        })?;

    #[derive(Debug, Serialize)]
    struct Group {
        name: String,
        path: String,
    }
    let main_group = Group {
        name: format!("{}{}", ORGA_GROUP_PREFIX, &organization_id),
        path: format!("{}{}{}", GROUP_SEP, ORGA_GROUP_PREFIX, &organization_id),
    };
    const OPERATION: &str = "creating organization groups in Keycloak";

    trace!("Creating organization groups in Keycloak");

    let res = client
        .post(group_url.as_str())
        .json(&main_group)
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
        })?;

    let main_group_id = extract_resource_id_from_redirection(&res)?;

    let sub_group_url = group_url
        .join(&format!("{}/{}/children", group_url.path(), &main_group_id))
        .map_err(|e| {
            error!(
                "Could not create a valid URL to request Keycloak's API: {}",
                &e
            );
            Hook0Problem::InternalServerError
        })?;

    trace!("Main group created (ID={})", &main_group_id);

    let mut editor_group_id = None;
    for role in Role::iter() {
        let body = Group {
            name: format!("{}{}", ROLE_GROUP_PREFIX, &role),
            path: format!(
                "{}{}{}{}{}{}",
                GROUP_SEP, ORGA_GROUP_PREFIX, &organization_id, GROUP_SEP, ROLE_GROUP_PREFIX, &role
            ),
        };

        let res = client
            .post(sub_group_url.as_str())
            .json(&body)
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
            })?;

        if role == Role::Editor {
            editor_group_id = Some(extract_resource_id_from_redirection(&res)?);
        }
    }
    let editor_group_id = editor_group_id.ok_or_else(|| {
        error!("Could not get ID of the newly created editor group");
        Hook0Problem::InternalServerError
    })?;

    Ok(editor_group_id)
}

fn gen_password() -> String {
    use rand::Rng;

    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(15)
        .map(|byte| byte as char)
        .collect()
}

async fn create_user_in_keycloak(
    client: &Client,
    api_url: &Url,
    group_id: &Uuid,
    first_name: &str,
    last_name: &str,
    email: &str,
) -> Result<(Uuid, String), Hook0Problem> {
    let user_url = api_url
        .join(&format!("{}/users", api_url.path()))
        .map_err(|e| {
            error!(
                "Could not create a valid URL to request Keycloak's API: {}",
                &e
            );
            Hook0Problem::InternalServerError
        })?;

    let password = gen_password();

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct User<'a> {
        username: &'a str,
        email: &'a str,
        first_name: &'a str,
        last_name: &'a str,
        credentials: Vec<Credential<'a>>,
        enabled: bool,
        email_verified: bool,
    }
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Credential<'a> {
        id: &'a str,
        value: &'a str,
        temporary: bool,
    }
    let user = User {
        username: email,
        email,
        first_name,
        last_name,
        credentials: vec![Credential {
            id: "password",
            value: &password,
            temporary: true,
        }],
        enabled: true,
        email_verified: false, // TODO: send a welcome email and set this to true
    };
    const OPERATION: &str = "creating user in Keycloak";

    trace!("Creating user in Keycloak");

    let res = client
        .post(user_url.as_str())
        .json(&user)
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
        })?;
    let user_id = extract_resource_id_from_redirection(&res)?;

    let user_group_url = api_url
        .join(&format!(
            "{}/users/{}/groups/{}",
            api_url.path(),
            &user_id,
            &group_id
        ))
        .map_err(|e| {
            error!(
                "Could not create a valid URL to request Keycloak's API: {}",
                &e
            );
            Hook0Problem::InternalServerError
        })?;

    client
        .put(user_group_url.as_str())
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
        })?;

    Ok((user_id, password))
}
