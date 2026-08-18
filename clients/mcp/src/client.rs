//! HTTP client for Hook0 API

use crate::config::Config;
use crate::error::Hook0McpError;
use reqwest::{Client, StatusCode, header};
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, warn};
use url::Url;

/// Longest each part this client composes its `User-Agent` out of may be, in characters.
///
/// The operating system is described by the platform rather than by this crate, so its length is
/// not this crate's to guarantee: the parts are cut here so that the header cannot grow with
/// whatever the platform feels like saying. Every part is also stripped of anything the grammar of
/// the header uses as punctuation, so a platform cannot forge a shape it does not have.
const MAX_USER_AGENT_PART_CHARS: usize = 64;

/// Name of the header every request states the retry policy behind it under.
const CLIENT_OPTIONS: &str = "hook0-client-options";

/// The retry policy behind every request this client makes, which is no retrying at all.
///
/// The SDKs carry a policy a caller sets and state it here; this server holds none, so it states
/// the one attempt it makes and the nothing it waits between attempts it does not make. The shape
/// is the shared contract's: parts joined by `,`, each cut at its first `=`, every duration a count
/// of milliseconds. Saying nothing instead would leave an instance unable to tell this server from
/// an SDK whose header went missing.
const NO_RETRIES: &str = "attempts=1,backoff=0,ceiling=0,budget=0";

