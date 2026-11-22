use chrono::{DateTime, Utc};
use clap::{crate_name, crate_version};
use futures::TryStreamExt;
use hex::ToHex;
use hmac::{Hmac, KeyInit, Mac};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, InvalidHeaderValue};
use reqwest::{Client, Method, Url};
use serde_json::Value;
use sha2::Sha256;
use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::{Duration, Instant};
use strum::VariantNames;
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;
use tracing::{debug, error, instrument, trace, warn};

use crate::dns::DnsResolver;
use crate::opentelemetry::{DeliveryTraceIds, report_response_body_truncated};
use crate::standard_webhooks::StandardWebhooksSignature;
use crate::{Config, RequestAttempt, SignatureVersion};

const USER_AGENT: &str = concat!(crate_name!(), "/", crate_version!());

#[derive(Debug, Clone, Copy, strum::Display, VariantNames)]
pub enum ResponseError {
    #[strum(serialize = "E_UNKNOWN")]
    Unknown,
    #[strum(serialize = "E_INVALID_HEADER")]
    InvalidHeader,
    #[strum(serialize = "E_INVALID_TARGET")]
    InvalidTarget,
    #[strum(serialize = "E_DNS")]
    Dns,
    #[strum(serialize = "E_CONNECTION")]
    Connection,
    #[strum(serialize = "E_TIMEOUT")]
    Timeout,
    #[strum(serialize = "E_HTTP")]
    Http,
    /// A failure on Hook0's side that says nothing about the target (it does not count against the subscription's health)
    #[strum(serialize = "E_INTERNAL")]
    Internal,
}

/// A failure that happened before any HTTP request was made: the stored URL could not be
/// parsed, or it could not be resolved. Carries its own code, because a resolver failure is not
/// necessarily the target's fault.
struct TargetFailure {
    /// Customer-visible; stored as the response body.
    message: String,
    response_error: ResponseError,
    /// Operators only — deliberately absent from `message`.
    detail: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub response_error: Option<ResponseError>,
    pub http_code: Option<u16>,
    pub headers: Option<HeaderMap>,
    pub body: Option<Vec<u8>>,
    /// The target sent more than `MAX_RESPONSE_BODY_SIZE`: `body` is only the beginning of it
    pub body_truncated: bool,
    pub elapsed_time: Duration,
}

impl Response {
    pub fn is_success(&self) -> bool {
        self.response_error.is_none()
    }

    #[allow(non_snake_case)]
    pub fn response_error__name(&self) -> Option<String> {
        self.response_error.map(|re| re.to_string())
    }

    pub fn http_code(&self) -> Option<i16> {
        self.http_code.and_then(|c| c.try_into().ok())
    }

    /// The response headers, as a JSON object.
    ///
    /// The target chooses these bytes, and a header value is only guaranteed to be bytes — HTTP
    /// allows values outside visible ASCII. Such a value is transcribed lossily rather than
    /// refused: a header we cannot read exactly is still worth recording, and no response a target
    /// decides to send may take a delivery down.
    pub fn headers(&self) -> Option<serde_json::Value> {
        self.headers.as_ref().and_then(|hm| {
            let hashmap: HashMap<String, String> = hm
                .iter()
                .map(|(k, v)| {
                    (
                        k.to_string(),
                        String::from_utf8_lossy(v.as_bytes()).into_owned(),
                    )
                })
                .collect();
            serde_json::to_value(hashmap).ok()
        })
    }

    pub fn elapsed_time_ms(&self) -> i32 {
        self.elapsed_time.as_millis().try_into().unwrap_or(0)
    }
}

