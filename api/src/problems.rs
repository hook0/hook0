use actix_web::error::JsonPayloadError;
use actix_web::{HttpResponse, ResponseError};
use http_api_problem::*;
use log::error;
use paperclip::actix::api_v2_errors;
use serde_json::{to_value, Value};
use sqlx::postgres::PgDatabaseError;
use sqlx::Error;
use std::borrow::Cow;
use std::fmt::Display;
use strum::{EnumIter, VariantNames};

use crate::handlers::events::PayloadContentType;
use crate::handlers::instance::HealthCheck;
use crate::iam::Role;
use crate::quotas::QuotaValue;

/**
 * How to implement a new type error for Hook0:
 * 1/ Add the type error variant inside Hook0Problem enum
 * 2/ Implement the Problem inside From<Hook0Problem> for Problem
 * 3/ Done! Enjoy!
 */
#[api_v2_errors(code = 403, code = 500, code = 400, code = 404, code = 409)]
#[derive(Debug, Clone, EnumIter, strum::Display)]
pub enum Hook0Problem {
    // Functional errors
    OrganizationNameMissing,
    UserAlreadyExist,
    RegistrationDisabled,
    PasswordTooShort(u8),
    OrganizationIsNotEmpty,
    InvitedUserDoesNotExist,

    ApplicationNameMissing,

    InvalidRole,

    EventTypeAlreadyExist,

    UnauthorizedWorkers(Vec<String>),

    EventAlreadyIngested,
    EventInvalidPayloadContentType,
    EventInvalidBase64Payload(String),
    EventInvalidJsonPayload(String),

    // Auth errors
    AuthNoAuthorizationHeader,
    AuthInvalidAuthorizationHeader,
    AuthApplicationSecretLookupError,
    AuthInvalidApplicationSecret,
    AuthBiscuitLookupError,
    AuthInvalidBiscuit,
    AuthFailedLogin,
    AuthFailedRefresh,

    // Quota errors
    TooManyMembersPerOrganization(QuotaValue),
    TooManyApplicationsPerOrganization(QuotaValue),
    TooManyEventsToday(QuotaValue),

    // Generic errors
    JsonPayload(JsonPayloadProblem),
    Validation(validator::ValidationErrors),
    NotFound,
    InternalServerError,
    Forbidden,
    ServiceUnavailable(HealthCheck),
    EmailSending(String),
}

impl From<sqlx::Error> for Hook0Problem {
    fn from(e: Error) -> Self {
        match e {
            Error::RowNotFound => Hook0Problem::NotFound,
            Error::Database(ex) => {
                // Goal map Box<dyn DatabaseError> to PgDatabaseError
                let pg_error: &PgDatabaseError = ex.try_downcast_ref::<PgDatabaseError>().unwrap();

                //let pg_error: PgDatabaseError = ex.into();

                match pg_error.constraint() {
                    Some("application_name_chk") => Hook0Problem::ApplicationNameMissing,
                    Some("event_type_pkey") => Hook0Problem::EventTypeAlreadyExist,
                    Some("event_pkey") => Hook0Problem::EventAlreadyIngested,
                    _ => {
                        error!("Database error: {}", &pg_error);
                        Hook0Problem::InternalServerError
                    }
                }
            }
            err => {
                error!("{}", &err);
                Hook0Problem::InternalServerError
            }
        }
    }
}

impl From<Hook0Problem> for HttpApiProblem {
    fn from(hook0_problem: Hook0Problem) -> Self {
        let problem: Problem = hook0_problem.to_owned().into();
        HttpApiProblem::new(problem.status)
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
        let problem: HttpApiProblem = self.to_owned().into();

        let effective_status = problem
            .status
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
        let actix_status = actix_web::http::StatusCode::from_u16(effective_status.as_u16())
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);

        let json = problem.json_bytes();

        actix_web::HttpResponse::build(actix_status)
            .append_header((
                actix_web::http::header::CONTENT_TYPE,
                PROBLEM_JSON_MEDIA_TYPE,
            ))
            .body(json)
    }
}

impl From<lettre::error::Error> for Hook0Problem {
    fn from(err: lettre::error::Error) -> Hook0Problem {
        Hook0Problem::EmailSending(err.to_string())
    }
}

impl From<lettre::transport::smtp::Error> for Hook0Problem {
    fn from(err: lettre::transport::smtp::Error) -> Hook0Problem {
        Hook0Problem::EmailSending(err.to_string())
    }
}

impl From<mrml::prelude::parser::Error> for Hook0Problem {
    fn from(err: mrml::prelude::parser::Error) -> Hook0Problem {
        Hook0Problem::EmailSending(err.to_string())
    }
}

impl From<mrml::prelude::render::Error> for Hook0Problem {
    fn from(err: mrml::prelude::render::Error) -> Hook0Problem {
        Hook0Problem::EmailSending(err.to_string())
    }
}

impl std::error::Error for Hook0Problem {}

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
            Hook0Problem::OrganizationIsNotEmpty => Problem {
                id: Hook0Problem::OrganizationIsNotEmpty,
                title: "Organization is not empty",
                detail: "Organizations that contain at least an application cannot be deleted; applications must be deleted first.".into(),
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
            Hook0Problem::AuthFailedRefresh => Problem {
                id: Hook0Problem::AuthFailedRefresh,
                title: "Refreshing access token failed",
                detail: "The provided refresh token is probably invalid or expired.".into(),
                validation: None,
                status: StatusCode::UNAUTHORIZED,
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
                let errors_str = e.to_string();
                Problem {
                    id: Hook0Problem::Validation(e.to_owned()),
                    title: "Provided input is malformed",
                    detail: errors_str.into(),
                    validation: to_value(e).ok(),
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
            Hook0Problem::ServiceUnavailable(h) => Problem {
                id: Hook0Problem::ServiceUnavailable(h),
                title: "Service unavailable",
                detail: format!("{h}.").into(),
                validation: None,
                status: StatusCode::SERVICE_UNAVAILABLE,
            },
            Hook0Problem::EmailSending(e) => Problem {
                id: Hook0Problem::EmailSending(e.to_owned()),
                title: "Could not send email",
                detail: format!("{e}.").into(),
                validation: None,
                status: StatusCode::INTERNAL_SERVER_ERROR,
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
        Self::Other("".to_owned())
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
