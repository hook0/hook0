use std::str::FromStr;

use actix_web::Responder;
use actix_web::http::header::{HeaderName, HeaderValue};
use base64::engine::Engine;
use base64::prelude::BASE64_URL_SAFE;
use chrono::{DateTime, Utc};
use paperclip::actix::OperationModifier;
use paperclip::v2::schema::{Apiv2Schema, TypedData};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

const NEXT_CURSOR_HEADER_NAME: HeaderName = HeaderName::from_static("x-pagination-cursor");

/// A pagination cursor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Cursor {
    pub date: DateTime<Utc>,
    pub id: Uuid,
}

impl Cursor {
    pub fn to_header_value(self) -> Option<HeaderValue> {
        serde_json::to_vec(&self)
            .ok()
            .map(|bytes| BASE64_URL_SAFE.encode(bytes))
            .and_then(|b64| HeaderValue::from_str(&b64).ok())
    }
}

/// Wrapper arround [`Cursor`] to implement the correct traits for it to be decoded fron base64 and correctly documented in OpenAPI spec
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodedDescCursor(pub Cursor);

impl Default for EncodedDescCursor {
    fn default() -> Self {
        Self(Cursor {
            date: Utc::now(),
            id: Uuid::nil(),
        })
    }
}

impl TypedData for EncodedDescCursor {
    fn data_type() -> paperclip::v2::models::DataType {
        paperclip::v2::models::DataType::String
    }
}

impl FromStr for EncodedDescCursor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BASE64_URL_SAFE
            .decode(s)
            .map_err(|e| e.to_string())
            .and_then(|bytes| {
                serde_json::from_slice::<Cursor>(&bytes)
                    .map(Self)
                    .map_err(|e| e.to_string())
            })
    }
}

impl<'de> Deserialize<'de> for EncodedDescCursor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Wrapper arround any Actix Web responder to add a header containing the next pagination cursor
#[derive(Debug, Clone)]
pub struct Paginated<T: Apiv2Schema + OperationModifier + Responder>(pub T, pub Option<Cursor>);

impl<T: Apiv2Schema + OperationModifier + Responder> Responder for Paginated<T> {
    type Body = T::Body;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let mut res = self.0.respond_to(req);
        if let Some(cursor) = self.1.and_then(|c| c.to_header_value()) {
            let headers = res.headers_mut();
            headers.insert(NEXT_CURSOR_HEADER_NAME, cursor);
        }
        res
    }
}

impl<T: Apiv2Schema + OperationModifier + Responder> Apiv2Schema for Paginated<T> {}

impl<T: Apiv2Schema + OperationModifier + Responder> OperationModifier for Paginated<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::TimeZone;
    use uuid::uuid;

    #[test]
    fn encode_and_decode_cursor() {
        let cursor = Cursor {
            date: Utc.with_ymd_and_hms(2025, 9, 28, 18, 0, 0).unwrap(),
            id: uuid!("8f27f238-ed88-4330-927f-0d20796da285"),
        };
        let encoded = cursor.to_header_value().unwrap();
        let decoded = EncodedDescCursor::from_str(encoded.to_str().unwrap()).unwrap();
        assert_eq!(decoded.0, cursor)
    }
}