/// Which SDK, at which version, on which runtime and operating system, is talking to the API.
///
/// The version is read from the manifest of this crate rather than written down again here: one
/// remembered in two places is one that will disagree with itself the first time it is bumped. The
/// name is this target's own rather than the Rust SDK's, since telling the two apart is the whole
/// point of an instance reading this.
fn user_agent() -> String {
    let version = clipped_part(env!("CARGO_PKG_VERSION"));
    // Nothing in the standard library answers which compiler built this, so the runtime is named
    // and not versioned; the operating system and the architecture are what it runs on.
    let os = clipped_part(&format!(
        "{} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
    format!("hook0-client-mcp/{version} (rust; {os})")
}

/// One part of the `User-Agent`, with everything the header's own grammar uses taken out of it and
/// cut to [`MAX_USER_AGENT_PART_CHARS`].
fn clipped_part(part: &str) -> String {
    part.chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ')
        .filter(|c| !matches!(c, '(' | ')' | ';'))
        .take(MAX_USER_AGENT_PART_CHARS)
        .collect()
}

/// HTTP client for Hook0 API
#[derive(Debug, Clone)]
pub struct Hook0Client {
    client: Client,
    base_url: Url,
    token: String,
}

impl Hook0Client {
    /// Create a new Hook0 client from configuration
    pub fn new(config: &Config) -> Result<Self, Hook0McpError> {
        let mut headers = header::HeaderMap::new();

        // Set default headers
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        // Said once here rather than per request: nothing this server does changes the policy
        // behind its requests, since it holds none to change.
        headers.insert(
            header::HeaderName::from_static(CLIENT_OPTIONS),
            header::HeaderValue::from_static(NO_RETRIES),
        );

        // Create HTTP client
        let client = Client::builder()
            .default_headers(headers)
            // An instance can otherwise not tell which SDKs, at which versions, are still reaching
            // it — and this server went a whole release without saying either.
            .user_agent(user_agent())
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(Hook0McpError::Http)?;

        Ok(Self {
            client,
            base_url: config.api_url.clone(),
            token: config.api_token.clone(),
        })
    }

    /// Build a URL for an API path
    fn url(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        let base_path = url.path().trim_end_matches('/');
        let clean_path = path.trim_start_matches('/');

        // Ensure we're using the API v1 prefix
        let full_path = if clean_path.starts_with("api/v1/") {
            format!("{}/{}", base_path, clean_path)
        } else {
            format!("{}/api/v1/{}", base_path, clean_path)
        };

        url.set_path(&full_path);
        url
    }

    /// Execute a GET request
    pub async fn get(&self, path: &str) -> Result<Value, Hook0McpError> {
        let url = self.url(path);
        debug!("GET {}", url);

        let response = self
            .client
            .get(url.clone())
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(Hook0McpError::Http)?;

        self.handle_response(response).await
    }

    /// Execute a POST request
    pub async fn post(&self, path: &str, body: Option<Value>) -> Result<Value, Hook0McpError> {
        let url = self.url(path);
        debug!("POST {}", url);

        let mut request = self.client.post(url.clone()).bearer_auth(&self.token);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await.map_err(Hook0McpError::Http)?;

        self.handle_response(response).await
    }

    /// Execute a PUT request
    pub async fn put(&self, path: &str, body: Option<Value>) -> Result<Value, Hook0McpError> {
        let url = self.url(path);
        debug!("PUT {}", url);

        let mut request = self.client.put(url.clone()).bearer_auth(&self.token);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await.map_err(Hook0McpError::Http)?;

        self.handle_response(response).await
    }

    /// Execute a PATCH request
    pub async fn patch(&self, path: &str, body: Option<Value>) -> Result<Value, Hook0McpError> {
        let url = self.url(path);
        debug!("PATCH {}", url);

        let mut request = self.client.patch(url.clone()).bearer_auth(&self.token);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await.map_err(Hook0McpError::Http)?;

        self.handle_response(response).await
    }

    /// Execute a DELETE request
    pub async fn delete(&self, path: &str) -> Result<Value, Hook0McpError> {
        let url = self.url(path);
        debug!("DELETE {}", url);

        let response = self
            .client
            .delete(url.clone())
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(Hook0McpError::Http)?;

        self.handle_response(response).await
    }

    /// Handle HTTP response
    async fn handle_response(&self, response: reqwest::Response) -> Result<Value, Hook0McpError> {
        let status = response.status();
        let url = response.url().clone();

        if status.is_success() {
            // Handle empty responses (204 No Content)
            if status == StatusCode::NO_CONTENT {
                return Ok(Value::Null);
            }

            // Try to parse JSON response
            let text = response.text().await.map_err(Hook0McpError::Http)?;
            if text.is_empty() {
                return Ok(Value::Null);
            }

            serde_json::from_str(&text).map_err(|e| {
                warn!("Failed to parse response as JSON: {}", e);
                Hook0McpError::Json(e)
            })
        } else {
            let error_body = response.text().await.unwrap_or_default();
            let message = refusal(&error_body);

            warn!("API error: {} {} - {}", status.as_u16(), url, message);

            Err(Hook0McpError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }
}

/// Longest a refusal read off an answer may be, in characters.
///
/// The body is written by the other end, and an error an assistant reads is not the place to hold
/// however much of it arrived. What is cut is said, so a reader knows there was more.
const MAX_REFUSAL_CHARS: usize = 2048;

/// One string field of a JSON body, when it carries one that says something.
fn field<'a>(body: &'a Value, name: &str) -> Option<&'a str> {
    body.get(name)
        .and_then(Value::as_str)
        .filter(|said| !said.is_empty())
}

/// As much of a refusal as is reported, saying what was left out.
fn clipped(said: &str) -> String {
    if said.chars().count() <= MAX_REFUSAL_CHARS {
        return said.to_owned();
    }
    let held: String = said.chars().take(MAX_REFUSAL_CHARS).collect();
    format!("{held}… ({} characters in all)", said.chars().count())
}

/// What an answer Hook0 refused a request with said.
///
/// Hook0 answers a refusal with an RFC 9457 problem document, and the stable name of the problem is
/// under `id`: `EventAlreadyIngested`, `AuthInvalidApplicationSecret`, and so on. That name is the
/// only part of the answer an assistant can act on — it is what tells a duplicated ingestion, which
/// is already done and must not be repeated, from any other conflict — so it is named first and the
/// prose beside it after. Dropping it in favour of the prose leaves every refusal looking alike.
///
/// A body that is not one of those documents is reported as it arrived: a proxy or a gateway
/// between the assistant and Hook0 writes what it likes, and what it wrote is the only clue there
/// is.
fn refusal(body: &str) -> String {
    let Ok(json) = serde_json::from_str::<Value>(body) else {
        return clipped(body);
    };

    let said = field(&json, "message")
        .or_else(|| field(&json, "error"))
        .or_else(|| field(&json, "detail"));

    match (field(&json, "id"), said) {
        (Some(problem), Some(said)) => clipped(&format!("{problem}: {said}")),
        (Some(problem), None) => clipped(problem),
        (None, Some(said)) => clipped(said),
        (None, None) => clipped(body),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Transport;
    use serde_json::json;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    /// Longest the exchange below is given before it is abandoned, so a socket that never answers
    /// fails the case rather than holding the suite.
    const EXCHANGE_TIMEOUT: Duration = Duration::from_secs(10);

    /// Largest request head read back off the socket, in bytes.
    const MAX_HEAD_BYTES: usize = 16 * 1024;

    /// Largest request body read back off the socket, in bytes. The body below is this suite's own,
    /// so one above this is a request nothing here meant to send.
    const MAX_BODY_BYTES: usize = 64 * 1024;

    /// Where the shared contract sits, from the crate this suite tests.
    const CORPUS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../conformance");

    /// Largest document of the corpus read back. The corpus is committed, so one above this is one
    /// that grew out of shape rather than one somebody meant.
    const MAX_CORPUS_BYTES: u64 = 512 * 1024;

    /// The credential the client below is built with, and the one the request document expects to
    /// find on the wire.
    const TOKEN: &str = "token-xyz";

    /// Which client this is, as the request document spells it. This target's own name rather than
    /// the Rust SDK's: telling the two apart is the whole point of an instance reading the header.
    const LANGUAGE: &str = "mcp";

    /// The retry policy behind every request this server makes, as the numbers the request document
    /// composes its header out of. It makes one attempt and waits nothing between attempts it does
    /// not make, so every hole of that header is one this suite can speak for and the value is
    /// compared whole rather than matched.
    const ONE_ATTEMPT: &str = "1";
    const NO_WAIT: &str = "0";

    /// One document of the shared contract, bounded before it is parsed.
    fn corpus(document: &str) -> Value {
        let path = PathBuf::from(CORPUS).join(document);
        let about = fs::metadata(&path).unwrap_or_else(|e| {
            panic!(
                "the shared contract at {} is unreadable: {e}",
                path.display()
            )
        });
        assert!(
            about.len() <= MAX_CORPUS_BYTES,
            "{} is {} bytes long, above the {MAX_CORPUS_BYTES} read back",
            path.display(),
            about.len(),
        );

        let written = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "the shared contract at {} is unreadable: {e}",
                path.display()
            )
        });
        serde_json::from_str(&written).unwrap_or_else(|e| {
            panic!(
                "{} does not read as the contract it is: {e}",
                path.display()
            )
        })
    }

    /// The entries the corpus carries at `at`, which no case may find empty: a document declaring
    /// nothing would let every case below pass without exercising anything.
    fn entries(document: &Value, at: &str) -> Vec<Value> {
        let found = document
            .pointer(at)
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("the shared contract carries nothing at `{at}`"));
        assert!(!found.is_empty(), "the shared contract is empty at `{at}`");
        found.to_owned()
    }

    /// What the corpus wrote at `field`, as text.
    fn text<'a>(entry: &'a Value, field: &str) -> &'a str {
        entry
            .get(field)
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("`{field}` is not the text the contract declares"))
    }

    /// What the corpus wrote at `field`, as a count of bytes.
    fn count(entry: &Value, field: &str) -> usize {
        entry
            .get(field)
            .and_then(Value::as_u64)
            .and_then(|counted| usize::try_from(counted).ok())
            .unwrap_or_else(|| panic!("`{field}` is not the count the contract declares"))
    }

    /// What a value of the request document is made of, once the holes this suite can speak for are
    /// filled in.
    ///
    /// A value is a template: `${name}` is a hole and everything around it is literal. A hole named
    /// in `bound` becomes part of the literal text around it; one that is not is a hole no suite can
    /// fill without reimplementing the client it is testing, and it separates two chunks. A template
    /// whose holes are all bound is therefore one chunk, and the whole value is that chunk.
    fn template_chunks(template: &str, bound: &[(&str, &str)]) -> Vec<String> {
        let mut chunks = vec![String::new()];
        let mut rest = template;

        while let Some(opened) = rest.find("${") {
            let Some(closed) = rest[opened..].find('}').map(|found| opened + found) else {
                break;
            };
            let last = chunks.len() - 1;
            chunks[last].push_str(&rest[..opened]);

            match bound
                .iter()
                .find(|(name, _)| *name == &rest[opened + 2..closed])
            {
                Some((_, filled)) => chunks[last].push_str(filled),
                None => chunks.push(String::new()),
            }
            rest = &rest[closed + 1..];
        }

        let last = chunks.len() - 1;
        chunks[last].push_str(rest);
        chunks
    }

    /// Whether what arrived is what those chunks describe: the literal text in order, anchored at
    /// both ends, with something non-empty standing in every hole between them.
    fn matches_chunks(chunks: &[String], carried: &str) -> bool {
        let Some((first, rest_of)) = chunks.split_first() else {
            return false;
        };
        let Some((last, between)) = rest_of.split_last() else {
            return carried == first;
        };
        let Some(mut rest) = carried.strip_prefix(first.as_str()) else {
            return false;
        };

        for chunk in between {
            // A hole stands before this chunk, and nothing is not something, so the search starts
            // past whatever fills it.
            let Some(found) = rest.get(1..).and_then(|past| past.find(chunk.as_str())) else {
                return false;
            };
            rest = &rest[1 + found + chunk.len()..];
        }

        rest.len() > last.len() && rest.ends_with(last.as_str())
    }

    /// How many bytes of body a request head says follow it, when it says.
    fn announced_body_bytes(head: &str) -> Option<usize> {
        head.lines()
            .filter_map(|line| line.split_once(':'))
            .find(|(name, _)| name.trim().eq_ignore_ascii_case("content-length"))
            .and_then(|(_, counted)| counted.trim().parse::<usize>().ok())
    }

    /// The headers a request arrived with, by the name HTTP compares them under.
    fn headers_of(head: &str) -> HashMap<String, String> {
        head.lines()
            .skip(1)
            .filter_map(|line| line.split_once(':'))
            .map(|(name, value)| (name.trim().to_lowercase(), value.trim().to_owned()))
            .collect()
    }

    /// What this server puts on the wire, read back off a socket and held against the shared
    /// conformance corpus.
    ///
    /// The corpus at `clients/conformance` is what the eleven SDKs are driven against, and
    /// `request.json` is the document of it this server can honour: it holds no retry policy to
    /// vary, receives no webhook to verify, and applies none of the payload and response bounds.
    /// Nothing held it to that one document either, which is how it went a whole release stating
    /// neither which client it is nor the policy behind its requests while every SDK stated both.
    /// The header table is read out of the committed contract rather than written down here, so a
    /// header renamed there, or added, fails here rather than at the first API that notices.
    ///
    /// The request carries a body, so every occasion the corpus declares applies to this one send.
    #[tokio::test]
    async fn every_request_carries_what_the_shared_contract_says_it_does() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a loopback port is available to bind");
        let address = listener
            .local_addr()
            .expect("a bound listener has a local address");

        let served = tokio::spawn(async move {
            let (mut stream, _) = listener
                .accept()
                .await
                .expect("the client reaches the listening socket");

            let mut read = Vec::new();
            let mut byte = [0u8; 1];
            while !read.ends_with(b"\r\n\r\n") && read.len() < MAX_HEAD_BYTES {
                match stream.read(&mut byte).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => read.push(byte[0]),
                }
            }
            let head = String::from_utf8_lossy(&read).into_owned();

            // The body is drained before the answer goes back: a socket closed with bytes still
            // unread is reset rather than closed, and the client reads the reset in place of the
            // answer.
            if let Some(carried) = announced_body_bytes(&head) {
                let mut body = vec![0u8; carried.min(MAX_BODY_BYTES)];
                let _ = stream.read_exact(&mut body).await;
            }

            stream
                .write_all(b"HTTP/1.1 204 No Content\r\ncontent-length: 0\r\n\r\n")
                .await
                .expect("the answer is written back");
            head
        });

        let config = Config {
            api_url: Url::parse(&format!("http://{address}"))
                .expect("a loopback address makes a parsable base URL"),
            api_token: TOKEN.to_owned(),
            transport: Transport::Stdio,
            read_only: false,
        };
        let client = Hook0Client::new(&config).expect("the client accepts a loopback API URL");

        tokio::time::timeout(
            EXCHANGE_TIMEOUT,
            client.post("/applications", Some(json!({ "name": "an application" }))),
        )
        .await
        .expect("the exchange finishes inside its deadline")
        .expect("the API answers the request");

        let head = tokio::time::timeout(EXCHANGE_TIMEOUT, served)
            .await
            .expect("the socket finishes reading inside its deadline")
            .expect("the listening task ran to the end");
        let carried = headers_of(&head);

        let contract = corpus("request.json");
        let composed_at_most = count(&contract, "max_composed_bytes");

        for header in entries(&contract, "/headers") {
            let name = text(&header, "name").to_lowercase();
            let template = text(&header, "value");
            let written = carried.get(&name).map(String::as_str).unwrap_or("");
            let chunks = template_chunks(
                template,
                &[
                    ("token", TOKEN),
                    ("language", LANGUAGE),
                    ("attempts", ONE_ATTEMPT),
                    ("backoff_ms", NO_WAIT),
                    ("ceiling_ms", NO_WAIT),
                    ("budget_ms", NO_WAIT),
                ],
            );

            assert!(
                matches_chunks(&chunks, written),
                "the request carried `{name}: {written}` where the shared contract says \
                 `{template}`: {}",
                text(&header, "reason"),
            );

            // A value with a hole this suite cannot fill is one the client composed out of what the
            // platform told it, and what the platform says is as long as it feels like.
            if chunks.len() > 1 {
                assert!(
                    written.len() <= composed_at_most,
                    "the request carried {} bytes of `{name}`, above the {composed_at_most} the \
                     shared contract cuts a composed value to",
                    written.len(),
                );
            }
        }
    }

    /// The problem document Hook0 answers a duplicated ingestion with, as its API writes it.
    const ALREADY_INGESTED: &str = r#"{"id":"EventAlreadyIngested","title":"Event already Ingested","detail":"This event was previously ingested and recorded inside Hook0 service.","status":409}"#;

    #[test]
    fn a_refusal_names_the_problem_hook0_named() {
        assert_eq!(
            refusal(ALREADY_INGESTED),
            "EventAlreadyIngested: This event was previously ingested and recorded inside Hook0 service."
        );
    }

    #[test]
    fn a_refusal_that_names_no_problem_is_reported_as_it_arrived() {
        assert_eq!(
            refusal("<html>502 Bad Gateway</html>"),
            "<html>502 Bad Gateway</html>"
        );
        assert_eq!(refusal(r#"{"message":"nope"}"#), "nope");
        assert_eq!(refusal(""), "");
    }

    #[test]
    fn a_refusal_is_cut_to_what_is_reported() {
        let long = "x".repeat(MAX_REFUSAL_CHARS + 1);

        let said = refusal(&long);

        assert!(said.starts_with(&"x".repeat(MAX_REFUSAL_CHARS)));
        assert!(said.ends_with(&format!("… ({} characters in all)", long.len())));
    }

    #[test]
    fn test_url_building() {
        let config = Config {
            api_url: Url::parse("https://app.hook0.com").unwrap(),
            api_token: "test-token".to_string(),
            transport: crate::config::Transport::Stdio,
            read_only: false,
        };

        let client = Hook0Client::new(&config).unwrap();

        // Should add api/v1 prefix
        let url = client.url("/applications");
        assert_eq!(url.as_str(), "https://app.hook0.com/api/v1/applications");

        // Should not duplicate api/v1 prefix
        let url = client.url("/api/v1/applications");
        assert_eq!(url.as_str(), "https://app.hook0.com/api/v1/applications");
    }
}
