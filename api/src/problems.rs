use actix_web::error::JsonPayloadError;
use actix_web::http::{StatusCode, header};
use actix_web::{HttpResponse, ResponseError};
use http_api_problem::{HttpApiProblem, PROBLEM_JSON_MEDIA_TYPE};
use paperclip::actix::api_v2_errors;
use serde_json::{Value, to_value};
use sqlx::Error;
use std::borrow::Cow;
use std::fmt::Display;
use strum::{EnumIter, VariantNames};
use tracing::{error, warn};

use crate::handlers::events::PayloadContentType;
use crate::iam::Role;
use crate::quotas::QuotaValue;
use crate::validators::CODE_SECRET_PREFIX;

/**
 * How to implement a new type error for Hook0:
 * 1/ Add the type error variant inside Hook0Problem enum
 * 2/ Implement the Problem inside From<Hook0Problem> for Problem
 * 3/ Done! Enjoy!
 */
#[api_v2_errors(code = 403, code = 500, code = 400, code = 404, code = 409, code = 503)]
#[derive(Debug, Clone, EnumIter, strum::Display)]
pub enum Hook0Problem {
    // Functional errors
    OrganizationNameMissing,
    UserAlreadyExist,
    RegistrationDisabled,
    PasswordTooShort(u8),
    PasswordTooLong,
    PasswordSimilarToEmail,
    PasswordSimilarToName,
    PasswordTooCommon,
    PasswordNotDiverseEnough,
    OrganizationIsNotEmpty,
    InvitedUserDoesNotExist,
    InvitedUserAlreadyInOrganization,

    ApplicationNameMissing,

    InvalidRole,

    EventTypeAlreadyExist,
    EventTypeDoesNotExist,

    UnauthorizedWorkers(Vec<String>),

    EventAlreadyIngested,
    EventInvalidPayloadContentType,
    EventInvalidBase64Payload(String),
    EventInvalidJsonPayload(String),

    LabelsAmbiguity,

    InvalidDateRange,

    // Auth errors
    AuthNoAuthorizationHeader,
    AuthInvalidAuthorizationHeader,
    AuthApplicationSecretLookupError,
    AuthInvalidApplicationSecret,
    AuthBiscuitLookupError,
    AuthInvalidBiscuit,
    AuthFailedLogin,
    AuthEmailNotVerified,
    AuthFailedRefresh,
    AuthEmailExpired,

    // Quota errors
    TooManyMembersPerOrganization(QuotaValue),
    TooManyApplicationsPerOrganization(QuotaValue),
    TooManyEventsToday(QuotaValue),
    TooManySubscriptionsPerApplication(QuotaValue),
    TooManyEventTypesPerApplication(QuotaValue),

    // Generic errors
    JsonPayload(JsonPayloadProblem),
    Validation(validator::ValidationErrors),
    NotFound,
    InternalServerError,
    Forbidden,
    ServiceUnavailable,
}

impl From<sqlx::Error> for Hook0Problem {
    fn from(e: Error) -> Self {
        match e {
            Error::RowNotFound => Hook0Problem::NotFound,
            Error::Database(ex) => {
                let code = ex.code();
                match code.as_deref() {
                    // 55P03 (lock_not_available): `lock_timeout` fired while waiting for a row
                    // lock. It carries no constraint name, so it has to be matched before the
                    // constraint table below, which would otherwise log transient contention as
                    // if it were a bug and answer 500 instead of a retryable 503.
                    Some("55P03") => {
                        warn!("Database lock timeout (likely quota-enforcement contention): {ex}");
                        Hook0Problem::ServiceUnavailable
                    }
                    _ => match ex.constraint() {
                        Some("application_name_chk") => Hook0Problem::ApplicationNameMissing,
                        Some("event_type_pkey") => Hook0Problem::EventTypeAlreadyExist,
                        Some("event_pkey") => Hook0Problem::EventAlreadyIngested,
                        Some(
                            "subscription__event_type_event_type__name_fkey"
                            | "event_event_type__name_fkey",
                        ) => Hook0Problem::EventTypeDoesNotExist,
                        Some("user__organization_pkey") => {
                            Hook0Problem::InvitedUserAlreadyInOrganization
                        }
                        constraint => {
                            error!(
                                "Database error (failed constraint = {}): {}",
                                constraint.unwrap_or("?"),
                                &ex
                            );
                            Hook0Problem::InternalServerError
                        }
                    },
                }
            }
            Error::PoolTimedOut => {
                warn!("Database connection pool timed out (likely saturation under load)");
                Hook0Problem::ServiceUnavailable
            }
            err => {
                error!("{}", &err);
                Hook0Problem::InternalServerError
            }
        }
    }
}

