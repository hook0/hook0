use actix_web::Responder;
use actix_web::http::header::{HeaderValue, LINK};
use base64::engine::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use paperclip::actix::OperationModifier;
use paperclip::v2::models::{DefaultOperationRaw, DefaultSchemaRaw, Parameter, SecurityScheme};
use paperclip::v2::schema::{Apiv2Schema, TypedData};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;
use tracing::error;
use url::Url;
use uuid::Uuid;

pub const PARAM_CURSOR: &str = "pagination_cursor";

/// Page size limit for a paginated endpoint.
/// `fetch_limit()` returns `size + 1` as `i64` to bind as PostgreSQL BIGINT.
#[derive(Debug, Clone, Copy)]
pub struct PageLimit {
    pub size: usize,
}

impl PageLimit {
    pub const fn new(size: usize) -> Self {
        Self { size }
    }

    pub const fn fetch_limit(self) -> i64 {
        (self.size as i64) + 1
    }
}

impl Default for PageLimit {
    fn default() -> Self {
        Self { size: 20 }
    }
}

/// Builds the absolute endpoint URL used for pagination Link headers.
/// Normalizes `app_url` with a trailing `/` first so relative paths resolve
/// against the full base. Returns `None` and logs if any step fails (which
/// should not happen for statically-known API paths).
pub fn build_endpoint_url(app_url: &Url, path: &str) -> Option<Url> {
    let base = if app_url.as_str().ends_with('/') {
        app_url.clone()
    } else {
        match Url::parse(&format!("{app_url}/")) {
            Ok(u) => u,
            Err(e) => {
                error!("Failed to normalize app_url with trailing slash: {e}");
                return None;
            }
        }
    };

    base.join(path)
        .inspect_err(|e| error!("Failed to build pagination endpoint URL: {e}"))
        .ok()
}

/// Per-request pagination state: what page size, which cursor the caller sent.
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub limit: PageLimit,
    pub cursor: Option<Cursor>,
}

impl Pagination {
    pub fn new(limit: PageLimit, cursor: Option<EncodedCursor>) -> Self {
        Self {
            limit,
            cursor: cursor.map(|c| c.0),
        }
    }

    pub fn is_backward(&self) -> bool {
        self.cursor
            .is_some_and(|c| c.direction == PaginationDirection::Backward)
    }

    /// Returns the caller's cursor or the first-page sentinel when absent.
    pub fn resolved_cursor(&self) -> Cursor {
        self.cursor.unwrap_or_else(Cursor::first_page_sentinel)
    }

    pub fn fetch_limit(&self) -> i64 {
        self.limit.fetch_limit()
    }

    /// Trims the over-fetched overshoot row and reverses if backward.
    /// Returns `has_more` — true when the discarded overshoot row existed.
    pub fn trim_and_orient<T>(&self, items: &mut Vec<T>) -> bool {
        let has_more = items.len() > self.limit.size;
        if has_more {
            items.truncate(self.limit.size);
        }
        if self.is_backward() {
            items.reverse();
        }
        has_more
    }

    /// Builds next/prev `PageParts` for the response Link header.
    /// `key_fn` extracts the `(created_at, id)` tuple used by the SQL ORDER BY.
    pub fn build_page_parts<T, F>(
        &self,
        items: &[T],
        endpoint_url: Option<Url>,
        query_params: Vec<(&'static str, String)>,
        has_more: bool,
        key_fn: F,
    ) -> (Option<PageParts>, Option<PageParts>)
    where
        F: Fn(&T) -> (DateTime<Utc>, Uuid),
    {
        BidirectionalPageConfig {
            endpoint_url,
            query_params,
            first_row_key: items.first().map(&key_fn),
            last_row_key: items.last().map(&key_fn),
            cursor: self.cursor,
            has_more,
        }
        .into_page_parts()
    }
}

/// Errors from cursor decoding/encoding.
#[derive(Debug, Error)]
pub enum CursorError {
    #[error("invalid base64: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("invalid cursor JSON: {0}")]
    Json(#[from] serde_json::Error),
}

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

/// Position in the paginated sequence + which way to navigate from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Cursor {
    pub date: DateTime<Utc>,
    pub id: Uuid,
    pub direction: PaginationDirection,
}

impl Cursor {
    pub fn forward(date: DateTime<Utc>, id: Uuid) -> Self {
        Self {
            date,
            id,
            direction: PaginationDirection::Forward,
        }
    }

