use chrono::{DateTime, Utc};
use clap::{crate_name, crate_version};
use hex::ToHex;
use hmac::{Hmac, Mac};
use log::{debug, error, trace, warn};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, InvalidHeaderValue};
use reqwest::{Client, Method, Url};
use sha2::Sha256;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::time::{Duration, Instant};
use strum::VariantNames;

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
    pub body: Option<String>,
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

pub async fn work(config: &Config, attempt: &RequestAttempt) -> Response {
    debug!(
        "Processing request attempt {}",
        &attempt.request_attempt__id
    );
    let start = Instant::now();

    let m = Method::from_str(attempt.http_method.as_str());
    let u = Url::parse(attempt.http_url.as_str())
        .map_err(|e| e.to_string())
        .and_then(|url| {
            let addrs = url.socket_addrs(|| None).map_err(|e| e.to_string())?;
            let has_forbidden_ip = addrs.iter().all(|addr| {
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

                match addr.ip() {
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
                    },
                }
            });

            if has_forbidden_ip {
                if config.disable_target_ip_check {
                    debug!("Target URL resolves to a forbidden IP but this is allowed in the worker's configuration");
                    Ok(url)
                } else {
                    Err("URL resolves to a forbidden IP".to_string())
                }
            } else {
                Ok(url)
            }
        });
    let c = mk_http_client(config.connect_timeout, config.timeout);
    let hs = attempt.headers();
    let et = HeaderValue::from_str(&attempt.event_type__name);
    let event_id = HeaderValue::from_str(attempt.event__id.to_string().as_str())
        .expect("Could not create a header value from the event ID UUID");
    let content_type = HeaderValue::from_str(attempt.payload_content_type.as_str())
        .expect("Could not create a header value from the event content type");

    match (m, u, c, hs, et) {
        (Ok(method), Ok(url), Ok(client), Ok(mut headers), Ok(et)) => {
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
                Response {
                    response_error: Some(ResponseError::InvalidHeader),
                    http_code: None,
                    headers: None,
                    body: Some(msg),
                    elapsed_time: start.elapsed(),
                }
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
                .map_err(|_| Response {
                    response_error: Some(ResponseError::InvalidHeader),
                    http_code: None,
                    headers: None,
                    body: None,
                    elapsed_time: start.elapsed(),
                })
            });

            match s {
                Ok(sig) => {
                    headers.insert(&config.signature_header_name, sig);

                    debug!("Calling webhook...");
                    trace!(
                        "HTTP {} {url} {headers:?}",
                        &method.to_string().to_uppercase(),
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
                            let body = res.text().await.ok();

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
                                warn!("Webhook call failed with HTTP code {status}");
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
                                body: Some(e.to_string()),
                                elapsed_time: start.elapsed(),
                            }
                        }
                        Err(e) if e.is_timeout() => {
                            warn!("Webhook call failed with timeout error: {e}");
                            Response {
                                response_error: Some(ResponseError::Timeout),
                                http_code: None,
                                headers: None,
                                body: Some(e.to_string()),
                                elapsed_time: start.elapsed(),
                            }
                        }
                        Err(e) => {
                            warn!("Webhook call failed with unknown error: {e}");
                            Response {
                                response_error: Some(ResponseError::Unknown),
                                http_code: None,
                                headers: None,
                                body: Some(e.to_string()),
                                elapsed_time: start.elapsed(),
                            }
                        }
                    }
                }
                Err(e) => e,
            }
        }
        (Err(e), _, _, _, _) => {
            error!("Target has an invalid HTTP method: {e}");
            Response {
                response_error: Some(ResponseError::InvalidTarget),
                http_code: None,
                headers: None,
                body: Some(e.to_string()),
                elapsed_time: start.elapsed(),
            }
        }
        (_, Err(e), _, _, _) => {
            warn!("Target has an invalid URL: {e}");
            Response {
                response_error: Some(ResponseError::InvalidTarget),
                http_code: None,
                headers: None,
                body: Some(e.to_string()),
                elapsed_time: start.elapsed(),
            }
        }
        (_, _, Err(e), _, _) => {
            error!("Could not create HTTP client: {e}");
            Response {
                response_error: Some(ResponseError::Unknown),
                http_code: None,
                headers: None,
                body: Some(e.to_string()),
                elapsed_time: start.elapsed(),
            }
        }
        (_, _, _, Err(e), _) => {
            warn!("Target has invalid headers: {e}");
            Response {
                response_error: Some(ResponseError::InvalidTarget),
                http_code: None,
                headers: None,
                body: Some(e.to_string()),
                elapsed_time: start.elapsed(),
            }
        }
        (_, _, _, _, Err(_)) => {
            let msg = format!(
                "Event type has an invalid header value: {}",
                &attempt.event_type__name
            );
            warn!("{msg}");
            Response {
                response_error: Some(ResponseError::InvalidHeader),
                http_code: None,
                headers: None,
                body: Some(msg),
                elapsed_time: start.elapsed(),
            }
        }
    }
}

fn mk_http_client(connect_timeout: Duration, timeout: Duration) -> reqwest::Result<Client> {
    Client::builder()
        .connection_verbose(true)
        .connect_timeout(connect_timeout)
        .timeout(timeout)
        .user_agent(USER_AGENT)
        .tcp_keepalive(None)
        .build()
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
}
