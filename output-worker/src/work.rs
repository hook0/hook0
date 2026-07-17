use chrono::{DateTime, Utc};
use clap::{crate_name, crate_version};
use hex::ToHex;
use hmac::{Hmac, KeyInit, Mac};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, InvalidHeaderValue};
use reqwest::{Client, Method, Url};
use serde_json::Value;
use sha2::Sha256;
use std::collections::HashMap;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use std::time::{Duration, Instant};
use strum::VariantNames;
use tracing::{debug, error, instrument, trace, warn};

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
    #[strum(serialize = "E_CONNECTION")]
    Connection,
    #[strum(serialize = "E_TIMEOUT")]
    Timeout,
    #[strum(serialize = "E_HTTP")]
    Http,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub response_error: Option<ResponseError>,
    pub http_code: Option<u16>,
    pub headers: Option<HeaderMap>,
    pub body: Option<Vec<u8>>,
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

    pub fn headers(&self) -> Option<serde_json::Value> {
        self.headers.as_ref().and_then(|hm| {
            let iter = hm
                .iter()
                .map(|(k, v)| {
                    let key = k.to_string();
                    let value = v
                        .to_str()
                        .expect("Failed to extract a HTTP header value (there might be invisible characters)")
                        .to_owned();
                    (key, value)
                });
            let hashmap: HashMap<String, String> = iter.collect();
            serde_json::to_value(hashmap).ok()
        })
    }

    pub fn elapsed_time_ms(&self) -> i32 {
        self.elapsed_time.as_millis().try_into().unwrap_or(0)
    }
}

