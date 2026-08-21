use actix_web::error::JsonPayloadError;
use actix_web::http::{StatusCode, header};
use actix_web::{HttpResponse, ResponseError};
use http_api_problem::{HttpApiProblem, PROBLEM_JSON_MEDIA_TYPE};
use paperclip::actix::api_v2_errors;
use paperclip::v2::models::{DataType, DefaultSchemaRaw};
use paperclip::v2::schema::Apiv2Schema;
use serde::{Serialize, Serializer};
use serde_json::{Value, to_value};
use sqlx::Error;
use std::borrow::Cow;
use std::fmt::Display;
use strum::{EnumIter, IntoEnumIterator, VariantNames};
use tracing::{error, warn};

use crate::handlers::events::PayloadContentType;
use crate::iam::Role;
use crate::quotas::QuotaValue;
use crate::validators::CODE_SECRET_PREFIX;

/// The published shape of an error body. `api_v2_errors` below names it as the schema every
/// error response answers with, and it parses that name as a bare identifier that it also reuses
/// verbatim to build the `$ref`, so the import has to be spelled exactly like this: no qualified
/// path, and no rename.
use crate::handlers::errors::Problem;

/**
 * How to implement a new type error for Hook0:
 * 1/ Add the type error variant inside Hook0Problem enum
 * 2/ Implement the ProblemDetails inside From<Hook0Problem> for ProblemDetails
 * 3/ Give it its public identifier inside Hook0Problem::id (the compiler asks for it)
 * 4/ Done! Enjoy!
 */
#[api_v2_errors(
    default_schema = "Problem",
    code = 400,
    code = 401,
    code = 403,
    code = 404,
    code = 409,
    code = 410,
    code = 422,
    code = 429,
    code = 500,
    code = 503
)]
#[derive(Debug, Clone, EnumIter)]
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
    AuthEmailAlreadyVerified,
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
    RateLimited,
    ServiceUnavailable,
}

/// Public identifier of a problem: the value carried by the `id` member of the RFC 7807 body.
///
/// It is rendered in the OpenAPI schema as a closed string enumeration rather than free-form
/// text, so that a client can match on the exact problem it received instead of comparing
/// strings it had to discover by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hook0ProblemId(&'static str);

impl Hook0ProblemId {
    pub const fn as_str(&self) -> &'static str {
        self.0
    }

    /// Documentation page of this problem, as carried by the `type` member of the RFC 7807 body.
    ///
    /// Built here rather than at each place a body is produced, so what the API sends and what it
    /// publishes cannot say two different things.
    ///
    /// It points at the anchor of this problem on the error reference, which is one page listing
    /// every identifier rather than a page each. The address this used to send people to,
    /// `hook0.com/documentation/errors/<id>`, answered 404 for every problem the API can return:
    /// the documentation is served from its own host, and it never had a page per error.
    ///
    /// The anchor is the identifier lowercased, which is what the documentation generator makes of
    /// a heading.
    pub fn type_url(&self) -> String {
        format!(
            "https://documentation.hook0.com/reference/error-codes#{}",
            self.0.to_lowercase()
        )
    }

    /// Every identifier the API can answer with, in variant declaration order.
    ///
    /// It is built by walking the variants, so it cannot drift from what `Hook0Problem::id`
    /// returns.
    fn all() -> Vec<Self> {
        Hook0Problem::iter().map(|problem| problem.id()).collect()
    }
}

impl Display for Hook0ProblemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl Serialize for Hook0ProblemId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.0)
    }
}

impl Apiv2Schema for Hook0ProblemId {
    fn name() -> Option<String> {
        Some("Hook0ProblemId".to_owned())
    }

    fn raw_schema() -> DefaultSchemaRaw {
        DefaultSchemaRaw {
            data_type: Some(DataType::String),
            enum_: Self::all()
                .iter()
                .map(|id| Value::String(id.as_str().to_owned()))
                .collect(),
            description: Some(
                "Identifier of the problem that occurred, stable across releases".to_owned(),
            ),
            ..Default::default()
        }
    }
}

