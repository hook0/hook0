use actix_web::Responder;
use actix_web::http::header::{HeaderValue, LINK};
use base64::engine::Engine;
use base64::prelude::BASE64_URL_SAFE;
use chrono::{DateTime, Utc};
use paperclip::actix::OperationModifier;
use paperclip::v2::models::{DefaultOperationRaw, DefaultSchemaRaw, Parameter, SecurityScheme};
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

/// Wrapper for [`Cursor`] decoded from base64, descending order
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

/// Wrapper for [`Cursor`] decoded from base64, ascending order
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodedAscCursor(pub Cursor);

impl Default for EncodedAscCursor {
    fn default() -> Self {
        Self(Cursor {
            date: DateTime::<Utc>::MIN_UTC,
            id: Uuid::nil(),
        })
    }
}

impl TypedData for EncodedAscCursor {
    fn data_type() -> paperclip::v2::models::DataType {
        paperclip::v2::models::DataType::String
    }
}

impl FromStr for EncodedAscCursor {
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

impl<'de> Deserialize<'de> for EncodedAscCursor {
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

#[derive(Debug, Clone)]
pub struct PrevPageParts {
    pub endpoint_url: Url,
    pub qs: Vec<(&'static str, Option<String>)>,
    pub cursor: Cursor,
}

impl PrevPageParts {
    pub fn mk_url(mut self) -> Url {
        let filtered_qs = self
            .qs
            .into_iter()
            .chain([("pagination_before_cursor", self.cursor.to_qs_value())])
            .filter_map(|(k, v_opt)| v_opt.map(|v| (k, v)));
        self.endpoint_url
            .query_pairs_mut()
            .extend_pairs(filtered_qs);
        self.endpoint_url
    }
}

/// Adds Link headers for cursor-based pagination
#[derive(Debug, Clone)]
pub struct Paginated<T: Apiv2Schema + OperationModifier + Responder> {
    pub data: T,
    pub next_page_parts: Option<NextPageParts>,
    pub prev_page_parts: Option<PrevPageParts>,
}

impl<T: Apiv2Schema + OperationModifier + Responder> Responder for Paginated<T> {
    type Body = T::Body;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let mut res = self.data.respond_to(req);

        let next_link = self
            .next_page_parts
            .map(|parts| format!(r#"<{}>; rel="next""#, parts.mk_url()));
        let prev_link = self
            .prev_page_parts
            .map(|parts| format!(r#"<{}>; rel="prev""#, parts.mk_url()));

        // RFC 8288: comma-separated Link values
        let combined = match (prev_link, next_link) {
            (Some(prev), Some(next)) => Some(format!("{prev}, {next}")),
            (Some(prev), None) => Some(prev),
            (None, Some(next)) => Some(next),
            (None, None) => None,
        };

        if let Some(link_value) = combined
            && let Ok(hv) = HeaderValue::from_str(&link_value)
        {
            res.headers_mut().insert(LINK, hv);
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

        // Paperclip currently does not handle headers in responses
        // if let Some(Either::Right(res)) = _op.responses.get_mut("200") {
        //     res.headers.insert(
        //         LINK.to_string(),
        //         Header {
        //             description: Some(r#"Value following HATEOAS conventions; for example: \<https://hook0_domain/endpoint?pagination_cursor=SOME_VALUE\>; rel="next""#.to_owned()),
        //             data_type: Some(DataType::String),
        //             ..Default::default()
        //         },
        //     );
        // }
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

    #[test]
    fn prev_page_url() {
        let prev_page_parts = PrevPageParts {
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
        let expected = Url::parse("https://test.local/endpoint?k1=v1&k3=v3&pagination_before_cursor=eyJkYXRlIjoiMTk3MC0wMS0wMVQwMDowMDowMFoiLCJpZCI6IjAwMDAwMDAwLTAwMDAtMDAwMC0wMDAwLTAwMDAwMDAwMDAwMCJ9").unwrap();
        assert_eq!(prev_page_parts.mk_url(), expected)
    }

    #[test]
    fn encode_and_decode_asc_cursor() {
        let cursor = Cursor {
            date: Utc.with_ymd_and_hms(2025, 9, 28, 18, 0, 0).unwrap(),
            id: uuid!("8f27f238-ed88-4330-927f-0d20796da285"),
        };
        let encoded = cursor.to_qs_value().unwrap();
        let decoded = EncodedAscCursor::from_str(&encoded).unwrap();
        assert_eq!(decoded.0, cursor)
    }

    #[test]
    fn paginated_both_links() {
        let cursor = Cursor {
            date: DateTime::UNIX_EPOCH,
            id: Uuid::nil(),
        };
        let qs: Vec<(&'static str, Option<String>)> = vec![];

        let next = NextPageParts {
            endpoint_url: Url::parse("https://test.local/items").unwrap(),
            qs: qs.clone(),
            cursor,
        };
        let prev = PrevPageParts {
            endpoint_url: Url::parse("https://test.local/items").unwrap(),
            qs,
            cursor,
        };

        let next_url = next.clone().mk_url();
        let prev_url = prev.clone().mk_url();

        let paginated = Paginated {
            data: actix_web::web::Json(Vec::<String>::new()),
            next_page_parts: Some(next),
            prev_page_parts: Some(prev),
        };

        let req = actix_web::test::TestRequest::default().to_http_request();
        let res = paginated.respond_to(&req);
        let link = res.headers().get(LINK).unwrap().to_str().unwrap();

        let expected = format!(r#"<{prev_url}>; rel="prev", <{next_url}>; rel="next""#);
        assert_eq!(link, expected);
    }
}