    pub fn backward(date: DateTime<Utc>, id: Uuid) -> Self {
        Self {
            date,
            id,
            direction: PaginationDirection::Backward,
        }
    }

    /// Sentinel used as the implicit cursor for the first forward page
    /// when the caller did not send one.
    /// Uses `Uuid::max()` so rows inserted at the exact sentinel timestamp
    /// still satisfy `(created_at, id) < (sentinel_date, sentinel_id)`.
    pub fn first_page_sentinel() -> Self {
        Self::forward(Utc::now(), Uuid::max())
    }

    /// Serializes to base64 for use in URL query strings.
    pub fn to_qs_value(self) -> Result<String, CursorError> {
        let bytes = serde_json::to_vec(&self)?;
        Ok(BASE64_URL_SAFE_NO_PAD.encode(bytes))
    }

    fn decode_from_base64(s: &str) -> Result<Self, CursorError> {
        let bytes = BASE64_URL_SAFE_NO_PAD.decode(s)?;
        Ok(serde_json::from_slice::<Cursor>(&bytes)?)
    }
}

/// Base64-encoded cursor for URL query strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodedCursor(pub Cursor);

impl TypedData for EncodedCursor {
    fn data_type() -> paperclip::v2::models::DataType {
        paperclip::v2::models::DataType::String
    }
}

impl FromStr for EncodedCursor {
    type Err = CursorError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Cursor::decode_from_base64(s).map(Self)
    }
}

impl<'de> Deserialize<'de> for EncodedCursor {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = EncodedCursor;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a base64-encoded cursor string")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                EncodedCursor::from_str(v).map_err(E::custom)
            }
        }
        d.deserialize_str(Visitor)
    }
}