#[instrument(skip_all, fields(request_attempt_id = %attempt.request_attempt_id, trace_id = %ids.trace_id, span_id = %ids.span_id))]
pub async fn work(
    config: &Config,
    resolver: &DnsResolver,
    attempt: &RequestAttempt,
    ids: &DeliveryTraceIds,
) -> Response {
    debug!("Processing request attempt");
    let start = Instant::now();

    let m = Method::from_str(attempt.http_method.as_str());
    let u = match Url::parse(attempt.http_url.as_str()) {
        Err(e) => Err(TargetFailure {
            message: e.to_string(),
            response_error: ResponseError::InvalidTarget,
            detail: None,
        }),
        Ok(url) => match resolver
            .resolve_target(&url, config.disable_target_ip_check)
            .await
        {
            Ok(addrs) => Ok((url, addrs)),
            Err(e) => Err(TargetFailure {
                response_error: e.response_error(),
                detail: e.detail().map(str::to_owned),
                message: e.to_string(),
            }),
        },
    };
    let hs = parse_headers(attempt.http_headers.clone());
    let et = HeaderValue::from_str(&attempt.event_type_name);
    let event_id = HeaderValue::from_str(attempt.event_id.to_string().as_str())
        .expect("Could not create a header value from the event ID UUID");
    let content_type = HeaderValue::from_str(attempt.payload_content_type.as_str())
        .expect("Could not create a header value from the event content type");

    match (m, u, hs, et) {
        (Ok(method), Ok((url, addrs)), Ok(mut headers), Ok(et)) => {
            // Pin the connection to the exact addresses we just vetted so reqwest cannot re-resolve the hostname to a different (forbidden) IP between the check and the request (DNS rebinding).
            // Only domain hosts need this; IP-literal URLs skip DNS.
            let pin = url.domain().map(|host| (host, addrs.as_slice()));
            let client = match mk_http_client(config.connect_timeout, config.timeout, pin) {
                Ok(client) => client,
                Err(e) => {
                    error!("Could not create HTTP client: {e}");
                    return Response {
                        response_error: Some(ResponseError::Internal),
                        http_code: None,
                        headers: None,
                        body: Some(e.to_string().into_bytes()),
                        body_truncated: false,
                        elapsed_time: start.elapsed(),
                    };
                }
            };

            headers.insert("Content-Type", content_type);
            headers.insert("X-Event-Id", event_id);
            headers.insert("X-Event-Type", et);

            let signed_at = Utc::now();
            let s = Signature::new(
                &attempt.secret.to_string(),
                &attempt.payload,
                signed_at,
                &headers,
            )
            .map_err(|e| {
                let msg =
                    format!("Could not construct header '{e}' because it has an invalid value");
                warn!["{msg}"];
                Box::new(Response {
                    response_error: Some(ResponseError::InvalidHeader),
                    http_code: None,
                    headers: None,
                    body: Some(msg.into_bytes()),
                    body_truncated: false,
                    elapsed_time: start.elapsed(),
                })
            })
            .and_then(|sig| {
                sig.to_header_value(
                    config
                        .enabled_signature_versions
                        .contains(&SignatureVersion::V0),
                    config
                        .enabled_signature_versions
                        .contains(&SignatureVersion::V1),
                )
                .map_err(|e| {
                    warn!(
                        "Could not construct signature header '{}' because it has an invalid value: {e}",
                        config.signature_header_name
                    );
                    Box::new(Response {
                        response_error: Some(ResponseError::InvalidHeader),
                        http_code: None,
                        headers: None,
                        body: None,
                        body_truncated: false,
                        elapsed_time: start.elapsed(),
                    })
                })
            });

            match s {
                Ok(sig) => {
                    headers.insert(&config.signature_header_name, sig);

                    // Standard Webhooks interop headers, emitted alongside Hook0's native
                    // signature when opted in. Inserted after the native signature so its v1
                    // header-signature never covers the `webhook-*` headers. The event id
                    // (stable across retries) is the message id.
                    if config.standard_webhooks_signature_enabled {
                        let sw = StandardWebhooksSignature::new(
                            &attempt.event_id.to_string(),
                            &attempt.secret.to_string(),
                            &attempt.payload,
                            signed_at,
                        );
                        headers.insert(
                            HeaderName::from_static(StandardWebhooksSignature::ID_HEADER),
                            sw.id_header_value(),
                        );
                        headers.insert(
                            HeaderName::from_static(StandardWebhooksSignature::TIMESTAMP_HEADER),
                            sw.timestamp_header_value(),
                        );
                        headers.insert(
                            HeaderName::from_static(StandardWebhooksSignature::SIGNATURE_HEADER),
                            sw.signature_header_value(),
                        );
                    }

                    debug!("Calling webhook...");
                    let redacted_headers = RedactedHeaders {
                        headers: &headers,
                        safe_headers: &[
                            HeaderName::from_static("content-type"),
                            HeaderName::from_static("x-event-id"),
                            HeaderName::from_static("x-event-type"),
                            HeaderName::from_static(StandardWebhooksSignature::ID_HEADER),
                            HeaderName::from_static(StandardWebhooksSignature::TIMESTAMP_HEADER),
                            HeaderName::from_static(StandardWebhooksSignature::SIGNATURE_HEADER),
                            config.signature_header_name.clone(),
                        ],
                    };
                    trace!(
                        http_method = %method.to_string().to_uppercase(),
                        %url,
                        headers = ?redacted_headers,
                        headers_count = headers.len(),
                        "Calling webhook"
                    );
                    let response = client
                        .request(method, url)
                        .headers(headers)
                        .body(attempt.payload.clone())
                        .send()
                        .await;

                    match response {
                        Ok(res) => {
                            let status = res.status();
                            let headers = res.headers().clone();
                            let body =
                                read_capped_body(res, config.max_response_body_size.as_u64()).await;

                            if body.truncated {
                                debug!(
                                    limit = %config.max_response_body_size,
                                    "Webhook response body was larger than the limit and was truncated"
                                );
                                report_response_body_truncated();
                            }

                            if status.is_success() {
                                debug!("Webhook call was successful");
                                Response {
                                    response_error: None,
                                    http_code: Some(status.as_u16()),
                                    headers: Some(headers),
                                    body: body.bytes,
                                    body_truncated: body.truncated,
                                    elapsed_time: start.elapsed(),
                                }
                            } else {
                                warn!(http_status = %status, "Webhook call failed with HTTP error");
                                Response {
                                    response_error: Some(ResponseError::Http),
                                    http_code: Some(status.as_u16()),
                                    headers: Some(headers),
                                    body: body.bytes,
                                    body_truncated: body.truncated,
                                    elapsed_time: start.elapsed(),
                                }
                            }
                        }
                        Err(e) if e.is_connect() => {
                            warn!("Webhook call failed with connection error: {e}");
                            Response {
                                response_error: Some(ResponseError::Connection),
                                http_code: None,
                                headers: None,
                                body: Some(e.to_string().into_bytes()),
                                body_truncated: false,
                                elapsed_time: start.elapsed(),
                            }
                        }
                        Err(e) if e.is_timeout() => {
                            warn!("Webhook call failed with timeout error: {e}");
                            Response {
                                response_error: Some(ResponseError::Timeout),
                                http_code: None,
                                headers: None,
                                body: Some(e.to_string().into_bytes()),
                                body_truncated: false,
                                elapsed_time: start.elapsed(),
                            }
                        }
                        Err(e) => {
                            warn!("Webhook call failed with unknown error: {e}");
                            Response {
                                response_error: Some(ResponseError::Unknown),
                                http_code: None,
                                headers: None,
                                body: Some(e.to_string().into_bytes()),
                                body_truncated: false,
                                elapsed_time: start.elapsed(),
                            }
                        }
                    }
                }
                Err(e) => *e,
            }
        }
        (Err(e), _, _, _) => {
            error!(
                target_http_method = attempt.http_method,
                "Target has an invalid HTTP method: {e}"
            );
            Response {
                response_error: Some(ResponseError::InvalidTarget),
                http_code: None,
                headers: None,
                body: Some(e.to_string().into_bytes()),
                body_truncated: false,
                elapsed_time: start.elapsed(),
            }
        }
        (_, Err(f), _, _) => {
            warn!(
                target_http_url = attempt.http_url,
                response_error = %f.response_error,
                detail = f.detail.as_deref(),
                "Could not use target URL: {}", f.message
            );
            Response {
                response_error: Some(f.response_error),
                http_code: None,
                headers: None,
                body: Some(f.message.into_bytes()),
                body_truncated: false,
                elapsed_time: start.elapsed(),
            }
        }
        (_, _, Err(e), _) => {
            warn!("Target has invalid headers: {e}");
            Response {
                response_error: Some(ResponseError::InvalidTarget),
                http_code: None,
                headers: None,
                body: Some(e.to_string().into_bytes()),
                body_truncated: false,
                elapsed_time: start.elapsed(),
            }
        }
        (_, _, _, Err(_)) => {
            let msg = format!(
                "Event type has an invalid header value: {}",
                attempt.event_type_name
            );
            warn!("{msg}");
            Response {
                response_error: Some(ResponseError::InvalidHeader),
                http_code: None,
                headers: None,
                body: Some(msg.into_bytes()),
                body_truncated: false,
                elapsed_time: start.elapsed(),
            }
        }
    }
}