impl From<lettre::error::Error> for Hook0Problem {
    fn from(err: lettre::error::Error) -> Hook0Problem {
        warn!("{err}");
        Hook0Problem::InternalServerError
    }
}

impl From<lettre::transport::smtp::Error> for Hook0Problem {
    fn from(err: lettre::transport::smtp::Error) -> Hook0Problem {
        warn!("{err}");
        Hook0Problem::InternalServerError
    }
}

impl From<mrml::prelude::parser::Error> for Hook0Problem {
    fn from(err: mrml::prelude::parser::Error) -> Hook0Problem {
        warn!("{err}");
        Hook0Problem::InternalServerError
    }
}

impl From<mrml::prelude::render::Error> for Hook0Problem {
    fn from(err: mrml::prelude::render::Error) -> Hook0Problem {
        warn!("{err}");
        Hook0Problem::InternalServerError
    }
}

impl From<html2text::Error> for Hook0Problem {
    fn from(err: html2text::Error) -> Hook0Problem {
        warn!("{err}");
        Hook0Problem::InternalServerError
    }
}

/// actix-web 4 is pinned to `http` 0.2 while `http-api-problem` 0.60+ uses `http` 1.x, so their
/// `StatusCode` types are distinct and must be converted through their numeric value.
fn to_problem_status(status: StatusCode) -> http_api_problem::StatusCode {
    http_api_problem::StatusCode::from_u16(status.as_u16())
        .unwrap_or(http_api_problem::StatusCode::INTERNAL_SERVER_ERROR)
}

impl From<Hook0Problem> for HttpApiProblem {
    fn from(hook0_problem: Hook0Problem) -> Self {
        let problem: Problem = hook0_problem.to_owned().into();
        HttpApiProblem::new(to_problem_status(problem.status))
            .type_url(format!(
                "https://hook0.com/documentation/errors/{hook0_problem}",
            )) // rely on Display trait of Hook0Problem
            .value("id".to_owned(), &hook0_problem.to_string()) // also rely on Display trait of Hook0Problem
            .value("validation".to_owned(), &problem.validation)
            .title(problem.title)
            .detail(problem.detail)
    }
}

impl ResponseError for Hook0Problem {
    fn status_code(&self) -> StatusCode {
        let problem: Problem = self.to_owned().into();
        problem.status
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let problem: HttpApiProblem = self.to_owned().into();

        let mut builder = HttpResponse::build(status);
        builder.append_header((header::CONTENT_TYPE, PROBLEM_JSON_MEDIA_TYPE));
        if status == StatusCode::SERVICE_UNAVAILABLE {
            // Tell well-behaved clients (e.g. Business Central, SDKs) to back off
            // and retry rather than treating this transient saturation as permanent.
            builder.append_header((header::RETRY_AFTER, "5"));
        }
        builder.body(problem.json_bytes())
    }
}

#[derive(Debug, Clone)]
pub struct Problem {
    pub id: Hook0Problem,
    pub title: &'static str,
    pub detail: Cow<'static, str>,
    pub validation: Option<Value>,
    pub status: StatusCode,
}