impl Hook0Problem {
    /// Public identifier of this problem, as carried by the `id` member of the RFC 7807 body and
    /// published in the OpenAPI schema.
    ///
    /// The match is exhaustive on purpose: a new variant does not compile until it is given an
    /// identifier, which keeps the published enumeration complete without a second list to
    /// maintain. The identifier never depends on the variant payload, so a client can match on
    /// it whatever the details of the failure.
    pub fn id(&self) -> Hook0ProblemId {
        let id = match self {
            // Functional errors
            Self::OrganizationNameMissing => "OrganizationNameMissing",
            Self::UserAlreadyExist => "UserAlreadyExist",
            Self::RegistrationDisabled => "RegistrationDisabled",
            Self::PasswordTooShort(_) => "PasswordTooShort",
            Self::PasswordTooLong => "PasswordTooLong",
            Self::PasswordSimilarToEmail => "PasswordSimilarToEmail",
            Self::PasswordSimilarToName => "PasswordSimilarToName",
            Self::PasswordTooCommon => "PasswordTooCommon",
            Self::PasswordNotDiverseEnough => "PasswordNotDiverseEnough",
            Self::OrganizationIsNotEmpty => "OrganizationIsNotEmpty",
            Self::InvitedUserDoesNotExist => "InvitedUserDoesNotExist",
            Self::InvitedUserAlreadyInOrganization => "InvitedUserAlreadyInOrganization",

            Self::ApplicationNameMissing => "ApplicationNameMissing",

            Self::InvalidRole => "InvalidRole",

            Self::EventTypeAlreadyExist => "EventTypeAlreadyExist",
            Self::EventTypeDoesNotExist => "EventTypeDoesNotExist",

            Self::UnauthorizedWorkers(_) => "UnauthorizedWorkers",

            Self::EventAlreadyIngested => "EventAlreadyIngested",
            Self::EventInvalidPayloadContentType => "EventInvalidPayloadContentType",
            Self::EventInvalidBase64Payload(_) => "EventInvalidBase64Payload",
            Self::EventInvalidJsonPayload(_) => "EventInvalidJsonPayload",

            Self::LabelsAmbiguity => "LabelsAmbiguity",

            Self::InvalidDateRange => "InvalidDateRange",

            // Auth errors
            Self::AuthNoAuthorizationHeader => "AuthNoAuthorizationHeader",
            Self::AuthInvalidAuthorizationHeader => "AuthInvalidAuthorizationHeader",
            Self::AuthApplicationSecretLookupError => "AuthApplicationSecretLookupError",
            Self::AuthInvalidApplicationSecret => "AuthInvalidApplicationSecret",
            Self::AuthBiscuitLookupError => "AuthBiscuitLookupError",
            Self::AuthInvalidBiscuit => "AuthInvalidBiscuit",
            Self::AuthFailedLogin => "AuthFailedLogin",
            Self::AuthEmailNotVerified => "AuthEmailNotVerified",
            Self::AuthEmailAlreadyVerified => "AuthEmailAlreadyVerified",
            Self::AuthFailedRefresh => "AuthFailedRefresh",
            Self::AuthEmailExpired => "AuthEmailExpired",

            // Quota errors
            Self::TooManyMembersPerOrganization(_) => "TooManyMembersPerOrganization",
            Self::TooManyApplicationsPerOrganization(_) => "TooManyApplicationsPerOrganization",
            Self::TooManyEventsToday(_) => "TooManyEventsToday",
            Self::TooManySubscriptionsPerApplication(_) => "TooManySubscriptionsPerApplication",
            Self::TooManyEventTypesPerApplication(_) => "TooManyEventTypesPerApplication",

            // Generic errors
            Self::JsonPayload(_) => "JsonPayload",
            Self::Validation(_) => "Validation",
            Self::NotFound => "NotFound",
            Self::InternalServerError => "InternalServerError",
            Self::Forbidden => "Forbidden",
            Self::RateLimited => "RateLimited",
            Self::ServiceUnavailable => "ServiceUnavailable",
        };
        Hook0ProblemId(id)
    }
}

impl Display for Hook0Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.id(), f)
    }
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
        let problem: ProblemDetails = hook0_problem.to_owned().into();
        HttpApiProblem::new(to_problem_status(problem.status))
            .type_url(hook0_problem.id().type_url())
            .value("id".to_owned(), &hook0_problem.id())
            .value("validation".to_owned(), &problem.validation)
            .title(problem.title)
            .detail(problem.detail)
    }
}

