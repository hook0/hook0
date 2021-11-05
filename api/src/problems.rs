use actix_web::{HttpResponse, ResponseError};
use http_api_problem::*;
use paperclip::actix::api_v2_errors;
use sqlx::postgres::PgDatabaseError;
use sqlx::Error;
use strum::{EnumIter};

/**
 * How to implement a new type error for Hook0:
 * 1/ Add the type error variant inside Hook0Problem enum
 * 2/ Implement the Problem inside Into<Problem> for Hook0Problem
 * 3/ Done! Enjoy!
 */
#[api_v2_errors(code = 403, code = 500, code = 400, code = 404, code = 409)]
#[derive(Debug, Copy, Clone, EnumIter, strum::Display)]
pub enum Hook0Problem {
    // Functional errors
    ApplicationNameMissing,

    EventAlreadyIngested,
    EventInvalidPayloadContentType,
    EventInvalidBase64Payload,
    EventInvalidMetadata,
    EventInvalidLabels,

    // Auth errors
    AuthNoAuthorizationHeader,
    AuthInvalidAuthorizationHeader,
    AuthApplicationSecretLookupError,
    AuthInvalidApplicationSecret,

    // Generic errors
    NotFound,
    InternalServerError,
    Forbidden,
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
                    Some("event_pkey") => Hook0Problem::EventAlreadyIngested,
                    _ => Hook0Problem::InternalServerError,
                }
            }
            _ => Hook0Problem::InternalServerError,
        }
    }
}

impl From<Hook0Problem> for HttpApiProblem {
    fn from(hook0_problem: Hook0Problem) -> Self {
        let problem: Problem = hook0_problem.into();
        HttpApiProblem::new(problem.status)
            .type_url(format!(
                "https://hook0.com/documentation/errors/{}",
                hook0_problem
            )) // rely on Display trait of Hook0Problem
            .value("id".to_string(), &format!("{}", hook0_problem)) // also rely on Display trait of Hook0Problem
            .title(problem.title)
            .detail(problem.detail)
    }
}

impl ResponseError for Hook0Problem {
    fn status_code(&self) -> StatusCode {
        let problem: Problem = (*self).into();
        problem.status
    }

    fn error_response(&self) -> HttpResponse {
        let problem: HttpApiProblem = (*self).into();

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

#[derive(Debug, Copy, Clone)]
pub struct Problem {
    pub id: Hook0Problem,
    pub title: &'static str,
    pub detail: &'static str,
    pub status: StatusCode,
}

impl From<Hook0Problem> for Problem {
    fn from(hook0_problem: Hook0Problem) -> Self {
        match hook0_problem {
            // Functional errors
            Hook0Problem::ApplicationNameMissing => Problem {
                id: Hook0Problem::ApplicationNameMissing,
                title: "Application name cannot be empty",
                detail: "Application name length must have more than 1 character",
                status: StatusCode::BAD_REQUEST,
            },

            Hook0Problem::EventAlreadyIngested => Problem {
                id: Hook0Problem::EventAlreadyIngested,
                title: "Event already Ingested",
                detail: "This event was previously ingested and recorded inside Hook0 service.",
                status: StatusCode::CONFLICT,
            },
            Hook0Problem::EventInvalidPayloadContentType => Problem {
                id: Hook0Problem::EventInvalidPayloadContentType,
                title: "Invalid event payload content type",
                detail: "The specified event payload content type is not registered. If this is not a mistake, please create it with /event_types",
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::EventInvalidBase64Payload => Problem {
                id: Hook0Problem::EventInvalidBase64Payload,
                title: "Invalid event payload",
                detail: "Event payload is not encoded in valid base64 format.",
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::EventInvalidMetadata => Problem {
                id: Hook0Problem::EventInvalidMetadata,
                title: "Invalid event metadata content",
                detail: "When specified, event metadata must be a key-value map in JSON object format.",
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::EventInvalidLabels => Problem {
                id: Hook0Problem::EventInvalidLabels,
                title: "Invalid event labels",
                detail: "When specified, event labels must be a key-value map in JSON object format.",
                status: StatusCode::BAD_REQUEST,
            },

            // Auth error
            Hook0Problem::AuthNoAuthorizationHeader => Problem {
                id: Hook0Problem::AuthNoAuthorizationHeader,
                title: "No `Authorization` header was found in the HTTP request.",
                detail: "`Authorization` header must be provided and must contain a bearer token.",
                status: StatusCode::UNAUTHORIZED,
            },
            Hook0Problem::AuthInvalidAuthorizationHeader => Problem {
                id: Hook0Problem::AuthInvalidAuthorizationHeader,
                title: "`Authorization` header is invalid",
                detail: "`Authorization` header value could not be decoded as a valid UTF-8 string containing `Bearer {UUID}`.",
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::AuthApplicationSecretLookupError => Problem {
                id: Hook0Problem::AuthApplicationSecretLookupError,
                title: "Could not check database to verify the provided application secret",
                detail: "This is likely to be caused by database unavailability.",
                status: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Hook0Problem::AuthInvalidApplicationSecret => Problem {
                id: Hook0Problem::AuthInvalidApplicationSecret,
                title: "Invalid application secret",
                detail: "The provided application secret does not exist.",
                status: StatusCode::FORBIDDEN,
            },

            // Generic errors
            Hook0Problem::InternalServerError => Problem {
                id: Hook0Problem::InternalServerError,
                title: "Something wrong happened",
                detail: "Hook0 server had issue handling your request. Our team was notified.",
                status: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Hook0Problem::NotFound => Problem {
                id: Hook0Problem::NotFound,
                title: "Item not found",
                detail: "Could not find the item. Check the identifier or that you have the right to access it.",
                status: StatusCode::NOT_FOUND,
            },
            Hook0Problem::Forbidden => Problem {
                id: Hook0Problem::Forbidden,
                title: "Insufficient rights",
                detail: "You don't have the right to access or edit this resource.",
                status: StatusCode::FORBIDDEN,
            },
        }
    }
}