/// URL parts for one pagination link.
/// Query params carry only present values; filter `None`s at the caller.
#[derive(Debug, Clone)]
pub struct PageParts {
    pub endpoint_url: Url,
    pub query_params: Vec<(&'static str, String)>,
    pub cursor: Cursor,
}

impl PageParts {
    pub fn mk_url(mut self) -> Result<Url, CursorError> {
        let cursor_str = self.cursor.to_qs_value()?;

        // Scope the mutable borrow so `self.endpoint_url` can be moved out below.
        {
            let mut pairs = self.endpoint_url.query_pairs_mut();
            for (key, value) in self.query_params {
                pairs.append_pair(key, &value);
            }
            pairs.append_pair(PARAM_CURSOR, &cursor_str);
        }

        Ok(self.endpoint_url)
    }
}

/// Config for bidirectional cursor pagination link building.
/// The caller's cursor (if any) tells us current direction and whether
/// we are past the first page — both are load-bearing for show_next/show_prev.
/// `endpoint_url` is `None` when the caller failed to build it; both links
/// are then returned as `None`.
pub struct BidirectionalPageConfig {
    pub endpoint_url: Option<Url>,
    pub query_params: Vec<(&'static str, String)>,
    pub first_row_key: Option<(DateTime<Utc>, Uuid)>,
    pub last_row_key: Option<(DateTime<Utc>, Uuid)>,
    pub cursor: Option<Cursor>,
    pub has_more: bool,
}

impl BidirectionalPageConfig {
    /// Returns (next_page, prev_page) link parts.
    pub fn into_page_parts(self) -> (Option<PageParts>, Option<PageParts>) {
        let Some(endpoint_url) = self.endpoint_url else {
            return (None, None);
        };

        let is_backward = self
            .cursor
            .is_some_and(|c| c.direction == PaginationDirection::Backward);
        let is_past_first_page = self.cursor.is_some();

        let show_next = self.has_more || is_backward;
        let show_prev = if is_backward {
            self.has_more
        } else {
            is_past_first_page
        };

        match (show_next, show_prev) {
            (true, true) => (
                self.last_row_key.map(|(date, id)| PageParts {
                    endpoint_url: endpoint_url.clone(),
                    query_params: self.query_params.clone(),
                    cursor: Cursor::forward(date, id),
                }),
                self.first_row_key.map(|(date, id)| PageParts {
                    endpoint_url,
                    query_params: self.query_params,
                    cursor: Cursor::backward(date, id),
                }),
            ),
            (true, false) => (
                self.last_row_key.map(|(date, id)| PageParts {
                    endpoint_url,
                    query_params: self.query_params,
                    cursor: Cursor::forward(date, id),
                }),
                None,
            ),
            (false, true) => (
                None,
                self.first_row_key.map(|(date, id)| PageParts {
                    endpoint_url,
                    query_params: self.query_params,
                    cursor: Cursor::backward(date, id),
                }),
            ),
            (false, false) => (None, None),
        }
    }
}

/// Adds Link headers for cursor-based pagination.
#[derive(Debug, Clone)]
pub struct Paginated<T: Apiv2Schema + OperationModifier + Responder> {
    pub data: T,
    pub next_page_parts: Option<PageParts>,
    pub prev_page_parts: Option<PageParts>,
}

impl<T: Apiv2Schema + OperationModifier + Responder> Paginated<T> {
    fn build_link(parts: Option<PageParts>, rel: &str) -> Option<String> {
        let parts = parts?;
        match parts.mk_url() {
            Ok(url) => Some(format!(r#"<{url}>; rel="{rel}""#)),
            Err(e) => {
                error!("Failed to build pagination Link header for rel=\"{rel}\": {e}");
                None
            }
        }
    }
}

impl<T: Apiv2Schema + OperationModifier + Responder> Responder for Paginated<T> {
    type Body = T::Body;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let mut res = self.data.respond_to(req);

        let next_link = Self::build_link(self.next_page_parts, "next");
        let prev_link = Self::build_link(self.prev_page_parts, "prev");

        // RFC 8288: comma-separated Link values.
        // Cursor blob encodes direction; callers just follow the link.
        let combined = match (prev_link, next_link) {
            (Some(prev), Some(next)) => Some(format!("{prev}, {next}")),
            (Some(prev), None) => Some(prev),
            (None, Some(next)) => Some(next),
            (None, None) => None,
        };

        if let Some(link_value) = combined {
            match HeaderValue::from_str(&link_value) {
                Ok(hv) => {
                    res.headers_mut().insert(LINK, hv);
                }
                Err(e) => {
                    error!("Failed to parse Link header value: {e}");
                }
            }
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

    /// Swagger/OpenAPI v2 supports response headers, but Paperclip's raw AST
    /// mutation for this is verbose. We intentionally skip registering the
    /// `Link` header here; clients rely on the documented pagination
    /// convention out-of-band.
    fn update_response(op: &mut DefaultOperationRaw) {
        T::update_response(op);
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
        let cursor = Cursor::forward(
            Utc.with_ymd_and_hms(2025, 9, 28, 18, 0, 0).unwrap(),
            uuid!("8f27f238-ed88-4330-927f-0d20796da285"),
        );
        let encoded = cursor.to_qs_value().unwrap();
        let decoded = EncodedCursor::from_str(&encoded).unwrap();
        assert_eq!(decoded.0, cursor)
    }

    #[test]
    fn page_url_contains_cursor() {
        let parts = PageParts {
            endpoint_url: Url::parse("https://test.local/endpoint").unwrap(),
            query_params: vec![("k1", "v1".to_owned())],
            cursor: Cursor::forward(DateTime::UNIX_EPOCH, Uuid::nil()),
        };
        let url = parts.mk_url().unwrap();
        assert!(url.as_str().contains("pagination_cursor="));
        assert!(url.as_str().contains("k1=v1"));
    }

    #[test]
    fn paginated_both_links() {
        let next = PageParts {
            endpoint_url: Url::parse("https://test.local/items").unwrap(),
            query_params: vec![],
            cursor: Cursor::forward(DateTime::UNIX_EPOCH, Uuid::nil()),
        };
        let prev = PageParts {
            endpoint_url: Url::parse("https://test.local/items").unwrap(),
            query_params: vec![],
            cursor: Cursor::backward(DateTime::UNIX_EPOCH, Uuid::nil()),
        };

        let next_url = next.clone().mk_url().unwrap();
        let prev_url = prev.clone().mk_url().unwrap();

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
