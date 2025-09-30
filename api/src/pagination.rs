use actix_web::Responder;
use actix_web::http::header::{HeaderValue, LINK};
use base64::engine::Engine;
use base64::prelude::BASE64_URL_SAFE;
use chrono::{DateTime, Utc};
use paperclip::actix::OperationModifier;
use paperclip::v2::models::{
    DataType, DefaultOperationRaw, DefaultSchemaRaw, Either, Header, Parameter, SecurityScheme,
};
use paperclip::v2::schema::{Apiv2Schema, TypedData};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;
use url::Url;
use uuid::Uuid;

/// A pagination cursor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Cursor {
    pub date: DateTime<Utc>,
    pub id: Uuid,
}

impl Cursor {
    pub fn to_qs_value(self) -> Option<String> {
        serde_json::to_vec(&self)
            .ok()
            .map(|bytes| BASE64_URL_SAFE.encode(bytes))
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

#[derive(Debug, Clone)]
pub struct NextPageParts {
    pub endpoint_url: Url,
    pub qs: Vec<(&'static str, Option<String>)>,
    pub cursor: Cursor,
}

impl NextPageParts {
    pub fn mk_url(mut self) -> Url {
        let filtered_qs = self
            .qs
            .into_iter()
            .chain([("pagination_cursor", self.cursor.to_qs_value())])
            .filter_map(|(k, v_opt)| v_opt.map(|v| (k, v)));
        self.endpoint_url
            .query_pairs_mut()
            .extend_pairs(filtered_qs);
        self.endpoint_url
    }
}

/// Wrapper arround any Actix Web responder to add a header containing the next pagination cursor
#[derive(Debug, Clone)]
pub struct Paginated<T: Apiv2Schema + OperationModifier + Responder> {
    pub data: T,
    pub next_page_parts: Option<NextPageParts>,
}

impl<T: Apiv2Schema + OperationModifier + Responder> Responder for Paginated<T> {
    type Body = T::Body;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let mut res = self.data.respond_to(req);

        if let Some(link_hv) = self.next_page_parts.and_then(|parts| {
            HeaderValue::from_str(&format!(r#"<{}>; rel="next""#, parts.mk_url())).ok()
        }) {
            let headers = res.headers_mut();
            headers.insert(LINK, link_hv);
        }

        res
    }
}

impl<T: Apiv2Schema + OperationModifier + Responder> Apiv2Schema for Paginated<T> {
    fn name() -> Option<String> {
        T::name()
    }

    fn description() -> &'static str {
        T::description()
    }

    fn required() -> bool {
        T::required()
    }

    fn raw_schema() -> DefaultSchemaRaw {
        T::raw_schema()
    }

    fn schema_with_ref() -> DefaultSchemaRaw {
        T::schema_with_ref()
    }

    fn security_scheme() -> Option<SecurityScheme> {
        T::security_scheme()
    }

    fn header_parameter_schema() -> Vec<Parameter<DefaultSchemaRaw>> {
        T::header_parameter_schema()
    }
}

impl<T: Apiv2Schema + OperationModifier + Responder> OperationModifier for Paginated<T> {
    fn update_parameter(op: &mut DefaultOperationRaw) {
        T::update_parameter(op);
    }

    fn update_response(_op: &mut DefaultOperationRaw) {
        T::update_response(_op);
        if let Some(Either::Right(res)) = _op.responses.get_mut("200") {
            res.headers.insert(
                LINK.to_string(),
                Header {
                    description: Some(r#"Value following HATEOAS conventions; for example: \<https://hook0_domain/endpoint?pagination_cursor=SOME_VALUE\>; rel="next""#.to_owned()),
                    data_type: Some(DataType::String),
                    format: None,
                    items: None,
                    collection_format: None,
                    default: None,
                    enum_: Vec::new(),
                    maximum: None,
                    exclusive_maximum: None,
                    minimum: None,
                    exclusive_minimum: None,
                    max_length: None,
                    min_length: None,
                    pattern: None,
                    max_items: None,
                    min_items: None,
                    unique_items: None,
                    multiple_of: None,
                },
            );
        }
    }

    fn update_definitions(map: &mut BTreeMap<String, DefaultSchemaRaw>) {
        T::update_definitions(map);
    }

    fn update_security(op: &mut DefaultOperationRaw) {
        T::update_security(op);
    }

    fn update_security_definitions(map: &mut std::collections::BTreeMap<String, SecurityScheme>) {
        T::update_security_definitions(map);
    }
}

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
        let encoded = cursor.to_qs_value().unwrap();
        let decoded = EncodedDescCursor::from_str(&encoded).unwrap();
        assert_eq!(decoded.0, cursor)
    }

    #[test]
    fn next_page_url() {
        let next_page_parts = NextPageParts {
            endpoint_url: Url::parse("https://test.local/endpoint").unwrap(),
            qs: vec![
                ("k1", Some("v1".to_owned())),
                ("k2", None),
                ("k3", Some("v3".to_owned())),
            ],
            cursor: Cursor {
                date: DateTime::UNIX_EPOCH,
                id: Uuid::nil(),
            },
        };
        let expected = Url::parse("https://test.local/endpoint?k1=v1&k3=v3&pagination_cursor=eyJkYXRlIjoiMTk3MC0wMS0wMVQwMDowMDowMFoiLCJpZCI6IjAwMDAwMDAwLTAwMDAtMDAwMC0wMDAwLTAwMDAwMDAwMDAwMCJ9").unwrap();
        assert_eq!(next_page_parts.mk_url(), expected)
    }
}
