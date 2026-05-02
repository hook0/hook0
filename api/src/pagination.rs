use actix_web::Responder;
use actix_web::http::header::{HeaderValue, LINK};
use base64::engine::Engine;
// We use `BASE64_URL_SAFE` (with `=` padding) for cursor encoding so cursor URLs
// already in flight from older clients keep decoding. Switching to NO_PAD requires
// a coordinated wire-format migration with all consumers (request_attempts SDK).
use base64::prelude::BASE64_URL_SAFE;
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

    /// Bounded constructor used by handlers that accept a client-supplied `limit`
    /// query parameter. Enforces `1 <= size <= MAX_LIMIT`. Returns
    /// `PageLimitError` on out-of-range, which handlers map to HTTP 400.
    pub const fn try_new(size: usize) -> Result<Self, PageLimitError> {
        if size == 0 {
            Err(PageLimitError::TooSmall)
        } else if size > Self::MAX_LIMIT {
            Err(PageLimitError::TooLarge)
        } else {
            Ok(Self { size })
        }
    }

    /// Hard ceiling for a single page across all paginated endpoints.
    /// Matches the previously-published doc `max=100` and the historical
    /// hardcoded `LIMIT 100` on `events::list`.
    pub const MAX_LIMIT: usize = 100;

    pub const fn fetch_limit(self) -> i64 {
        (self.size as i64) + 1
    }
}

impl Default for PageLimit {
    fn default() -> Self {
        Self { size: 20 }
    }
}

/// Bounds-violation error for `PageLimit::try_new`. Handlers convert this
/// to `Hook0Problem::BadRequest`-style 400 with a clear message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PageLimitError {
    #[error("`limit` must be >= 1")]
    TooSmall,
    #[error("`limit` must be <= 100")]
    TooLarge,
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
        Ok(BASE64_URL_SAFE.encode(bytes))
    }

    fn decode_from_base64(s: &str) -> Result<Self, CursorError> {
        let bytes = BASE64_URL_SAFE.decode(s)?;
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

// ============================================================================
// Name-tiebreak cursor variant — used by endpoints whose primary key is a
// `(application_id, name: TEXT)` composite (e.g., `event.event_type`). Mirrors
// the Uuid-tiebreak machinery above but with `name: String` for the keyset
// comparison `(created_at, name) < ($N, $N+1)`.
// ============================================================================

/// Position cursor for endpoints with a string (TEXT) tiebreak instead of a Uuid.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct NameCursor {
    pub date: DateTime<Utc>,
    pub name: String,
    pub direction: PaginationDirection,
}

impl NameCursor {
    pub fn forward(date: DateTime<Utc>, name: String) -> Self {
        Self {
            date,
            name,
            direction: PaginationDirection::Forward,
        }
    }

    pub fn backward(date: DateTime<Utc>, name: String) -> Self {
        Self {
            date,
            name,
            direction: PaginationDirection::Backward,
        }
    }

    /// Sentinel for the implicit first forward page when no cursor is supplied.
    /// Uses `Utc::now()` plus the highest reasonable string sentinel `\u{10FFFF}`
    /// so any real `(created_at, name)` row satisfies the `<` keyset predicate.
    pub fn first_page_sentinel() -> Self {
        Self::forward(Utc::now(), String::from('\u{10FFFF}'))
    }

    pub fn to_qs_value(&self) -> Result<String, CursorError> {
        let bytes = serde_json::to_vec(&self)?;
        Ok(BASE64_URL_SAFE.encode(bytes))
    }

    fn decode_from_base64(s: &str) -> Result<Self, CursorError> {
        let bytes = BASE64_URL_SAFE.decode(s)?;
        Ok(serde_json::from_slice::<NameCursor>(&bytes)?)
    }
}

/// Base64-encoded `NameCursor` for URL query strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedNameCursor(pub NameCursor);

impl TypedData for EncodedNameCursor {
    fn data_type() -> paperclip::v2::models::DataType {
        paperclip::v2::models::DataType::String
    }
}

impl FromStr for EncodedNameCursor {
    type Err = CursorError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NameCursor::decode_from_base64(s).map(Self)
    }
}

impl<'de> Deserialize<'de> for EncodedNameCursor {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = EncodedNameCursor;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a base64-encoded name-cursor string")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                EncodedNameCursor::from_str(v).map_err(E::custom)
            }
        }
        d.deserialize_str(Visitor)
    }
}