impl From<Hook0Problem> for Problem {
    fn from(hook0_problem: Hook0Problem) -> Self {
        match hook0_problem {
            // Functional errors
            Hook0Problem::OrganizationNameMissing => Problem {
                id: Hook0Problem::OrganizationNameMissing,
                title: "Organization name cannot be empty",
                detail: "Organization name length must have more than 1 character.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::UserAlreadyExist => Problem {
                id: Hook0Problem::UserAlreadyExist,
                title: "This user already exist",
                detail: "This email is already registered.".into(),
                validation: None,
                status: StatusCode::CONFLICT,
            },
            Hook0Problem::RegistrationDisabled => Problem {
                id: Hook0Problem::RegistrationDisabled,
                title: "Registrations are disabled",
                detail: "Registration was disabled by an administrator.".into(),
                validation: None,
                status: StatusCode::GONE,
            },
            Hook0Problem::PasswordTooShort(minimum_length) => {
                let detail = format!("Password must be at least {minimum_length} characters long.");
                Problem {
                    id: Hook0Problem::PasswordTooShort(minimum_length),
                    title: "Provided password is too short",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::BAD_REQUEST,
                }
            },
            Hook0Problem::PasswordTooLong => Problem {
                id: Hook0Problem::PasswordTooLong,
                title: "Provided password is too long",
                detail: format!(
                    "Password must be at most {} characters long.",
                    crate::password::MAXIMUM_LENGTH
                )
                .into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::PasswordSimilarToEmail => Problem {
                id: Hook0Problem::PasswordSimilarToEmail,
                title: "Provided password is too close to the email address",
                detail: "Password must not be built from the email address of the account: anyone who knows the address would guess it. Please pick something unrelated.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::PasswordSimilarToName => Problem {
                id: Hook0Problem::PasswordSimilarToName,
                title: "Provided password is too close to the user name",
                detail: "Password must not be built from the first or last name of the account. Please pick something unrelated.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::PasswordTooCommon => Problem {
                id: Hook0Problem::PasswordTooCommon,
                title: "Provided password is too common",
                detail: "This password (or a lightly disguised version of it) is among the most frequently used ones, so it is one of the first an attacker tries. Please pick another one.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::PasswordNotDiverseEnough => Problem {
                id: Hook0Problem::PasswordNotDiverseEnough,
                title: "Provided password is not diverse enough",
                detail: "Password is made of too few different characters, which makes it easy to guess despite its length. Please pick another one.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::OrganizationIsNotEmpty => Problem {
                id: Hook0Problem::OrganizationIsNotEmpty,
                title: "Organization is not empty",
                detail: "Organizations that contain at least an application cannot be deleted; applications must be deleted first. If you believe this is a mistake, please contact the Hook0 support team.".into(),
                validation: None,
                status: StatusCode::CONFLICT,
            },
            Hook0Problem::InvitedUserDoesNotExist => Problem {
                id: Hook0Problem::InvitedUserDoesNotExist,
                title: "Invited user does not exist",
                detail: "The user you are trying to invite does not exist. Please make sure the user is already register in Hook0.".into(),
                validation: None,
                status: StatusCode::NOT_FOUND,
            },
            Hook0Problem::InvitedUserAlreadyInOrganization => Problem {
                id: Hook0Problem::InvitedUserAlreadyInOrganization,
                title: "Invited user is already in the organization",
                detail: "The user you are trying to invite has already access to the organization.".into(),
                validation: None,
                status: StatusCode::CONFLICT,
            },

            Hook0Problem::ApplicationNameMissing => Problem {
                id: Hook0Problem::ApplicationNameMissing,
                title: "Application name cannot be empty",
                detail: "Application name length must have more than 1 character.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },

            Hook0Problem::InvalidRole => {
                let roles = format!("Valid roles are: {}.", Role::VARIANTS.join(", "));
                Problem {
                    id: Hook0Problem::InvalidRole,
                    title: "Provided role does not exist",
                    detail: roles.into(),
                    validation: None,
                    status: StatusCode::BAD_REQUEST,
                }
            },

            Hook0Problem::EventTypeAlreadyExist => Problem {
                id: Hook0Problem::EventTypeAlreadyExist,
                title: "This event type already exist",
                detail: "An event type with this name is already present.".into(),
                validation: None,
                status: StatusCode::CONFLICT,
            },
            Hook0Problem::EventTypeDoesNotExist => Problem {
                id: Hook0Problem::EventTypeDoesNotExist,
                title: "Invalid event type",
                detail: "Event type does not exist or was deactivated. You should (re)create it.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },

            Hook0Problem::UnauthorizedWorkers(w) => {
                let detail = format!("You do not have access to the following workers: {}", w.join(", "));
                Problem {
                    id: Hook0Problem::UnauthorizedWorkers(w),
                    title: "Some of the provided dedicated workers are not authorized for your organization",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::BAD_REQUEST,
                }
            },

            Hook0Problem::EventAlreadyIngested => Problem {
                id: Hook0Problem::EventAlreadyIngested,
                title: "Event already Ingested",
                detail: "This event was previously ingested and recorded inside Hook0 service.".into(),
                validation: None,
                status: StatusCode::CONFLICT,
            },
            Hook0Problem::EventInvalidPayloadContentType => {
                let detail = format!("The specified event payload content type is not handled. Valid content types are: {}", PayloadContentType::VARIANTS.join(", "));
                Problem {
                    id: Hook0Problem::EventInvalidPayloadContentType,
                    title: "Invalid event payload content type",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::BAD_REQUEST,
                }
            },
            Hook0Problem::EventInvalidBase64Payload(e) => {
                let detail = format!("Event payload is not encoded in valid base64 format: {e}");
                Problem {
                    id: Hook0Problem::EventInvalidBase64Payload(e),
                    title: "Invalid event base64 payload",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::BAD_REQUEST,
                }
            },
            Hook0Problem::EventInvalidJsonPayload(e) => {
                let detail = format!("Event payload is not encoded in valid JSON format: {e}.");
                Problem {
                    id: Hook0Problem::EventInvalidJsonPayload(e),
                    title: "Invalid event JSON payload",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::BAD_REQUEST,
                }
            },
            Hook0Problem::LabelsAmbiguity => Problem {
                id: Hook0Problem::LabelsAmbiguity,
                title: "Ambiguous labels specification",
                detail: "You must specify either the `labels` property as an object with a least one property (recommended) or separated `label_key` and `label_value` properties as strings (legacy), but not both.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },

            Hook0Problem::InvalidDateRange => Problem {
                id: Hook0Problem::InvalidDateRange,
                title: "Invalid date range",
                detail: "'from' date must not be after 'to' date.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },

            // Auth error
            Hook0Problem::AuthNoAuthorizationHeader => Problem {
                id: Hook0Problem::AuthNoAuthorizationHeader,
                title: "No `Authorization` header was found in the HTTP request",
                detail: "`Authorization` header must be provided and must contain a bearer token.".into(),
                validation: None,
                status: StatusCode::UNAUTHORIZED,
            },
            Hook0Problem::AuthInvalidAuthorizationHeader => Problem {
                id: Hook0Problem::AuthInvalidAuthorizationHeader,
                title: "`Authorization` header is invalid",
                detail: "`Authorization` header value could not be decoded as a valid UTF-8 string containing `Bearer {UUID}`.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::AuthApplicationSecretLookupError => Problem {
                id: Hook0Problem::AuthApplicationSecretLookupError,
                title: "Could not check database to verify the provided application secret",
                detail: "This is likely to be caused by database unavailability.".into(),
                validation: None,
                status: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Hook0Problem::AuthInvalidApplicationSecret => Problem {
                id: Hook0Problem::AuthInvalidApplicationSecret,
                title: "Invalid application secret",
                detail: "The provided application secret does not exist.".into(),
                validation: None,
                status: StatusCode::FORBIDDEN,
            },
            Hook0Problem::AuthBiscuitLookupError => Problem {
                id: Hook0Problem::AuthBiscuitLookupError,
                title: "Could not check database to verify if the provided Biscuit was revoked",
                detail: "This is likely to be caused by database unavailability.".into(),
                validation: None,
                status: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Hook0Problem::AuthInvalidBiscuit => Problem {
                id: Hook0Problem::AuthInvalidBiscuit,
                title: "Invalid Biscuit",
                detail: "The provided authentication token (Biscuit) is not valid, was not created using the current private key or is expired.".into(),
                validation: None,
                status: StatusCode::FORBIDDEN,
            },
            Hook0Problem::AuthFailedLogin => Problem {
                id: Hook0Problem::AuthFailedLogin,
                title: "Login failed",
                detail: "The provided credentials do not match ones of a valid user.".into(),
                validation: None,
                status: StatusCode::UNAUTHORIZED,
            },
            Hook0Problem::AuthEmailNotVerified => Problem {
                id: Hook0Problem::AuthEmailNotVerified,
                title: "Email not verified",
                detail: "Your email has not been verified yet. Please check your inbox.".into(),
                validation: None,
                status: StatusCode::UNAUTHORIZED,
            },
            Hook0Problem::AuthFailedRefresh => Problem {
                id: Hook0Problem::AuthFailedRefresh,
                title: "Refreshing access token failed",
                detail: "The provided refresh token is probably invalid or expired.".into(),
                validation: None,
                status: StatusCode::UNAUTHORIZED,
            },
            Hook0Problem::AuthEmailExpired => {
                Problem {
                    id: Hook0Problem::AuthEmailExpired,
                    title: "Could not verify your link",
                    detail: "The link you clicked might be expired. Please retry the whole process or contact support.".into(),
                    validation: None,
                    status: StatusCode::UNAUTHORIZED,
                }
            },

            // Quota errors
            Hook0Problem::TooManyMembersPerOrganization(limit) => {
                let detail = format!("This organization cannot have more than {limit} users. You might want to upgrade to a better plan.");
                Problem {
                    id: Hook0Problem::TooManyMembersPerOrganization(limit),
                    title: "Exceeded number of users that can be invited in this organization",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::TOO_MANY_REQUESTS,
                }
            },
            Hook0Problem::TooManyApplicationsPerOrganization(limit) => {
                let detail = format!("This organization cannot have more than {limit} applications. You might want to upgrade to a better plan.");
                Problem {
                    id: Hook0Problem::TooManyApplicationsPerOrganization(limit),
                    title: "Exceeded number of applications that can be created in this organization",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::TOO_MANY_REQUESTS,
                }
            },
            Hook0Problem::TooManyEventsToday(limit) => {
                let detail = format!("This organization cannot ingest more than {limit} events per day. You might want to upgrade to a better plan.");
                Problem {
                    id: Hook0Problem::TooManyEventsToday(limit),
                    title: "Exceeded number of events that can be ingested in this organization today",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::TOO_MANY_REQUESTS,
                }
            },
            Hook0Problem::TooManySubscriptionsPerApplication(limit) => {
                let detail = format!("This application cannot have more than {limit} subscriptions. You might want to upgrade to a better plan.");
                Problem {
                    id: Hook0Problem::TooManySubscriptionsPerApplication(limit),
                    title: "Exceeded number of subscriptions that can be created in this application",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::TOO_MANY_REQUESTS,
                }
            },
            Hook0Problem::TooManyEventTypesPerApplication(limit) => {
                let detail = format!("This application cannot have more than {limit} event types. You might want to upgrade to a better plan.");
                Problem {
                    id: Hook0Problem::TooManyEventTypesPerApplication(limit),
                    title: "Exceeded number of event types that can be created in this application",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::TOO_MANY_REQUESTS,
                }
            },

            // Generic errors
            Hook0Problem::JsonPayload(e) => {
                let error_str = e.to_string();
                Problem {
                    id: Hook0Problem::JsonPayload(e),
                    title: "Provided body could not be decoded as JSON",
                    detail: error_str.into(),
                    validation: None,
                    status: StatusCode::BAD_REQUEST,
                }
            },
            Hook0Problem::Validation(e) => {
                /// The `validator` derive attaches the value it refused to
                /// every error it builds, and the whole tree ends up in the
                /// response body. For a password that means handing it back to
                /// the caller, to the browser console, and to anything logging
                /// response bodies — so errors a validator marked as being
                /// about a secret lose that value on the way out.
                fn without_secret_values(mut value: Value) -> Value {
                    match &mut value {
                        Value::Array(items) => {
                            for item in items {
                                *item = without_secret_values(item.take());
                            }
                        },
                        Value::Object(fields) => {
                            let is_secret = fields
                                .get("code")
                                .and_then(Value::as_str)
                                .is_some_and(|code| code.starts_with(CODE_SECRET_PREFIX));
                            if let Some(Value::Object(params)) = fields.get_mut("params")
                                && is_secret
                            {
                                params.remove("value");
                            }
                            for (_, field) in fields.iter_mut() {
                                *field = without_secret_values(field.take());
                            }
                        },
                        _ => {},
                    }
                    value
                }

                let errors_str = e.to_string();
                // `ValidationErrors` renders as an empty string when it holds no error, which only
                // happens for the value `EnumIter` fabricates to build the public error catalogue.
                let detail = if errors_str.is_empty() {
                    "Provided input did not pass validation.".to_owned()
                } else {
                    errors_str
                };
                Problem {
                    id: Hook0Problem::Validation(e.to_owned()),
                    title: "Provided input is malformed",
                    detail: detail.into(),
                    validation: to_value(e).ok().map(without_secret_values),
                    status: StatusCode::UNPROCESSABLE_ENTITY,
                }
            },
            Hook0Problem::InternalServerError => Problem {
                id: Hook0Problem::InternalServerError,
                title: "Something wrong happened",
                detail: "Hook0 server had issue handling your request. Our team was notified.".into(),
                validation: None,
                status: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Hook0Problem::NotFound => Problem {
                id: Hook0Problem::NotFound,
                title: "Item not found",
                detail: "Could not find the item. Check the identifier or that you have the right to access it.".into(),
                validation: None,
                status: StatusCode::NOT_FOUND,
            },
            Hook0Problem::Forbidden => Problem {
                id: Hook0Problem::Forbidden,
                title: "Insufficient rights",
                detail: "You don't have the right to access or edit this resource.".into(),
                validation: None,
                status: StatusCode::FORBIDDEN,
            },
            Hook0Problem::ServiceUnavailable => Problem {
                id: Hook0Problem::ServiceUnavailable,
                title: "Service temporarily unavailable",
                detail: "Hook0 is under heavy load and could not authorize your request in time. This is a temporary, server-side condition, not a rights issue: the request is safe to retry. Wait a moment and resubmit, honoring the Retry-After response header.".into(),
                validation: None,
                status: StatusCode::SERVICE_UNAVAILABLE,
            },
        }
    }
}

/// Simplified error type for the JSON body parser
#[derive(Debug, Clone)]
pub enum JsonPayloadProblem {
    Overflow { limit: usize },
    ContentType,
    Deserialize(String),
    Serialize(String),
    Payload(String),
    Other(String),
}

impl Default for JsonPayloadProblem {
    fn default() -> Self {
        Self::Other("Unknown JSON payload error".to_owned())
    }
}

impl Display for JsonPayloadProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Overflow { limit } => write!(f, "Body is too big (maximum is {limit} bytes)"),
            Self::ContentType => {
                write!(f, "Content-Type header should be set to 'application/json'")
            }
            Self::Deserialize(e) => write!(f, "JSON deserialization error: {e}"),
            Self::Serialize(e) => write!(f, "JSON serialization error: {e}"),
            Self::Payload(e) => write!(f, "Payload error: {e}"),
            Self::Other(e) => write!(f, "{e}"),
        }
    }
}

impl From<JsonPayloadError> for JsonPayloadProblem {
    fn from(e: JsonPayloadError) -> Self {
        match e {
            JsonPayloadError::OverflowKnownLength { length: _, limit } => Self::Overflow { limit },
            JsonPayloadError::Overflow { limit } => Self::Overflow { limit },
            JsonPayloadError::ContentType => Self::ContentType,
            JsonPayloadError::Deserialize(e) => Self::Deserialize(e.to_string()),
            JsonPayloadError::Serialize(e) => Self::Serialize(e.to_string()),
            JsonPayloadError::Payload(e) => Self::Payload(e.to_string()),
            e => Self::Other(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::body::to_bytes;
    use strum::IntoEnumIterator;

    /// Check the response contract of every error Hook0 can return.
    #[actix_web::test]
    async fn every_problem_response_matches_its_contract() {
        for hook0_problem in Hook0Problem::iter() {
            let expected_status = Problem::from(hook0_problem.to_owned()).status;
            // Only 503 tells well-behaved clients to back off; see `error_response`.
            let expected_retry_after = if expected_status == StatusCode::SERVICE_UNAVAILABLE {
                Some("5")
            } else {
                None
            };

            let response = hook0_problem.error_response();

            assert_eq!(
                response.status(),
                expected_status,
                "unexpected HTTP status for {hook0_problem}"
            );
            assert_eq!(
                response
                    .headers()
                    .get(header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok()),
                Some(PROBLEM_JSON_MEDIA_TYPE),
                "unexpected content type for {hook0_problem}"
            );
            assert_eq!(
                response
                    .headers()
                    .get(header::RETRY_AFTER)
                    .and_then(|value| value.to_str().ok()),
                expected_retry_after,
                "unexpected `Retry-After` header for {hook0_problem}"
            );

            let body = to_bytes(response.into_body())
                .await
                .expect("error response body should be readable");
            let problem = serde_json::from_slice::<Value>(&body)
                .expect("error response body should be valid JSON");

            // `status` travels through `to_problem_status` while the response status does not,
            // so this catches the `http` 0.2 / `http` 1.x bridge falling back to 500.
            assert_eq!(
                problem["status"].as_u64(),
                Some(u64::from(expected_status.as_u16())),
                "serialized status does not match response status for {hook0_problem}"
            );
            assert!(
                problem["title"].as_str().is_some_and(|t| !t.is_empty()),
                "missing title for {hook0_problem}"
            );
            assert!(
                problem["detail"].as_str().is_some_and(|d| !d.is_empty()),
                "missing detail for {hook0_problem}"
            );
        }
    }
}