/// What was kept of a target's response body.
struct CappedBody {
    /// `None` when there is no body to speak of: reading is disabled, or the read failed before
    /// a single byte arrived. It can be empty though, when all that arrived was half a character.
    bytes: Option<Vec<u8>>,
    /// The target sent more than the limit, and `bytes` stops at the limit (or up to 3 bytes
    /// before it, when the limit fell inside a character)
    truncated: bool,
}

/// Read a response body, keeping at most `limit` bytes of it.
///
/// How much a target sends is up to the target, so the body is never collected as a whole: it
/// is read through a reader that stops one byte past the limit. That extra byte is what tells a
/// body that fits exactly from one that does not. Reading stops there, and dropping the reader
/// closes the connection: the target only gets to send what the HTTP client and the kernel had
/// buffered ahead of the reader, a small fraction of a large body.
///
/// A read that fails midway (the target hangs up, or the request times out) keeps what had
/// arrived by then. Whether a delivery succeeded is settled by the status line alone, so a body
/// cut short is still worth recording.
///
/// A body that stops early, at the limit or because the read failed, may end in the middle of a
/// character. That half character is dropped: the API returns a body that is not valid UTF-8 as
/// base64, which would turn a truncated JSON body into an unreadable blob.
///
/// What is kept holds no more memory than its own length, since it is copied again on its way
/// to storage.
///
/// A limit of zero means bodies are not read at all, rather than waited for and thrown away.
async fn read_capped_body(res: reqwest::Response, limit: u64) -> CappedBody {
    if limit == 0 {
        CappedBody {
            bytes: None,
            truncated: false,
        }
    } else {
        let cap = limit.saturating_add(1);
        // A body that announces its length is read into a buffer of that size, instead of one
        // that doubles as it fills. Bounded by `cap`, so announcing more than it sends does not
        // get a target more memory than the limit.
        let announced = res.content_length().map_or(0, |len| len.min(cap));
        let stream = Box::pin(res.bytes_stream().map_err(std::io::Error::other));
        let mut reader = StreamReader::new(stream).take(cap);

        let mut bytes = Vec::with_capacity(usize::try_from(announced).unwrap_or(0));
        // When this fails, what was read before the failure is still in `bytes`
        let read = reader.read_to_end(&mut bytes).await;
        if let Err(e) = &read {
            debug!("Could not read the whole webhook response body: {e}");
        }

        // Settled before trimming: bytes did arrive, even if all of them were half a character
        let arrived = read.is_ok() || !bytes.is_empty();

        let limit = usize::try_from(limit).unwrap_or(usize::MAX);
        let truncated = bytes.len() > limit;
        bytes.truncate(limit);
        // A body that arrived whole is kept exactly as the target sent it
        if truncated || read.is_err() {
            trim_partial_utf8_tail(&mut bytes);
        }
        // A body that did not announce its length was read into a buffer that doubled as it
        // filled, up to twice the limit: that slack is not carried into storage
        bytes.shrink_to_fit();

        CappedBody {
            bytes: arrived.then_some(bytes),
            truncated,
        }
    }
}