/// URL parts for one pagination link with a `NameCursor`.
#[derive(Debug, Clone)]
pub struct NamePageParts {
    pub endpoint_url: Url,
    pub query_params: Vec<(&'static str, String)>,
    pub cursor: NameCursor,
}

impl NamePageParts {
    pub fn mk_url(mut self) -> Result<Url, CursorError> {
        let cursor_str = self.cursor.to_qs_value()?;
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

/// Per-request pagination state for name-tiebreak endpoints.
#[derive(Debug, Clone)]
pub struct NamePagination {
    pub limit: PageLimit,
    pub cursor: Option<NameCursor>,
}

impl NamePagination {
    pub fn new(limit: PageLimit, cursor: Option<EncodedNameCursor>) -> Self {
        Self {
            limit,
            cursor: cursor.map(|c| c.0),
        }
    }

    pub fn is_backward(&self) -> bool {
        self.cursor
            .as_ref()
            .is_some_and(|c| c.direction == PaginationDirection::Backward)
    }

    pub fn resolved_cursor(&self) -> NameCursor {
        self.cursor
            .clone()
            .unwrap_or_else(NameCursor::first_page_sentinel)
    }

    pub fn fetch_limit(&self) -> i64 {
        self.limit.fetch_limit()
    }

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

    /// Builds next/prev `NamePageParts` for the response Link header.
    /// `key_fn` extracts the `(created_at, name)` tuple used by the SQL ORDER BY.
    pub fn build_page_parts<T, F>(
        &self,
        items: &[T],
        endpoint_url: Option<Url>,
        query_params: Vec<(&'static str, String)>,
        has_more: bool,
        key_fn: F,
    ) -> (Option<NamePageParts>, Option<NamePageParts>)
    where
        F: Fn(&T) -> (DateTime<Utc>, String),
    {
        NameBidirectionalPageConfig {
            endpoint_url,
            query_params,
            first_row_key: items.first().map(&key_fn),
            last_row_key: items.last().map(&key_fn),
            cursor: self.cursor.clone(),
            has_more,
        }
        .into_page_parts()
    }
}

/// Config for bidirectional name-cursor pagination link building.
pub struct NameBidirectionalPageConfig {
    pub endpoint_url: Option<Url>,
    pub query_params: Vec<(&'static str, String)>,
    pub first_row_key: Option<(DateTime<Utc>, String)>,
    pub last_row_key: Option<(DateTime<Utc>, String)>,
    pub cursor: Option<NameCursor>,
    pub has_more: bool,
}

impl NameBidirectionalPageConfig {
    pub fn into_page_parts(self) -> (Option<NamePageParts>, Option<NamePageParts>) {
        let Some(endpoint_url) = self.endpoint_url else {
            return (None, None);
        };

        let is_backward = self
            .cursor
            .as_ref()
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
                self.last_row_key.map(|(date, name)| NamePageParts {
                    endpoint_url: endpoint_url.clone(),
                    query_params: self.query_params.clone(),
                    cursor: NameCursor::forward(date, name),
                }),
                self.first_row_key.map(|(date, name)| NamePageParts {
                    endpoint_url,
                    query_params: self.query_params,
                    cursor: NameCursor::backward(date, name),
                }),
            ),
            (true, false) => (
                self.last_row_key.map(|(date, name)| NamePageParts {
                    endpoint_url,
                    query_params: self.query_params,
                    cursor: NameCursor::forward(date, name),
                }),
                None,
            ),
            (false, true) => (
                None,
                self.first_row_key.map(|(date, name)| NamePageParts {
                    endpoint_url,
                    query_params: self.query_params,
                    cursor: NameCursor::backward(date, name),
                }),
            ),
            (false, false) => (None, None),
        }
    }
}

/// Adds Link headers for cursor-based pagination using a `NameCursor`.
/// Mirrors `Paginated<T>` but for endpoints whose tiebreak is a TEXT column.
#[derive(Debug, Clone)]
pub struct PaginatedByName<T: Apiv2Schema + OperationModifier + Responder> {
    pub data: T,
    pub next_page_parts: Option<NamePageParts>,
    pub prev_page_parts: Option<NamePageParts>,
}

impl<T: Apiv2Schema + OperationModifier + Responder> PaginatedByName<T> {
    fn build_link(parts: Option<NamePageParts>, rel: &str) -> Option<String> {
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

impl<T: Apiv2Schema + OperationModifier + Responder> Responder for PaginatedByName<T> {
    type Body = T::Body;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let mut res = self.data.respond_to(req);

        let next_link = Self::build_link(self.next_page_parts, "next");
        let prev_link = Self::build_link(self.prev_page_parts, "prev");

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

impl<T: Apiv2Schema + OperationModifier + Responder> Apiv2Schema for PaginatedByName<T> {
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

impl<T: Apiv2Schema + OperationModifier + Responder> OperationModifier for PaginatedByName<T> {
    fn update_parameter(op: &mut DefaultOperationRaw) {
        T::update_parameter(op);
    }
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

    /// Cursor wire format must stay on `URL_SAFE` (with `=` padding) so cursor
    /// URLs already in flight from older clients keep decoding. This test fails
    /// if anyone re-introduces `NO_PAD` without coordinating the wire-format
    /// migration with the consumers (request_attempts SDK callers in particular).
    #[test]
    fn wire_format_compat_pad() {
        let cursor = Cursor::forward(DateTime::UNIX_EPOCH, Uuid::nil());
        let encoded = cursor.to_qs_value().unwrap();
        // URL_SAFE base64 of a JSON cursor blob ends with one or more `=` padding chars.
        assert!(
            encoded.ends_with('='),
            "expected URL_SAFE (with padding) cursor encoding, got '{encoded}'"
        );
    }

    /// Round-trip a `(date, name)` cursor for endpoints with a string tiebreak.
    #[test]
    fn encode_and_decode_name_cursor() {
        let cursor = NameCursor::forward(
            Utc.with_ymd_and_hms(2025, 9, 28, 18, 0, 0).unwrap(),
            "myservice.user.created".to_owned(),
        );
        let encoded = cursor.to_qs_value().unwrap();
        let decoded = EncodedNameCursor::from_str(&encoded).unwrap();
        assert_eq!(decoded.0, cursor)
    }

    #[test]
    fn name_cursor_decode_malformed_returns_error() {
        // Non-base64 must produce a typed error, not panic.
        assert!(EncodedNameCursor::from_str("not!base64@@@").is_err());
        // Valid base64 of non-JSON bytes must also error cleanly.
        let bad_payload = BASE64_URL_SAFE.encode(b"not json");
        assert!(EncodedNameCursor::from_str(&bad_payload).is_err());
    }

    /// `NameCursor` must also stay on URL_SAFE-with-padding (matches `Cursor`).
    #[test]
    fn name_cursor_wire_format_compat_pad() {
        let cursor = NameCursor::forward(DateTime::UNIX_EPOCH, "x".to_owned());
        let encoded = cursor.to_qs_value().unwrap();
        assert!(
            encoded.ends_with('='),
            "expected URL_SAFE (with padding) name-cursor encoding, got '{encoded}'"
        );
    }

    /// `PageLimit::try_new` enforces 1 <= size <= MAX_LIMIT.
    #[test]
    fn page_limit_bounds() {
        assert_eq!(PageLimit::try_new(0).unwrap_err(), PageLimitError::TooSmall);
        assert_eq!(
            PageLimit::try_new(101).unwrap_err(),
            PageLimitError::TooLarge
        );
        assert_eq!(PageLimit::try_new(1).unwrap().size, 1);
        assert_eq!(PageLimit::try_new(50).unwrap().size, 50);
        assert_eq!(PageLimit::try_new(100).unwrap().size, 100);
        // Default is 20 and within bounds.
        assert_eq!(PageLimit::default().size, 20);
        assert!(PageLimit::try_new(PageLimit::default().size).is_ok());
    }

    /// `PageParts::mk_url` propagates whatever the handler puts in `query_params`
    /// into the next-link URL — including `limit`. The pagination machinery itself
    /// does NOT inject `limit` automatically; that's the handler's job. This test
    /// pins that contract.
    #[test]
    fn next_url_propagates_limit() {
        let parts = PageParts {
            endpoint_url: Url::parse("https://test.local/endpoint").unwrap(),
            query_params: vec![
                (
                    "application_id",
                    "00000000-0000-0000-0000-000000000000".to_owned(),
                ),
                ("limit", "42".to_owned()),
            ],
            cursor: Cursor::forward(DateTime::UNIX_EPOCH, Uuid::nil()),
        };
        let url = parts.mk_url().unwrap();
        let s = url.as_str();
        assert!(s.contains("limit=42"), "expected limit=42 in URL, got {s}");
        assert!(
            s.contains("pagination_cursor="),
            "expected cursor in URL, got {s}"
        );
    }

    /// Same propagation contract for NamePageParts (used by event_types).
    #[test]
    fn name_next_url_propagates_limit() {
        let parts = NamePageParts {
            endpoint_url: Url::parse("https://test.local/endpoint").unwrap(),
            query_params: vec![("limit", "42".to_owned())],
            cursor: NameCursor::forward(DateTime::UNIX_EPOCH, "n".to_owned()),
        };
        let url = parts.mk_url().unwrap();
        assert!(url.as_str().contains("limit=42"));
        assert!(url.as_str().contains("pagination_cursor="));
    }
}