impl ResponseError for Hook0Problem {
    fn status_code(&self) -> StatusCode {
        let problem: ProblemDetails = self.to_owned().into();
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
pub struct ProblemDetails {
    pub id: Hook0Problem,
    pub title: &'static str,
    pub detail: Cow<'static, str>,
    pub validation: Option<Value>,
    pub status: StatusCode,
}

impl From<Hook0Problem> for ProblemDetails {
    fn from(hook0_problem: Hook0Problem) -> Self {
        match hook0_problem {
            // Functional errors
            Hook0Problem::OrganizationNameMissing => ProblemDetails {
                id: Hook0Problem::OrganizationNameMissing,
                title: "Organization name cannot be empty",
                detail: "Organization name length must have more than 1 character.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::UserAlreadyExist => ProblemDetails {
                id: Hook0Problem::UserAlreadyExist,
                title: "This user already exist",
                detail: "This email is already registered.".into(),
                validation: None,
                status: StatusCode::CONFLICT,
            },
            Hook0Problem::RegistrationDisabled => ProblemDetails {
                id: Hook0Problem::RegistrationDisabled,
                title: "Registrations are disabled",
                detail: "Registration was disabled by an administrator.".into(),
                validation: None,
                status: StatusCode::GONE,
            },
            Hook0Problem::PasswordTooShort(minimum_length) => {
                let detail = format!("Password must be at least {minimum_length} characters long.");
                ProblemDetails {
                    id: Hook0Problem::PasswordTooShort(minimum_length),
                    title: "Provided password is too short",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::BAD_REQUEST,
                }
            },
            Hook0Problem::PasswordTooLong => ProblemDetails {
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
            Hook0Problem::PasswordSimilarToEmail => ProblemDetails {
                id: Hook0Problem::PasswordSimilarToEmail,
                title: "Provided password is too close to the email address",
                detail: "Password must not be built from the email address of the account: anyone who knows the address would guess it. Please pick something unrelated.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::PasswordSimilarToName => ProblemDetails {
                id: Hook0Problem::PasswordSimilarToName,
                title: "Provided password is too close to the user name",
                detail: "Password must not be built from the first or last name of the account. Please pick something unrelated.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::PasswordTooCommon => ProblemDetails {
                id: Hook0Problem::PasswordTooCommon,
                title: "Provided password is too common",
                detail: "This password (or a lightly disguised version of it) is among the most frequently used ones, so it is one of the first an attacker tries. Please pick another one.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::PasswordNotDiverseEnough => ProblemDetails {
                id: Hook0Problem::PasswordNotDiverseEnough,
                title: "Provided password is not diverse enough",
                detail: "Password is made of too few different characters, which makes it easy to guess despite its length. Please pick another one.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::OrganizationIsNotEmpty => ProblemDetails {
                id: Hook0Problem::OrganizationIsNotEmpty,
                title: "Organization is not empty",
                detail: "Organizations that contain at least an application cannot be deleted; applications must be deleted first. If you believe this is a mistake, please contact the Hook0 support team.".into(),
                validation: None,
                status: StatusCode::CONFLICT,
            },
            Hook0Problem::InvitedUserDoesNotExist => ProblemDetails {
                id: Hook0Problem::InvitedUserDoesNotExist,
                title: "Invited user does not exist",
                detail: "The user you are trying to invite does not exist. Please make sure the user is already register in Hook0.".into(),
                validation: None,
                status: StatusCode::NOT_FOUND,
            },
            Hook0Problem::InvitedUserAlreadyInOrganization => ProblemDetails {
                id: Hook0Problem::InvitedUserAlreadyInOrganization,
                title: "Invited user is already in the organization",
                detail: "The user you are trying to invite has already access to the organization.".into(),
                validation: None,
                status: StatusCode::CONFLICT,
            },

            Hook0Problem::ApplicationNameMissing => ProblemDetails {
                id: Hook0Problem::ApplicationNameMissing,
                title: "Application name cannot be empty",
                detail: "Application name length must have more than 1 character.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },

            Hook0Problem::InvalidRole => {
                let roles = format!("Valid roles are: {}.", Role::VARIANTS.join(", "));
                ProblemDetails {
                    id: Hook0Problem::InvalidRole,
                    title: "Provided role does not exist",
                    detail: roles.into(),
                    validation: None,
                    status: StatusCode::BAD_REQUEST,
                }
            },

            Hook0Problem::EventTypeAlreadyExist => ProblemDetails {
                id: Hook0Problem::EventTypeAlreadyExist,
                title: "This event type already exist",
                detail: "An event type with this name is already present.".into(),
                validation: None,
                status: StatusCode::CONFLICT,
            },
            Hook0Problem::EventTypeDoesNotExist => ProblemDetails {
                id: Hook0Problem::EventTypeDoesNotExist,
                title: "Invalid event type",
                detail: "Event type does not exist or was deactivated. You should (re)create it.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },

            Hook0Problem::UnauthorizedWorkers(w) => {
                let detail = format!("You do not have access to the following workers: {}", w.join(", "));
                ProblemDetails {
                    id: Hook0Problem::UnauthorizedWorkers(w),
                    title: "Some of the provided dedicated workers are not authorized for your organization",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::BAD_REQUEST,
                }
            },

            Hook0Problem::EventAlreadyIngested => ProblemDetails {
                id: Hook0Problem::EventAlreadyIngested,
                title: "Event already Ingested",
                detail: "This event was previously ingested and recorded inside Hook0 service.".into(),
                validation: None,
                status: StatusCode::CONFLICT,
            },
            Hook0Problem::EventInvalidPayloadContentType => {
                let detail = format!("The specified event payload content type is not handled. Valid content types are: {}", PayloadContentType::VARIANTS.join(", "));
                ProblemDetails {
                    id: Hook0Problem::EventInvalidPayloadContentType,
                    title: "Invalid event payload content type",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::BAD_REQUEST,
                }
            },
            Hook0Problem::EventInvalidBase64Payload(e) => {
                let detail = format!("Event payload is not encoded in valid base64 format: {e}");
                ProblemDetails {
                    id: Hook0Problem::EventInvalidBase64Payload(e),
                    title: "Invalid event base64 payload",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::BAD_REQUEST,
                }
            },
            Hook0Problem::EventInvalidJsonPayload(e) => {
                let detail = format!("Event payload is not encoded in valid JSON format: {e}.");
                ProblemDetails {
                    id: Hook0Problem::EventInvalidJsonPayload(e),
                    title: "Invalid event JSON payload",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::BAD_REQUEST,
                }
            },
            Hook0Problem::LabelsAmbiguity => ProblemDetails {
                id: Hook0Problem::LabelsAmbiguity,
                title: "Ambiguous labels specification",
                detail: "You must specify either the `labels` property as an object with a least one property (recommended) or separated `label_key` and `label_value` properties as strings (legacy), but not both.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },

            Hook0Problem::InvalidDateRange => ProblemDetails {
                id: Hook0Problem::InvalidDateRange,
                title: "Invalid date range",
                detail: "'from' date must not be after 'to' date.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },

            // Auth error
            Hook0Problem::AuthNoAuthorizationHeader => ProblemDetails {
                id: Hook0Problem::AuthNoAuthorizationHeader,
                title: "No `Authorization` header was found in the HTTP request",
                detail: "`Authorization` header must be provided and must contain a bearer token.".into(),
                validation: None,
                status: StatusCode::UNAUTHORIZED,
            },
            Hook0Problem::AuthInvalidAuthorizationHeader => ProblemDetails {
                id: Hook0Problem::AuthInvalidAuthorizationHeader,
                title: "`Authorization` header is invalid",
                detail: "`Authorization` header value could not be decoded as a valid UTF-8 string containing `Bearer {UUID}`.".into(),
                validation: None,
                status: StatusCode::BAD_REQUEST,
            },
            Hook0Problem::AuthApplicationSecretLookupError => ProblemDetails {
                id: Hook0Problem::AuthApplicationSecretLookupError,
                title: "Could not check database to verify the provided application secret",
                detail: "This is likely to be caused by database unavailability.".into(),
                validation: None,
                status: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Hook0Problem::AuthInvalidApplicationSecret => ProblemDetails {
                id: Hook0Problem::AuthInvalidApplicationSecret,
                title: "Invalid application secret",
                detail: "The provided application secret does not exist.".into(),
                validation: None,
                status: StatusCode::FORBIDDEN,
            },
            Hook0Problem::AuthBiscuitLookupError => ProblemDetails {
                id: Hook0Problem::AuthBiscuitLookupError,
                title: "Could not check database to verify if the provided Biscuit was revoked",
                detail: "This is likely to be caused by database unavailability.".into(),
                validation: None,
                status: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Hook0Problem::AuthInvalidBiscuit => ProblemDetails {
                id: Hook0Problem::AuthInvalidBiscuit,
                title: "Invalid Biscuit",
                detail: "The provided authentication token (Biscuit) is not valid, was not created using the current private key or is expired.".into(),
                validation: None,
                status: StatusCode::FORBIDDEN,
            },
            Hook0Problem::AuthFailedLogin => ProblemDetails {
                id: Hook0Problem::AuthFailedLogin,
                title: "Login failed",
                detail: "The provided credentials do not match ones of a valid user.".into(),
                validation: None,
                status: StatusCode::UNAUTHORIZED,
            },
            Hook0Problem::AuthEmailNotVerified => ProblemDetails {
                id: Hook0Problem::AuthEmailNotVerified,
                title: "Email not verified",
                detail: "Your email has not been verified yet. Please check your inbox.".into(),
                validation: None,
                status: StatusCode::UNAUTHORIZED,
            },
            Hook0Problem::AuthFailedRefresh => ProblemDetails {
                id: Hook0Problem::AuthFailedRefresh,
                title: "Refreshing access token failed",
                detail: "The provided refresh token is probably invalid or expired.".into(),
                validation: None,
                status: StatusCode::UNAUTHORIZED,
            },
            Hook0Problem::AuthEmailAlreadyVerified => ProblemDetails {
                id: Hook0Problem::AuthEmailAlreadyVerified,
                title: "Email already verified",
                detail: "This address has already been verified, so this link has nothing left to do. Sign in to continue.".into(),
                validation: None,
                status: StatusCode::CONFLICT,
            },
            Hook0Problem::AuthEmailExpired => {
                ProblemDetails {
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
                ProblemDetails {
                    id: Hook0Problem::TooManyMembersPerOrganization(limit),
                    title: "Exceeded number of users that can be invited in this organization",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::TOO_MANY_REQUESTS,
                }
            },
            Hook0Problem::TooManyApplicationsPerOrganization(limit) => {
                let detail = format!("This organization cannot have more than {limit} applications. You might want to upgrade to a better plan.");
                ProblemDetails {
                    id: Hook0Problem::TooManyApplicationsPerOrganization(limit),
                    title: "Exceeded number of applications that can be created in this organization",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::TOO_MANY_REQUESTS,
                }
            },
            Hook0Problem::TooManyEventsToday(limit) => {
                let detail = format!("This organization cannot ingest more than {limit} events per day. You might want to upgrade to a better plan.");
                ProblemDetails {
                    id: Hook0Problem::TooManyEventsToday(limit),
                    title: "Exceeded number of events that can be ingested in this organization today",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::TOO_MANY_REQUESTS,
                }
            },
            Hook0Problem::TooManySubscriptionsPerApplication(limit) => {
                let detail = format!("This application cannot have more than {limit} subscriptions. You might want to upgrade to a better plan.");
                ProblemDetails {
                    id: Hook0Problem::TooManySubscriptionsPerApplication(limit),
                    title: "Exceeded number of subscriptions that can be created in this application",
                    detail: detail.into(),
                    validation: None,
                    status: StatusCode::TOO_MANY_REQUESTS,
                }
            },
            Hook0Problem::TooManyEventTypesPerApplication(limit) => {
                let detail = format!("This application cannot have more than {limit} event types. You might want to upgrade to a better plan.");
                ProblemDetails {
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
                ProblemDetails {
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
                ProblemDetails {
                    id: Hook0Problem::Validation(e.to_owned()),
                    title: "Provided input is malformed",
                    detail: detail.into(),
                    validation: to_value(e).ok().map(without_secret_values),
                    status: StatusCode::UNPROCESSABLE_ENTITY,
                }
            },
            Hook0Problem::InternalServerError => ProblemDetails {
                id: Hook0Problem::InternalServerError,
                title: "Something wrong happened",
                detail: "Hook0 server had issue handling your request. Our team was notified.".into(),
                validation: None,
                status: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Hook0Problem::NotFound => ProblemDetails {
                id: Hook0Problem::NotFound,
                title: "Item not found",
                detail: "Could not find the item. Check the identifier or that you have the right to access it.".into(),
                validation: None,
                status: StatusCode::NOT_FOUND,
            },
            Hook0Problem::Forbidden => ProblemDetails {
                id: Hook0Problem::Forbidden,
                title: "Insufficient rights",
                detail: "You don't have the right to access or edit this resource.".into(),
                validation: None,
                status: StatusCode::FORBIDDEN,
            },
            Hook0Problem::RateLimited => ProblemDetails {
                id: Hook0Problem::RateLimited,
                title: "Too many requests",
                detail: "Requests are coming in faster than this Hook0 instance accepts them, so this one was not processed. This is a temporary, client-side pacing condition, not a rights issue: the request is safe to send again once the delay given by the `Retry-After` response header has elapsed.".into(),
                validation: None,
                status: StatusCode::TOO_MANY_REQUESTS,
            },
            Hook0Problem::ServiceUnavailable => ProblemDetails {
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

    use actix_web::body::{MessageBody, to_bytes};
    use paperclip::actix::{OpenApiExt, web};
    use proptest::prelude::*;
    use std::collections::BTreeSet;
    use std::mem::discriminant;
    use std::sync::OnceLock;
    use url::Url;

    /// The identifiers published in the OpenAPI schema, as a client reading the spec sees them.
    fn published_identifiers() -> BTreeSet<String> {
        Hook0ProblemId::raw_schema()
            .enum_
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_owned)
                    .expect("published problem identifiers should be JSON strings")
            })
            .collect()
    }

    /// Every problem the API can answer with, with an arbitrary payload wherever a variant
    /// carries one, so that the identifier is exercised independently of the failure details.
    fn any_problem() -> impl Strategy<Value = Hook0Problem> {
        (
            proptest::sample::select(Hook0Problem::iter().collect::<Vec<_>>()),
            any::<u8>(),
            proptest::collection::vec("\\PC{0,32}", 0..8),
            "\\PC{0,128}",
            any::<QuotaValue>(),
            0_usize..1_000_000,
        )
            .prop_map(|(problem, byte, words, text, quota, limit)| match problem {
                Hook0Problem::PasswordTooShort(_) => Hook0Problem::PasswordTooShort(byte),
                Hook0Problem::UnauthorizedWorkers(_) => Hook0Problem::UnauthorizedWorkers(words),
                Hook0Problem::EventInvalidBase64Payload(_) => {
                    Hook0Problem::EventInvalidBase64Payload(text)
                }
                Hook0Problem::EventInvalidJsonPayload(_) => {
                    Hook0Problem::EventInvalidJsonPayload(text)
                }
                Hook0Problem::TooManyMembersPerOrganization(_) => {
                    Hook0Problem::TooManyMembersPerOrganization(quota)
                }
                Hook0Problem::TooManyApplicationsPerOrganization(_) => {
                    Hook0Problem::TooManyApplicationsPerOrganization(quota)
                }
                Hook0Problem::TooManyEventsToday(_) => Hook0Problem::TooManyEventsToday(quota),
                Hook0Problem::TooManySubscriptionsPerApplication(_) => {
                    Hook0Problem::TooManySubscriptionsPerApplication(quota)
                }
                Hook0Problem::TooManyEventTypesPerApplication(_) => {
                    Hook0Problem::TooManyEventTypesPerApplication(quota)
                }
                Hook0Problem::JsonPayload(_) => {
                    Hook0Problem::JsonPayload(JsonPayloadProblem::Overflow { limit })
                }
                other => other,
            })
    }

    /// A client generated from the spec can only recognize the problems the spec enumerates, so
    /// the enumeration must cover every problem the API can answer with, once each.
    #[test]
    fn published_enumeration_covers_every_problem_exactly_once() {
        let schema = Hook0ProblemId::raw_schema();
        assert_eq!(
            schema.data_type,
            Some(DataType::String),
            "problem identifiers must be published as strings"
        );

        let published = published_identifiers();
        for problem in Hook0Problem::iter() {
            assert!(
                published.contains(problem.id().as_str()),
                "{problem} is missing from the published identifier enumeration"
            );
        }
        assert_eq!(
            published.len(),
            Hook0Problem::iter().count(),
            "the published enumeration holds duplicate or unknown identifiers: {published:?}"
        );
    }

    /// The identifier a client reads in the spec is the one it actually receives on the wire.
    #[actix_web::test]
    async fn problem_body_carries_the_published_identifier() {
        for problem in Hook0Problem::iter() {
            let expected = problem.id();
            let response = problem.error_response();

            let body = to_bytes(response.into_body())
                .await
                .expect("error response body should be readable");
            let json = serde_json::from_slice::<Value>(&body)
                .expect("error response body should be valid JSON");

            assert_eq!(
                json["id"].as_str(),
                Some(expected.as_str()),
                "unexpected `id` member for {problem}"
            );
            assert_eq!(
                json["type"].as_str(),
                Some(
                    format!(
                        "https://documentation.hook0.com/reference/error-codes#{}",
                        expected.0.to_lowercase()
                    )
                    .as_str()
                ),
                "unexpected `type` member for {problem}"
            );
        }
    }

    /// Builds the v3 document paperclip serves, off an app carrying a single operation. No
    /// server and no database are involved.
    fn produced_spec() -> Value {
        let app_url = Url::parse("https://api.hook0.test/").expect("the test app url parses");
        let mut produced = None;

        let _app = actix_web::App::new()
            .wrap_api_with_spec(crate::openapi::default_spec(&app_url))
            .service(web::resource("/errors").route(web::get().to(crate::handlers::errors::list)))
            .with_raw_json_spec_v3(|app, spec| {
                produced = Some(spec);
                app
            })
            .build();

        produced.expect("paperclip yields the v3 document it would serve")
    }

    /// Read from the document paperclip actually serves rather than from the Rust types it was
    /// built from: that JSON is the whole of what a client generator consumes, so it is the only
    /// place where the guarantee is worth anything.
    #[test]
    fn the_produced_spec_publishes_every_problem_identifier() {
        let spec = produced_spec();

        // An error response has to answer with a body a generator can type, not with a bare
        // description.
        let responses = spec["paths"]["/errors"]["get"]["responses"]
            .as_object()
            .expect("the mounted operation declares responses");
        let error_codes = responses
            .keys()
            .filter(|code| code.parse::<u16>().is_ok_and(|code| code >= 400))
            .collect::<Vec<_>>();
        assert!(
            !error_codes.is_empty(),
            "the mounted operation declares no error response, so there is nothing to check"
        );
        for code in &error_codes {
            let content = &responses[code.as_str()]["content"]["application/json"]["schema"];
            assert!(
                content.is_object(),
                "response {code} carries no schema in its content, it is {}",
                responses[code.as_str()]
            );
        }

        // And the identifier it carries has to be a closed enumeration covering every problem.
        let problem = &spec["components"]["schemas"]["Problem"];
        assert!(
            problem.is_object(),
            "the error responses point at a Problem schema the components never define"
        );

        let identifier = &problem["properties"]["id"];
        let published = identifier["enum"]
            .as_array()
            .unwrap_or_else(|| {
                panic!("the served Problem publishes no enumeration for `id`, it publishes {identifier}")
            })
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_owned)
                    .expect("published identifiers are JSON strings")
            })
            .collect::<BTreeSet<String>>();

        for problem in Hook0Problem::iter() {
            assert!(
                published.contains(problem.id().as_str()),
                "the served spec does not publish {problem}, it publishes {published:?}"
            );
        }
        assert_eq!(
            published.len(),
            Hook0Problem::iter().count(),
            "the served enumeration holds duplicate or unknown identifiers: {published:?}"
        );
    }

    proptest! {
        /// Whatever payload a problem carries, its identifier stays the one published in the
        /// spec: a client matching on `EventAlreadyIngested` keeps recognizing it even when the
        /// details of the failure change.
        #[test]
        fn problem_identifier_is_non_empty_stable_and_published(problem in any_problem()) {
            let id = problem.id();

            prop_assert!(
                !id.as_str().is_empty(),
                "{problem:?} exposes an empty identifier"
            );
            prop_assert!(
                published_identifiers().contains(id.as_str()),
                "{problem:?} exposes {id}, which the spec does not publish"
            );

            // Compare against the same variant carrying its default payload: the identifier is a
            // property of the variant, never of what it holds.
            let payload_free = Hook0Problem::iter()
                .find(|candidate| discriminant(candidate) == discriminant(&problem));
            match payload_free {
                Some(payload_free) => prop_assert_eq!(
                    id,
                    payload_free.id(),
                    "{:?} exposes an identifier that depends on its payload",
                    problem
                ),
                None => prop_assert!(false, "{:?} is not reachable through variant iteration", problem),
            }
        }
    }

    /// Check the response contract of every error Hook0 can return.
    #[actix_web::test]
    async fn every_problem_response_matches_its_contract() {
        for hook0_problem in Hook0Problem::iter() {
            let expected_status = ProblemDetails::from(hook0_problem.to_owned()).status;
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

    /// The document the real application serves, read once, from a synchronous test.
    fn served_document() -> &'static Value {
        static DOCUMENT: OnceLock<Value> = OnceLock::new();
        DOCUMENT.get_or_init(|| {
            actix_web::rt::System::new().block_on(crate::app::test_support::openapi_spec())
        })
    }

    /// Every operation of a served document, named by method and path.
    ///
    /// An entry of a path item that declares responses is an operation; reading it that way keeps
    /// a list of HTTP methods from having to be maintained here alongside the one the document
    /// already carries.
    fn operations(document: &Value) -> Vec<(String, &Value)> {
        let mut operations = Vec::new();
        for (path, item) in document["paths"].as_object().into_iter().flatten() {
            for (method, operation) in item.as_object().into_iter().flatten() {
                if operation["responses"].is_object() {
                    operations.push((format!("{method} {path}"), operation));
                }
            }
        }
        operations
    }

    /// Every status the API can answer with, taken from the problems themselves rather than
    /// written down a second time, so a problem given a new status is seen here.
    fn statuses_the_api_can_answer() -> BTreeSet<u16> {
        Hook0Problem::iter()
            .map(|problem| ProblemDetails::from(problem).status.as_u16())
            .collect()
    }

    /// Error statuses one operation of a served document declares.
    fn declared_error_statuses(operation: &Value) -> BTreeSet<u16> {
        operation["responses"]
            .as_object()
            .into_iter()
            .flatten()
            .filter_map(|(code, _)| code.parse::<u16>().ok())
            .filter(|code| *code >= 400)
            .collect()
    }

    /// Error statuses a served document declares anywhere.
    fn declared_error_statuses_of_the_document(document: &Value) -> BTreeSet<u16> {
        operations(document)
            .into_iter()
            .flat_map(|(_, operation)| declared_error_statuses(operation))
            .collect()
    }

    /// Members a served document says an error body carries.
    fn documented_problem_members(document: &Value) -> BTreeSet<String> {
        document["components"]["schemas"]["Problem"]["properties"]
            .as_object()
            .expect("the served document defines the Problem schema")
            .keys()
            .cloned()
            .collect()
    }

    /// Members an error body really carries, read off the bytes the response holds so that a
    /// property test can call it without a runtime.
    fn wire_members(problem: &Hook0Problem) -> BTreeSet<String> {
        let Ok(bytes) = problem.error_response().into_body().try_into_bytes() else {
            panic!("{problem} answers a body that is not held in memory");
        };
        serde_json::from_slice::<Value>(&bytes)
            .expect("an error response body is valid JSON")
            .as_object()
            .unwrap_or_else(|| panic!("{problem} answers a body that is not a JSON object"))
            .keys()
            .cloned()
            .collect()
    }

    /// A generated client only handles the statuses its operations declare, and it is generated
    /// from this document alone. The list it is checked against is derived from the problems, so
    /// giving one a status nothing declares fails here rather than at a caller's expense.
    #[actix_web::test]
    async fn every_status_the_api_can_answer_is_declared_by_every_operation() {
        let document = crate::app::test_support::openapi_spec().await;
        let answerable = statuses_the_api_can_answer();

        let operations = operations(&document);
        assert!(
            !operations.is_empty(),
            "the served document declares no operation, so this proves nothing"
        );

        for (name, operation) in operations {
            assert_eq!(
                declared_error_statuses(operation),
                answerable,
                "the error statuses `{name}` declares are not the ones the API can answer with"
            );
        }
    }

    /// The published schema is the whole of what a generated client knows about an error body. It
    /// has to name every member the body carries — and none it does not, which would leave a
    /// client waiting for a field the API never sends.
    #[actix_web::test]
    async fn the_published_problem_schema_names_exactly_the_members_of_the_wire_body() {
        let document = crate::app::test_support::openapi_spec().await;
        let documented = documented_problem_members(&document);

        let required: BTreeSet<String> = document["components"]["schemas"]["Problem"]["required"]
            .as_array()
            .expect("the published Problem schema names its required members")
            .iter()
            .filter_map(|member| member.as_str().map(str::to_owned))
            .collect();
        assert!(
            required.is_subset(&documented),
            "the published Problem schema requires members it does not describe: {:?}",
            required.difference(&documented).collect::<Vec<_>>()
        );

        for problem in Hook0Problem::iter() {
            assert_eq!(
                wire_members(&problem),
                documented,
                "the body of {problem} and the published Problem schema do not carry the same members"
            );
        }
    }

    /// The rate limiters wrap the whole `/api/v1` scope, so their answer is the one error a client
    /// can meet that never passes through `Hook0Problem`'s own response. Rate limiting is also the
    /// error a busy emitter meets most often, so it is the one an SDK can least afford not to
    /// type: it has to arrive as the same body, under the same content type, and to say that the
    /// request is worth sending again.
    ///
    /// Driven by exhausting the quota against the real application, because the middleware is what
    /// is under test.
    #[actix_web::test]
    async fn exhausting_a_rate_limiter_answers_the_documented_problem_body() {
        use crate::app::{build_app, test_support::inert_app_factory_config};
        use crate::rate_limiting::Hook0RateLimiters;
        use actix_web::test;
        use std::net::SocketAddr;

        /// Lets exactly one request through.
        const BURST: u32 = 1;
        /// Holds the next one back long enough that the test cannot race the quota replenishing.
        const REPLENISH_PERIOD_IN_MS: u64 = 3_600_000;

        let document = crate::app::test_support::openapi_spec().await;
        let documented = documented_problem_members(&document);
        let published = published_identifiers();

        // Each limiter is exercised alone, so one still answering the plain-text default of the
        // middleware cannot hide behind another refusing the request first.
        let limiters = [
            (
                "instance-wide",
                Hook0RateLimiters::new(
                    false,
                    false,
                    BURST,
                    REPLENISH_PERIOD_IN_MS,
                    true,
                    BURST,
                    REPLENISH_PERIOD_IN_MS,
                    true,
                    BURST,
                    REPLENISH_PERIOD_IN_MS,
                ),
            ),
            (
                "per-IP",
                Hook0RateLimiters::new(
                    false,
                    true,
                    BURST,
                    REPLENISH_PERIOD_IN_MS,
                    false,
                    BURST,
                    REPLENISH_PERIOD_IN_MS,
                    true,
                    BURST,
                    REPLENISH_PERIOD_IN_MS,
                ),
            ),
        ];

        for (limiter, rate_limiters) in limiters {
            let mut config = inert_app_factory_config().await;
            config.rate_limiters = rate_limiters;
            let app = test::init_service(build_app(&config)).await;

            // `/api/v1/errors` reaches no database, so what the quota lets through stays cheap.
            let request = || {
                test::TestRequest::get()
                    .uri("/api/v1/errors")
                    .peer_addr(SocketAddr::from(([127, 0, 0, 1], 5678)))
                    .to_request()
            };

            let allowed = test::call_service(&app, request()).await;
            assert!(
                allowed.status().is_success(),
                "the {limiter} rate limiter refused the first request with {}, so nothing below is about exceeding a quota",
                allowed.status()
            );

            let refused = test::call_service(&app, request()).await;
            assert_eq!(
                refused.status(),
                StatusCode::TOO_MANY_REQUESTS,
                "the {limiter} rate limiter let a second request through"
            );
            assert_eq!(
                refused
                    .headers()
                    .get(header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok()),
                Some(PROBLEM_JSON_MEDIA_TYPE),
                "the {limiter} rate limiter does not answer a problem document"
            );
            assert!(
                refused.headers().contains_key(header::RETRY_AFTER),
                "the {limiter} rate limiter does not tell the client when to try again"
            );

            let body = test::read_body(refused).await;
            let body = serde_json::from_slice::<Value>(&body)
                .expect("the rate limiter answers a body that is valid JSON");

            let carried: BTreeSet<String> = body
                .as_object()
                .expect("the rate limiter answers a JSON object")
                .keys()
                .cloned()
                .collect();
            assert_eq!(
                carried, documented,
                "the body the {limiter} rate limiter answers does not carry the members the published Problem schema names"
            );

            let id = body["id"]
                .as_str()
                .expect("the rate limiter names the problem it answers");
            assert!(
                published.contains(id),
                "the {limiter} rate limiter answers `{id}`, which the published enumeration does not hold"
            );
            assert_eq!(
                body["status"].as_u64(),
                Some(u64::from(StatusCode::TOO_MANY_REQUESTS.as_u16())),
                "the body the {limiter} rate limiter answers disagrees with its own status"
            );
            let documentation_page = Hook0Problem::iter()
                .find(|problem| problem.id().as_str() == id)
                .unwrap_or_else(|| {
                    panic!("the {limiter} rate limiter answers `{id}`, which is no known problem")
                })
                .id()
                .type_url();
            assert_eq!(
                body["type"].as_str(),
                Some(documentation_page).as_deref(),
                "the {limiter} rate limiter points at the wrong documentation page"
            );
            assert!(
                body["title"]
                    .as_str()
                    .is_some_and(|title| !title.is_empty()),
                "the {limiter} rate limiter answers no title"
            );
            assert!(
                body["detail"]
                    .as_str()
                    .is_some_and(|detail| !detail.is_empty()),
                "the {limiter} rate limiter answers no detail"
            );
        }
    }

    proptest! {
        /// Whatever payload a problem carries, the response it answers stays inside what the
        /// served document promises: a status some operation declares, and exactly the members the
        /// published schema names.
        #[test]
        fn every_problem_answers_a_declared_status_carrying_the_documented_members(problem in any_problem()) {
            let document = served_document();

            let status = problem.error_response().status().as_u16();
            prop_assert!(
                declared_error_statuses_of_the_document(document).contains(&status),
                "{:?} answers {}, which the served document declares nowhere",
                problem,
                status
            );

            prop_assert_eq!(
                wire_members(&problem),
                documented_problem_members(document),
                "the body of {:?} does not carry the members the published Problem schema names",
                problem
            );
        }
    }
}