/// Drop a multi-byte character cut in half at the end of `bytes`, so that a text body cut short
/// is still text. Only an incomplete trailing sequence is removed (`error_len()` is `None`).
///
/// This runs whatever the body is, because the API decides between text and base64 on UTF-8
/// validity, not on a Content-Type the target controls. Binary data is left alone: it has an
/// invalid byte long before its end, where `from_utf8` stops (`error_len()` is `Some`). A body is
/// only altered when everything before its last 1 to 3 bytes is valid UTF-8, which is exactly
/// when the API would show it as text.
fn trim_partial_utf8_tail(bytes: &mut Vec<u8>) {
    if let Err(e) = std::str::from_utf8(bytes)
        && e.error_len().is_none()
    {
        bytes.truncate(e.valid_up_to());
    }
}

fn mk_http_client(
    connect_timeout: Duration,
    timeout: Duration,
    // When set, pins DNS resolution of `host` to the already-vetted addresses so reqwest
    // cannot re-resolve the hostname to a different IP than the one we checked.
    pin: Option<(&str, &[SocketAddr])>,
) -> reqwest::Result<Client> {
    let mut builder = Client::builder()
        .connection_verbose(true)
        .connect_timeout(connect_timeout)
        .timeout(timeout)
        .user_agent(USER_AGENT)
        .tcp_keepalive(None)
        // Do not follow redirects: a target could 3xx to an internal/loopback/metadata URL
        // that would bypass the target-IP guard (SSRF).
        .redirect(reqwest::redirect::Policy::none());

    // NOTE: an egress HTTP proxy (HTTP_PROXY/HTTPS_PROXY/ALL_PROXY) is intentionally still
    // honored from the environment. When a proxy is configured it performs its own DNS
    // resolution, which bypasses this pin and the target-IP guard; that is the operator's
    // trusted egress path and their responsibility.

    if let Some((host, addrs)) = pin {
        builder = builder.resolve_to_addrs(host, addrs);
    }

    builder.build()
}

struct RedactedHeaders<'a> {
    headers: &'a HeaderMap,
    safe_headers: &'a [HeaderName],
}

impl fmt::Debug for RedactedHeaders<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();
        for (name, value) in self.headers.iter() {
            if self.safe_headers.iter().any(|s| s == name) {
                map.entry(&name.as_str(), &value);
            } else {
                map.entry(&name.as_str(), &"[REDACTED]");
            }
        }
        map.finish()
    }
}

/// Parse headers of HTTP target from JSON and prepare them to be fed to reqwest
fn parse_headers(value: Value) -> anyhow::Result<HeaderMap> {
    let hashmap = serde_json::from_value::<HashMap<String, String>>(value)?;
    let headermap = HeaderMap::try_from(&hashmap)?;
    Ok(headermap)
}

struct Signature {
    pub timestamp: i64,
    pub headers: String,
    pub v0: String,
    pub v1: String,
}

impl Signature {
    const PAYLOAD_SEPARATOR: &'static str = ".";
    const PAYLOAD_SEPARATOR_BYTES: &'static [u8] = Self::PAYLOAD_SEPARATOR.as_bytes();
    const SIGNATURE_PART_ASSIGNATOR: char = '=';
    const SIGNATURE_PART_SEPARATOR: &'static str = ",";
    const SIGNATURE_PART_HEADER_NAMES_SEPARATOR: &'static str = " ";

