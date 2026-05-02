//! Black-box test of `Hook0Client::get_paginated` cursor-following.
//!
//! Spins up a `wiremock::MockServer` that mimics Hook0's RFC 5988 `Link: rel="next"`
//! pagination contract and verifies the MCP cursor-follow helper:
//!
//! - AC-29: walking a 3-page chain returns the merged flat array (no duplicates,
//!   no skipped items, in the order returned by the server).
//! - AC-30: when total rows exceed the 1000-item cap, the response is wrapped in
//!   the `{ items, _truncated, _truncated_count, _message }` envelope so the LLM
//!   can drop down to manual cursor follow.
//!
//! The MCP server itself is not exercised here — we test the HTTP client layer
//! used by every paginated MCP tool, which is the actual unit shipped in this PR.

use hook0_mcp::client::Hook0Client;
use hook0_mcp::config::{Config, Transport};
use serde_json::{Value, json};
use url::Url;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_client(server: &MockServer) -> Hook0Client {
    let config = Config {
        api_url: Url::parse(&server.uri()).expect("MockServer URI must parse"),
        api_token: "test-token".to_string(),
        transport: Transport::Stdio,
        read_only: false,
    };
    Hook0Client::new(&config).expect("client must construct")
}

/// Builds a simple JSON array of `count` event-type-shaped rows with names
/// `<prefix><index>` so test assertions can match deterministically.
fn rows(prefix: &str, start: usize, count: usize) -> Value {
    let arr: Vec<Value> = (0..count)
        .map(|i| {
            json!({
                "event_type_name": format!("{}{}", prefix, start + i),
                "service": "test",
                "resource_type": "x",
                "verb": "y",
            })
        })
        .collect();
    Value::Array(arr)
}

#[tokio::test]
async fn paginated_walk_merges_three_pages_in_order() {
    let server = MockServer::start().await;

    // Page 1 (initial call): returns rows 0..100 + Link to /api/v1/event_types?cursor=p2.
    // The MCP `get_paginated` always appends `?limit=100` to the initial URL.
    let next_p2 = format!("{}/api/v1/event_types?cursor=p2", server.uri());
    let next_p3 = format!("{}/api/v1/event_types?cursor=p3", server.uri());

    Mock::given(method("GET"))
        .and(path("/api/v1/event_types"))
        .and(query_param("limit", "100"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Link", format!("<{}>; rel=\"next\"", next_p2))
                .set_body_json(rows("name", 0, 100)),
        )
        .expect(1)
        .mount(&server)
        .await;

    // Page 2: cursor=p2 -> 100 more rows + next pointing to cursor=p3.
    Mock::given(method("GET"))
        .and(path("/api/v1/event_types"))
        .and(query_param("cursor", "p2"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Link", format!("<{}>; rel=\"next\"", next_p3))
                .set_body_json(rows("name", 100, 100)),
        )
        .expect(1)
        .mount(&server)
        .await;

    // Page 3 (last): cursor=p3 -> 5 rows, no next link.
    Mock::given(method("GET"))
        .and(path("/api/v1/event_types"))
        .and(query_param("cursor", "p3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(rows("name", 200, 5)))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let result = client
        .get_paginated("/api/v1/event_types?application_id=11111111-1111-1111-1111-111111111111")
        .await
        .expect("paginated walk must succeed");

    let items = result.as_array().expect("non-truncated walk returns Value::Array");
    assert_eq!(
        items.len(),
        205,
        "expected 100+100+5 merged rows, got {}",
        items.len()
    );

    // Sanity: order preserved, no duplicates, no skipped.
    for (i, row) in items.iter().enumerate() {
        let name = row
            .get("event_type_name")
            .and_then(|v| v.as_str())
            .expect("each row must carry event_type_name");
        assert_eq!(name, format!("name{i}"), "row {i} out of order: {name}");
    }
}

#[tokio::test]
async fn paginated_walk_truncates_at_cap_and_returns_envelope() {
    // Build a server that hands out 1100 rows across 11 pages of 100. The MCP
    // helper hard-caps at 1000 items so we expect the final result wrapped in
    // the truncation envelope.
    let server = MockServer::start().await;

    // Initial page (limit=100, no cursor) -> rows 0..100, next=p1.
    let next_url = |p: usize| format!("{}/api/v1/event_types?cursor=p{p}", server.uri());

    Mock::given(method("GET"))
        .and(path("/api/v1/event_types"))
        .and(query_param("limit", "100"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Link", format!("<{}>; rel=\"next\"", next_url(1)))
                .set_body_json(rows("r", 0, 100)),
        )
        .mount(&server)
        .await;

    // Pages cursor=p1..p10 each return 100 rows. The last one, p10, yields the
    // overshoot rows that should be truncated server-side. We register them as
    // OPTIONAL because the helper is allowed to stop short of fetching p10
    // once it sees the in-memory cap was reached.
    for p in 1..=10 {
        let body = rows("r", p * 100, 100);
        let mut resp = ResponseTemplate::new(200).set_body_json(body);
        if p < 10 {
            resp = resp.insert_header("Link", format!("<{}>; rel=\"next\"", next_url(p + 1)));
        }
        Mock::given(method("GET"))
            .and(path("/api/v1/event_types"))
            .and(query_param("cursor", format!("p{p}")))
            .respond_with(resp)
            .mount(&server)
            .await;
    }

    let client = make_client(&server);
    let result = client
        .get_paginated("/api/v1/event_types?application_id=11111111-1111-1111-1111-111111111111")
        .await
        .expect("paginated walk must succeed under cap");

    // Truncation envelope shape.
    let obj = result.as_object().expect(
        "exceeding the 1000-item cap must produce a `{items, _truncated, ...}` object envelope",
    );
    assert_eq!(
        obj.get("_truncated").and_then(|v| v.as_bool()),
        Some(true),
        "envelope must mark `_truncated: true`"
    );
    assert_eq!(
        obj.get("_truncated_count").and_then(|v| v.as_u64()),
        Some(1000),
        "envelope must record the 1000-item cap"
    );
    let message = obj
        .get("_message")
        .and_then(|v| v.as_str())
        .expect("envelope must carry a human-readable `_message`");
    assert!(
        message.to_lowercase().contains("truncat") || message.contains("1000"),
        "message should mention truncation/cap, got: {message}"
    );
    let items = obj
        .get("items")
        .and_then(|v| v.as_array())
        .expect("envelope must carry the partial `items` array");
    assert_eq!(items.len(), 1000, "items must be capped at 1000");

    // Order preserved through cap: first row is r0, last is r999.
    let first = items[0]
        .get("event_type_name")
        .and_then(|v| v.as_str())
        .unwrap();
    let last = items[999]
        .get("event_type_name")
        .and_then(|v| v.as_str())
        .unwrap();
    assert_eq!(first, "r0");
    assert_eq!(last, "r999");
}

#[tokio::test]
async fn paginated_walk_single_page_returns_array_directly() {
    // Smallest case: server returns one page with no Link header. The helper
    // must NOT wrap a < cap response in the truncation envelope; it returns
    // a plain Value::Array.
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/event_types"))
        .and(query_param("limit", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(rows("only", 0, 3)))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let result = client
        .get_paginated("/api/v1/event_types?application_id=11111111-1111-1111-1111-111111111111")
        .await
        .expect("paginated walk must succeed");

    let arr = result.as_array().expect("single page must return Value::Array, not envelope");
    assert_eq!(arr.len(), 3);
}