#[instrument(skip_all, fields(request_attempt_id = %attempt.request_attempt_id))]
pub async fn work(config: &Config, attempt: &RequestAttempt) -> Response {
    debug!("Processing request attempt");
    let start = Instant::now();

    let m = Method::from_str(attempt.http_method.as_str());
    let u = Url::parse(attempt.http_url.as_str())
        .map_err(|e| e.to_string())
        .and_then(|url| {
            let addrs = url.socket_addrs(|| None).map_err(|e| e.to_string())?;
            if addrs.is_empty() {
                Err("URL did not resolve to any IP address".to_string())
            } else {
                // Reject if *any* resolved address is forbidden: a hostname that resolves to a mix of public and internal addresses must not pass.
                let has_forbidden_ip = addrs.iter().any(|addr| is_forbidden_ip(addr.ip()));

                if has_forbidden_ip {
                    if config.disable_target_ip_check {
                        debug!("Target URL resolves to a forbidden IP but this is allowed in the worker's configuration");
                        Ok((url, addrs))
                    } else {
                        Err("URL resolves to a forbidden IP".to_string())
                    }
                } else {
                    Ok((url, addrs))
                }
            }
        });
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
                        response_error: Some(ResponseError::Unknown),
                        http_code: None,
                        headers: None,
                        body: Some(e.to_string().into_bytes()),
                        elapsed_time: start.elapsed(),
                    };
                }
            };

            headers.insert("Content-Type", content_type);
            headers.insert("X-Event-Id", event_id);
            headers.insert("X-Event-Type", et);

            let s = Signature::new(
                &attempt.secret.to_string(),
                &attempt.payload,
                Utc::now(),
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
                .map_err(|_| {
                    Box::new(Response {
                        response_error: Some(ResponseError::InvalidHeader),
                        http_code: None,
                        headers: None,
                        body: None,
                        elapsed_time: start.elapsed(),
                    })
                })
            });

            match s {
                Ok(sig) => {
                    headers.insert(&config.signature_header_name, sig);

                    debug!("Calling webhook...");
                    let redacted_headers = RedactedHeaders {
                        headers: &headers,
                        safe_headers: &[
                            HeaderName::from_static("content-type"),
                            HeaderName::from_static("x-event-id"),
                            HeaderName::from_static("x-event-type"),
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
                            let body = res.bytes().await.ok().map(|b| b.to_vec());

                            if status.is_success() {
                                debug!("Webhook call was successful");
                                Response {
                                    response_error: None,
                                    http_code: Some(status.as_u16()),
                                    headers: Some(headers),
                                    body,
                                    elapsed_time: start.elapsed(),
                                }
                            } else {
                                warn!(http_status = %status, "Webhook call failed with HTTP error");
                                Response {
                                    response_error: Some(ResponseError::Http),
                                    http_code: Some(status.as_u16()),
                                    headers: Some(headers),
                                    body,
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
                elapsed_time: start.elapsed(),
            }
        }
        (_, Err(e), _, _) => {
            warn!(
                target_http_url = attempt.http_url,
                "Target has an invalid URL: {e}"
            );
            Response {
                response_error: Some(ResponseError::InvalidTarget),
                http_code: None,
                headers: None,
                body: Some(e.to_string().into_bytes()),
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
                elapsed_time: start.elapsed(),
            }
        }
    }
}

/// Returns `true` when the given IP address must not be targeted by a webhook (loopback, private, link-local, shared, cloud-metadata, and other non-globally-reachable ranges).
fn is_forbidden_ip(ip: IpAddr) -> bool {
    // This should be replaced by https://doc.rust-lang.org/nightly/core/net/enum.IpAddr.html#method.is_global when it becomes stable

    // v4
    fn is_shared(ip: &Ipv4Addr) -> bool {
        ip.octets()[0] == 100 && (ip.octets()[1] & 0b1100_0000 == 0b0100_0000)
    }
    fn is_benchmarking(ip: &Ipv4Addr) -> bool {
        ip.octets()[0] == 198 && (ip.octets()[1] & 0xfe) == 18
    }
    fn is_reserved(ip: &Ipv4Addr) -> bool {
        ip.octets()[0] & 240 == 240 && !ip.is_broadcast()
    }

    // v6
    fn is_documentation(ip: &Ipv6Addr) -> bool {
        (ip.segments()[0] == 0x2001) && (ip.segments()[1] == 0xdb8)
    }
    fn is_unique_local(ip: &Ipv6Addr) -> bool {
        (ip.segments()[0] & 0xfe00) == 0xfc00
    }
    fn is_unicast_link_local(ip: &Ipv6Addr) -> bool {
        (ip.segments()[0] & 0xffc0) == 0xfe80
    }

    match ip {
        IpAddr::V4(ip) => {
            ip.octets()[0] == 0 // "This network"
                || ip.is_private()
                || is_shared(&ip)
                || ip.is_loopback()
                || ip.is_link_local()
                // addresses reserved for future protocols (`192.0.0.0/24`)
                ||(ip.octets()[0] == 192 && ip.octets()[1] == 0 && ip.octets()[2] == 0)
                || ip.is_documentation()
                || is_benchmarking(&ip)
                || is_reserved(&ip)
                || ip.is_broadcast()
        }
        IpAddr::V6(ip) => {
            ip.is_unspecified()
                || ip.is_loopback()
                // IPv4-mapped Address (`::ffff:0:0/96`)
                || matches!(ip.segments(), [0, 0, 0, 0, 0, 0xffff, _, _])
                // IPv4-IPv6 Translat. (`64:ff9b:1::/48`)
                || matches!(ip.segments(), [0x64, 0xff9b, 1, _, _, _, _, _])
                // Discard-Only Address Block (`100::/64`)
                || matches!(ip.segments(), [0x100, 0, 0, 0, _, _, _, _])
                // IETF Protocol Assignments (`2001::/23`)
                || (matches!(ip.segments(), [0x2001, b, _, _, _, _, _, _] if b < 0x200)
                    && !(
                        // Port Control Protocol Anycast (`2001:1::1`)
                        u128::from_be_bytes(ip.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0001
                        // Traversal Using Relays around NAT Anycast (`2001:1::2`)
                        || u128::from_be_bytes(ip.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0002
                        // AMT (`2001:3::/32`)
                        || matches!(ip.segments(), [0x2001, 3, _, _, _, _, _, _])
                        // AS112-v6 (`2001:4:112::/48`)
                        || matches!(ip.segments(), [0x2001, 4, 0x112, _, _, _, _, _])
                        // ORCHIDv2 (`2001:20::/28`)
                        || matches!(ip.segments(), [0x2001, b, _, _, _, _, _, _] if (0x20..=0x2F).contains(&b))
                    ))
                || is_documentation(&ip)
                || is_unique_local(&ip)
                || is_unicast_link_local(&ip)
        }
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

    fn ip(s: &str) -> IpAddr {
        s.parse().expect("invalid test IP")
    }

    #[test]
    fn forbids_non_globally_reachable_ips() {
        // IPv4
        assert!(is_forbidden_ip(ip("0.0.0.0")));
        assert!(is_forbidden_ip(ip("127.0.0.1")));
        assert!(is_forbidden_ip(ip("10.0.0.1")));
        assert!(is_forbidden_ip(ip("172.16.5.4")));
        assert!(is_forbidden_ip(ip("192.168.1.1")));
        assert!(is_forbidden_ip(ip("100.64.0.1"))); // shared (CGNAT)
        assert!(is_forbidden_ip(ip("169.254.1.1"))); // link-local
        assert!(is_forbidden_ip(ip("169.254.169.254"))); // cloud metadata
        assert!(is_forbidden_ip(ip("255.255.255.255"))); // broadcast
        // IPv6
        assert!(is_forbidden_ip(ip("::1"))); // loopback
        assert!(is_forbidden_ip(ip("::"))); // unspecified
        assert!(is_forbidden_ip(ip("fc00::1"))); // unique local
        assert!(is_forbidden_ip(ip("fe80::1"))); // link-local
        assert!(is_forbidden_ip(ip("::ffff:127.0.0.1"))); // IPv4-mapped loopback
        assert!(is_forbidden_ip(ip("::ffff:169.254.169.254"))); // IPv4-mapped metadata
        assert!(is_forbidden_ip(ip("64:ff9b:1::1"))); // IPv4/IPv6 translation
        assert!(is_forbidden_ip(ip("100::1"))); // discard-only block
        // IETF Protocol Assignments (`2001::/23`), excluding the globally-reachable carve-outs
        assert!(is_forbidden_ip(ip("2001::1"))); // generic 2001::/23 (Teredo region)
        assert!(is_forbidden_ip(ip("2001:1ff::1"))); // top of the /23 (b == 0x1ff)
        assert!(is_forbidden_ip(ip("2001:db8::1"))); // documentation
    }

    #[test]
    fn allows_public_ips() {
        assert!(!is_forbidden_ip(ip("1.1.1.1")));
        assert!(!is_forbidden_ip(ip("8.8.8.8")));
        assert!(!is_forbidden_ip(ip("93.184.216.34"))); // example.com
        assert!(!is_forbidden_ip(ip("2606:4700:4700::1111"))); // Cloudflare DNS
        assert!(!is_forbidden_ip(ip("2001:4860:4860::8888"))); // Google DNS
        // Globally-reachable carve-outs inside `2001::/23`
        assert!(!is_forbidden_ip(ip("2001:1::1"))); // Port Control Protocol Anycast
        assert!(!is_forbidden_ip(ip("2001:1::2"))); // TURN Anycast
        assert!(!is_forbidden_ip(ip("2001:3::1"))); // AMT
        assert!(!is_forbidden_ip(ip("2001:4:112::1"))); // AS112-v6
        assert!(!is_forbidden_ip(ip("2001:20::1"))); // ORCHIDv2 (low)
        assert!(!is_forbidden_ip(ip("2001:2f::1"))); // ORCHIDv2 (high)
        // First block just above `2001::/23` (b == 0x200) is globally reachable
        assert!(!is_forbidden_ip(ip("2001:200::1")));
    }
}