    pub fn new(
        secret: &str,
        payload: &[u8],
        signed_at: DateTime<Utc>,
        headers: &HeaderMap,
    ) -> Result<Self, HeaderName> {
        let timestamp = signed_at.timestamp();
        let timestamp_str = timestamp.to_string();
        let timestamp_str_bytes = timestamp_str.as_bytes();

        let sorted_headers_with_lowercased_names = {
            let mut hs = headers
                .iter()
                .map(|(k, v)| {
                    v.to_str()
                        .map(|str| (k.as_str().to_lowercase(), str))
                        .map_err(|_| k)
                })
                .collect::<Result<Vec<_>, _>>()?;
            hs.sort_by_key(|e| e.0.to_owned());
            hs
        };

        type HmacSha256 = Hmac<Sha256>;
        let mut mac_v0 = HmacSha256::new_from_slice(secret.as_bytes()).unwrap(); // MAC can take key of any size; this should never fail
        mac_v0.update(timestamp_str_bytes);
        mac_v0.update(Self::PAYLOAD_SEPARATOR_BYTES);

        let mut mac_v1 = mac_v0.clone();

        mac_v0.update(payload);
        let v0 = mac_v0.finalize().into_bytes().encode_hex::<String>();

        let header_names = sorted_headers_with_lowercased_names
            .iter()
            .map(|(k, _v)| k.as_str())
            .collect::<Vec<_>>()
            .join(Self::SIGNATURE_PART_HEADER_NAMES_SEPARATOR);

        mac_v1.update(header_names.as_bytes());
        mac_v1.update(Self::PAYLOAD_SEPARATOR_BYTES);

        mac_v1.update(
            sorted_headers_with_lowercased_names
                .iter()
                .map(|(_k, v)| *v)
                .collect::<Vec<_>>()
                .join(Self::PAYLOAD_SEPARATOR)
                .as_bytes(),
        );
        mac_v1.update(Self::PAYLOAD_SEPARATOR_BYTES);

        mac_v1.update(payload);
        let v1 = mac_v1.finalize().into_bytes().encode_hex::<String>();

        Ok(Self {
            timestamp,
            headers: header_names,
            v0,
            v1,
        })
    }

    pub fn value(&self, v0_enabled: bool, v1_enabled: bool) -> String {
        let timestamp_str = self.timestamp.to_string();
        let mut parts = vec![("t", timestamp_str.as_str())];

        if v0_enabled {
            parts.push(("v0", self.v0.as_str()));
        }

        if v1_enabled {
            parts.push(("h", self.headers.as_str()));
            parts.push(("v1", self.v1.as_str()));
        }

        itertools::Itertools::intersperse(
            parts
                .iter()
                .map(|p| format!("{}{}{}", p.0, Self::SIGNATURE_PART_ASSIGNATOR, p.1)),
            Self::SIGNATURE_PART_SEPARATOR.to_owned(),
        )
        .collect::<String>()
    }

    pub fn to_header_value(
        &self,
        v0_enabled: bool,
        v1_enabled: bool,
    ) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::from_str(&self.value(v0_enabled, v1_enabled))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::prelude::*;
    use proptest::prelude::*;
    use std::net::Ipv4Addr;
    use tokio::io::AsyncWriteExt;
    use tokio::net::{TcpListener, TcpStream};
    use tokio::task::JoinHandle;
    use tokio::time::{sleep, timeout};

