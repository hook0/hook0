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

pub const PARAM_CURSOR: &str = "pagination_cursor";
pub const PARAM_DIRECTION: &str = "pagination_direction";

/// Direction for bidirectional cursor pagination.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Deserialize,
    Serialize,
    paperclip::actix::Apiv2Schema,
)]
#[serde(rename_all = "snake_case")]
pub enum PaginationDirection {
    #[default]
    Forward,
    Backward,
}

impl PaginationDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Forward => "forward",
            Self::Backward => "backward",
        }
    }
}

/// A pagination cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Cursor {
    pub date: DateTime<Utc>,
    pub id: Uuid,
}

impl Cursor {
    /// Serializes to base64 for use in URL query strings.
    /// DateTime + UUID serialization cannot fail.
    pub fn to_qs_value(self) -> String {
        let bytes = serde_json::to_vec(&self).expect("Cursor JSON serialization cannot fail");
        BASE64_URL_SAFE.encode(bytes)
    }

    fn decode_from_base64(s: &str) -> Result<Self, String> {
        let bytes = BASE64_URL_SAFE
            .decode(s)
            .map_err(|e| format!("invalid base64: {e}"))?;
        serde_json::from_slice::<Cursor>(&bytes).map_err(|e| format!("invalid cursor JSON: {e}"))
    }
}

/// Base64-encoded cursor for URL query strings.
/// Direction is carried separately via [`PaginationDirection`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodedCursor(pub Cursor);

impl Default for EncodedCursor {
    fn default() -> Self {
        Self(Cursor {
            date: Utc::now(),
            id: Uuid::nil(),
        })
    }
}

impl TypedData for EncodedCursor {
    fn data_type() -> paperclip::v2::models::DataType {
        paperclip::v2::models::DataType::String
    }
}

impl FromStr for EncodedCursor {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Cursor::decode_from_base64(s).map(Self)
    }
}

impl<'de> Deserialize<'de> for EncodedCursor {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// URL parts for one pagination link.
#[derive(Debug, Clone)]
pub struct PageParts {
    pub endpoint_url: Url,
    pub query_params: Vec<(&'static str, Option<String>)>,
    pub cursor: Cursor,
    pub direction: PaginationDirection,
}

impl PageParts {
    /// Builds the final URL with query params, cursor, and direction.
    pub fn mk_url(mut self) -> Url {
        let mut pairs = self.endpoint_url.query_pairs_mut();

        for (key, value_opt) in self.query_params {
            if let Some(value) = value_opt {
                pairs.append_pair(key, &value);
            }
        }

        pairs.append_pair(PARAM_CURSOR, &self.cursor.to_qs_value());
        pairs.append_pair(PARAM_DIRECTION, self.direction.as_str());

        drop(pairs);
        self.endpoint_url
    }
}

/// Config for bidirectional cursor pagination link building.
pub struct BidirectionalPageConfig {
    pub endpoint_url: Url,
    pub query_params: Vec<(&'static str, Option<String>)>,
    pub next_cursor: Option<Cursor>,
    pub prev_cursor: Option<Cursor>,
    pub direction: PaginationDirection,
    pub has_more: bool,
    pub is_past_first_page: bool,
}

impl BidirectionalPageConfig {
    /// Returns (next_page, prev_page) link parts.
    pub fn into_page_parts(self) -> (Option<PageParts>, Option<PageParts>) {
        let is_backward = self.direction == PaginationDirection::Backward;
        let show_next = self.has_more || is_backward;
        let show_prev = if is_backward {
            self.has_more
        } else {
            self.is_past_first_page
        };

        let next_page = if show_next {
            self.next_cursor.map(|cursor| PageParts {
                endpoint_url: self.endpoint_url.clone(),
                query_params: self.query_params.clone(),
                cursor,
                direction: PaginationDirection::Forward,
            })
        } else {
            None
        };

        let prev_page = if show_prev {
            self.prev_cursor.map(|cursor| PageParts {
                endpoint_url: self.endpoint_url,
                query_params: self.query_params,
                cursor,
                direction: PaginationDirection::Backward,
            })
        } else {
            None
        };

        (next_page, prev_page)
    }
}

/// Adds Link headers for cursor-based pagination.
#[derive(Debug, Clone)]
pub struct Paginated<T: Apiv2Schema + OperationModifier + Responder> {
    pub data: T,
    pub next_page_parts: Option<PageParts>,
    pub prev_page_parts: Option<PageParts>,
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
        // next only: Link: <.../ep?pagination_cursor=abc&pagination_direction=forward>; rel="next"
        // prev only: Link: <.../ep?pagination_cursor=def&pagination_direction=backward>; rel="prev"
        // both:      Link: <...>; rel="prev", <...>; rel="next"
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
        let encoded = cursor.to_qs_value();
        let decoded = EncodedCursor::from_str(&encoded).unwrap();
        assert_eq!(decoded.0, cursor)
    }

    #[test]
    fn forward_page_url() {
        let parts = PageParts {
            endpoint_url: Url::parse("https://test.local/endpoint").unwrap(),
            query_params: vec![
                ("k1", Some("v1".to_owned())),
                ("k2", None),
                ("k3", Some("v3".to_owned())),
            ],
            cursor: Cursor {
                date: DateTime::UNIX_EPOCH,
                id: Uuid::nil(),
            },
            direction: PaginationDirection::Forward,
        };
        let url = parts.mk_url();
        assert!(url.as_str().contains("pagination_cursor="));
        assert!(url.as_str().contains("pagination_direction=forward"));
        assert!(url.as_str().contains("k1=v1"));
        assert!(url.as_str().contains("k3=v3"));
    }

    #[test]
    fn backward_page_url() {
        let parts = PageParts {
            endpoint_url: Url::parse("https://test.local/endpoint").unwrap(),
            query_params: vec![],
            cursor: Cursor {
                date: DateTime::UNIX_EPOCH,
                id: Uuid::nil(),
            },
            direction: PaginationDirection::Backward,
        };
        let url = parts.mk_url();
        assert!(url.as_str().contains("pagination_direction=backward"));
    }

    #[test]
    fn paginated_both_links() {
        let cursor = Cursor {
            date: DateTime::UNIX_EPOCH,
            id: Uuid::nil(),
        };
        let qs: Vec<(&'static str, Option<String>)> = vec![];

        let next = PageParts {
            endpoint_url: Url::parse("https://test.local/items").unwrap(),
            query_params: qs.clone(),
            cursor,
            direction: PaginationDirection::Forward,
        };
        let prev = PageParts {
            endpoint_url: Url::parse("https://test.local/items").unwrap(),
            query_params: qs,
            cursor,
            direction: PaginationDirection::Backward,
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