    /// A target picks its own response headers, and HTTP lets a value carry bytes outside visible
    /// ASCII. Reading those headers back must yield the value, not bring the delivery down.
    #[test]
    fn response_headers_read_a_value_that_is_not_visible_ascii() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-reason",
            HeaderValue::from_bytes("café".as_bytes()).expect("a legal header value"),
        );

        let response = Response {
            response_error: None,
            http_code: Some(200),
            headers: Some(headers),
            body: None,
            body_truncated: false,
            elapsed_time: Duration::from_millis(1),
        };

        let read = response.headers().expect("headers must be readable");
        assert_eq!(read["x-reason"].as_str(), Some("café"));
    }

    /// The bytes a target sends need not be UTF-8 at all. What cannot be decoded is replaced, so
    /// the readable part of the value still reaches the record.
    #[test]
    fn response_headers_read_a_value_that_is_not_utf8() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-reason",
            HeaderValue::from_bytes(b"caf\xe9").expect("a legal header value"),
        );

        let response = Response {
            response_error: None,
            http_code: Some(200),
            headers: Some(headers),
            body: None,
            body_truncated: false,
            elapsed_time: Duration::from_millis(1),
        };

        let read = response.headers().expect("headers must be readable");
        assert_eq!(read["x-reason"].as_str(), Some("caf\u{FFFD}"));
    }

    /// Long enough that no test waits for it; reaching it means something is stuck
    const A_LONG_TIME: Duration = Duration::from_secs(10);

    /// A recognisable body, so that what was kept can be compared with what was sent
    fn body_of(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i % 251) as u8).collect()
    }

    /// The head of a `200` announcing a body of `len` bytes
    fn head_announcing(len: usize) -> Vec<u8> {
        format!("HTTP/1.1 200 OK\r\nContent-Length: {len}\r\n\r\n").into_bytes()
    }

    /// A target for one request: accepts a single connection, swallows the request head, then
    /// lets `respond` write whatever that target answers. The connection closes when `respond`
    /// is done, and the handle yields what it returned.
    async fn target<F, Fut, T>(respond: F) -> (SocketAddr, JoinHandle<T>)
    where
        F: FnOnce(TcpStream) -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send,
        T: Send + 'static,
    {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("a local port to listen on");
        let addr = listener.local_addr().expect("the listener's address");

        let handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("an incoming connection");

            let mut request_head = Vec::new();
            let mut buf = [0u8; 1024];
            let mut reading = true;
            while reading {
                let n = stream.read(&mut buf).await.expect("the request head");
                request_head.extend_from_slice(&buf[..n]);
                reading = n > 0 && !request_head.windows(4).any(|w| w == b"\r\n\r\n");
            }

            respond(stream).await
        });

        (addr, handle)
    }

    /// A target that answers with the whole of `body`, announced for what it is
    async fn target_answering(body: Vec<u8>) -> SocketAddr {
        let (addr, _) = target(move |mut stream: TcpStream| async move {
            // The caller may hang up before the end, and that is not this target's failure
            let _ = stream.write_all(&head_announcing(body.len())).await;
            let _ = stream.write_all(&body).await;
        })
        .await;
        addr
    }

    /// The client deliveries are made with, so that the body is read the way a delivery reads it
    fn client(timeout: Duration) -> Client {
        mk_http_client(Duration::from_secs(5), timeout, None).expect("an HTTP client")
    }

    async fn call(client: &Client, addr: SocketAddr) -> reqwest::Response {
        client
            .get(format!("http://{addr}/"))
            .send()
            .await
            .expect("the head of a response")
    }

    #[tokio::test]
    async fn a_body_under_the_limit_is_kept_whole() {
        let body = body_of(1000);
        let addr = target_answering(body.clone()).await;
        let client = client(A_LONG_TIME);

        let kept = read_capped_body(call(&client, addr).await, 1001).await;

        assert_eq!(kept.bytes, Some(body));
        assert!(!kept.truncated);
    }

    /// The limit is the largest body kept whole, not the first one cut
    #[tokio::test]
    async fn a_body_exactly_at_the_limit_is_kept_whole() {
        let body = body_of(1000);
        let addr = target_answering(body.clone()).await;
        let client = client(A_LONG_TIME);

        let kept = read_capped_body(call(&client, addr).await, 1000).await;

        assert_eq!(kept.bytes, Some(body));
        assert!(!kept.truncated);
    }

    #[tokio::test]
    async fn a_body_one_byte_over_the_limit_is_truncated() {
        let body = body_of(1000);
        let addr = target_answering(body.clone()).await;
        let client = client(A_LONG_TIME);

        let kept = read_capped_body(call(&client, addr).await, 999).await;

        assert_eq!(kept.bytes.as_deref(), Some(&body[..999]));
        assert!(kept.truncated);
    }

    /// What the limit is for: a target that announces far more than the limit does not get to
    /// send it. The worker stops reading and hangs up, and the target is left with most of its
    /// body unsent.
    #[tokio::test]
    async fn a_body_far_over_the_limit_is_not_downloaded() {
        const ANNOUNCED: usize = 64 * 1024 * 1024;
        const LIMIT: usize = 1024;

        let (addr, sent) = target(|mut stream: TcpStream| async move {
            let chunk = vec![0u8; 64 * 1024];
            let mut sent = 0;
            let mut writing = stream.write_all(&head_announcing(ANNOUNCED)).await.is_ok();
            while writing {
                if stream.write_all(&chunk).await.is_ok() {
                    sent += chunk.len();
                    writing = sent < ANNOUNCED;
                } else {
                    writing = false;
                }
            }
            sent
        })
        .await;
        let client = client(A_LONG_TIME);

        let kept = read_capped_body(call(&client, addr).await, LIMIT as u64).await;

        let bytes = kept.bytes.expect("the beginning of the body");
        assert_eq!(bytes.len(), LIMIT);
        assert!(
            bytes.capacity() <= LIMIT,
            "what is kept must not hold more memory than the limit, got {} bytes",
            bytes.capacity()
        );
        assert!(kept.truncated);

        let sent = timeout(A_LONG_TIME, sent)
            .await
            .expect("the target must be hung up on")
            .expect("the target must not panic");
        assert!(
            sent < ANNOUNCED / 2,
            "the target got to send {sent} bytes of the {ANNOUNCED} it announced"
        );
    }

    /// A body does not have to announce its length, so the limit cannot rely on being told one
    #[tokio::test]
    async fn a_chunked_body_over_the_limit_is_truncated() {
        let body = body_of(16 * 1024);
        let sent = body.clone();
        let (addr, _) = target(move |mut stream: TcpStream| async move {
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n")
                .await;
            for piece in sent.chunks(1024) {
                let _ = stream
                    .write_all(format!("{:x}\r\n", piece.len()).as_bytes())
                    .await;
                let _ = stream.write_all(piece).await;
                let _ = stream.write_all(b"\r\n").await;
            }
            let _ = stream.write_all(b"0\r\n\r\n").await;
        })
        .await;
        let client = client(A_LONG_TIME);

        let kept = read_capped_body(call(&client, addr).await, 1500).await;

        let bytes = kept.bytes.expect("the beginning of the body");
        assert_eq!(bytes, body[..1500]);
        // Nothing announced how much was coming, so the buffer grew as it filled
        assert!(
            bytes.capacity() <= 1500,
            "what is kept must not hold more memory than the limit, got {} bytes",
            bytes.capacity()
        );
        assert!(kept.truncated);
    }

    /// The API returns a body that is not valid UTF-8 as base64, so a cut in the middle of a
    /// character would turn the text that was kept into an unreadable blob
    #[tokio::test]
    async fn a_text_body_over_the_limit_is_not_cut_inside_a_character() {
        let text = "é".repeat(1000);
        let addr = target_answering(text.clone().into_bytes()).await;
        let client = client(A_LONG_TIME);

        // Each "é" takes 2 bytes: the limit falls in the middle of the 501st
        let kept = read_capped_body(call(&client, addr).await, 1001).await;

        let bytes = kept.bytes.expect("the beginning of the body");
        assert_eq!(String::from_utf8(bytes).as_deref(), Ok(&text[..1000]));
        assert!(kept.truncated);
    }

    #[tokio::test]
    async fn a_text_body_cut_short_inside_a_character_keeps_whole_characters() {
        let (addr, _) = target(|mut stream: TcpStream| async move {
            let _ = stream.write_all(&head_announcing(100)).await;
            // The first byte of "é", then the target hangs up
            let _ = stream.write_all(b"caf\xC3").await;
        })
        .await;
        let client = client(A_LONG_TIME);

        let kept = read_capped_body(call(&client, addr).await, 1000).await;

        assert_eq!(kept.bytes.as_deref(), Some(&b"caf"[..]));
        assert!(!kept.truncated);
    }

    /// Only a body that Hook0 cut is trimmed: one that arrived whole is what the target sent
    #[tokio::test]
    async fn a_complete_body_is_kept_verbatim_even_if_not_utf8() {
        let addr = target_answering(b"caf\xC3".to_vec()).await;
        let client = client(A_LONG_TIME);

        let kept = read_capped_body(call(&client, addr).await, 1000).await;

        assert_eq!(kept.bytes.as_deref(), Some(&b"caf\xC3"[..]));
        assert!(!kept.truncated);
    }

    /// Binary data has an invalid byte long before its end, and is left as it is
    #[test]
    fn trimming_leaves_a_binary_body_alone() {
        let mut bytes = vec![0xFF, b'a', 0xC3];

        trim_partial_utf8_tail(&mut bytes);

        assert_eq!(bytes, [0xFF, b'a', 0xC3]);
    }

    proptest! {
        /// Wherever a text is cut, what is left after trimming is every character that the cut
        /// left whole, and nothing else
        #[test]
        fn trimming_a_cut_text_keeps_its_whole_characters(
            text in ".{0,64}",
            cut in any::<prop::sample::Index>(),
        ) {
            let cut = cut.index(text.len() + 1);
            let mut bytes = text.as_bytes()[..cut].to_vec();

            trim_partial_utf8_tail(&mut bytes);

            let boundary = (0..=cut)
                .rev()
                .find(|&i| text.is_char_boundary(i))
                .expect("the start of a text is always a character boundary");
            prop_assert_eq!(bytes, text.as_bytes()[..boundary].to_vec());
        }
    }

    /// The status line already said how the delivery went, so the part of a body that made it
    /// through before the target hung up is still recorded.
    #[tokio::test]
    async fn a_body_cut_short_by_the_target_keeps_what_arrived() {
        let (addr, _) = target(|mut stream: TcpStream| async move {
            let _ = stream.write_all(&head_announcing(100)).await;
            let _ = stream.write_all(b"partial").await;
        })
        .await;
        let client = client(A_LONG_TIME);

        let kept = read_capped_body(call(&client, addr).await, 1000).await;

        assert_eq!(kept.bytes.as_deref(), Some(&b"partial"[..]));
        assert!(!kept.truncated);
    }

    /// Nothing at all arrived, which is not the same as an empty body: there is no body
    #[tokio::test]
    async fn a_body_that_never_starts_is_no_body() {
        let (addr, _) = target(|mut stream: TcpStream| async move {
            let _ = stream.write_all(&head_announcing(100)).await;
        })
        .await;
        let client = client(A_LONG_TIME);

        let kept = read_capped_body(call(&client, addr).await, 1000).await;

        assert_eq!(kept.bytes, None);
        assert!(!kept.truncated);
    }

    #[tokio::test]
    async fn an_empty_body_is_an_empty_body() {
        let addr = target_answering(Vec::new()).await;
        let client = client(A_LONG_TIME);

        let kept = read_capped_body(call(&client, addr).await, 1000).await;

        assert_eq!(kept.bytes, Some(Vec::new()));
        assert!(!kept.truncated);
    }

    #[tokio::test]
    async fn a_body_slower_than_the_timeout_keeps_what_arrived() {
        let (addr, _) = target(|mut stream: TcpStream| async move {
            let _ = stream.write_all(&head_announcing(100)).await;
            let _ = stream.write_all(b"slow").await;
            // The rest never comes, and the connection stays open
            sleep(A_LONG_TIME).await;
        })
        .await;
        let client = client(Duration::from_secs(1));

        let kept = read_capped_body(call(&client, addr).await, 1000).await;

        assert_eq!(kept.bytes.as_deref(), Some(&b"slow"[..]));
        assert!(!kept.truncated);
    }

    /// With a limit of zero the body is not wanted, so it is not waited for either
    #[tokio::test]
    async fn a_limit_of_zero_does_not_wait_for_the_body() {
        let (addr, _) = target(|mut stream: TcpStream| async move {
            let _ = stream.write_all(&head_announcing(100)).await;
            sleep(A_LONG_TIME).await;
        })
        .await;
        let client = client(A_LONG_TIME);
        let res = call(&client, addr).await;

        let kept = timeout(Duration::from_secs(2), read_capped_body(res, 0))
            .await
            .expect("a body that is not wanted must not be waited for");

        assert_eq!(kept.bytes, None);
        assert!(!kept.truncated);
    }

    #[test]
    fn create_signature_v0() {
        let signed_at = Utc.with_ymd_and_hms(2021, 11, 15, 0, 30, 0).unwrap();
        let payload = "hello !";
        let secret = "secret";

        let sig = Signature::new(secret, payload.as_bytes(), signed_at, &HeaderMap::new()).unwrap();
        assert_eq!(
            sig.value(true, false),
            "t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98"
        );
    }

    #[test]
    fn create_signature_v1() {
        let signed_at = Utc.with_ymd_and_hms(2021, 11, 15, 0, 30, 0).unwrap();
        let payload = "hello !";
        let secret = "secret";
        let mut headers = HeaderMap::new();
        // Signature must be consistant and ignore header name's case and order
        headers.insert(
            "X-EVENT-TYPE",
            HeaderValue::from_str("service.resource.verb").expect("Invalid header values"),
        );
        headers.insert(
            "X-Event-Id",
            HeaderValue::from_str("1a01cb48-5142-4d9b-8f90-d20cca61f0ee")
                .expect("Invalid header values"),
        );

        let sig = Signature::new(secret, payload.as_bytes(), signed_at, &headers).unwrap();
        assert_eq!(
            sig.value(false, true),
            "t=1636936200,h=x-event-id x-event-type,v1=bc521546ba5de381b12f135782d2008b028c3065c191760b12b76850a8fc8f51"
        );
    }

    #[test]
    fn create_signature_v0_and_v1() {
        let signed_at = Utc.with_ymd_and_hms(2021, 11, 15, 0, 30, 0).unwrap();
        let payload = "hello !";
        let secret = "secret";
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Event-Id",
            HeaderValue::from_str("1a01cb48-5142-4d9b-8f90-d20cca61f0ee")
                .expect("Invalid header values"),
        );
        headers.insert(
            "X-Event-Type",
            HeaderValue::from_str("service.resource.verb").expect("Invalid header values"),
        );

        let sig = Signature::new(secret, payload.as_bytes(), signed_at, &headers).unwrap();
        assert_eq!(
            sig.value(true, true),
            "t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98,h=x-event-id x-event-type,v1=bc521546ba5de381b12f135782d2008b028c3065c191760b12b76850a8fc8f51"
        );
    }

    #[test]
    fn create_signature_wrong_header() {
        let signed_at = Utc.with_ymd_and_hms(2021, 11, 15, 0, 30, 0).unwrap();
        let payload = "hello !";
        let secret = "secret";
        let mut headers = HeaderMap::new();
        // Signature must be consistant and ignore header name's case and order
        headers.insert(
            "X-EVENT-TYPE",
            HeaderValue::from_str("pj.cartão.autorização").expect("Invalid header values"),
        );

        let sig = Signature::new(secret, payload.as_bytes(), signed_at, &headers);
        assert!(matches!(sig, Err(h) if h == HeaderName::from_static("x-event-type")));
    }
}
