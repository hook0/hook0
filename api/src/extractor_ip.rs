use actix_web::web::Data;
use actix_web::{FromRequest, ResponseError};
use futures_util::future::{ready, Ready};
use ipnetwork::{IpNetwork, IpNetworkError};
use lazy_static::lazy_static;
use paperclip::actix::OperationModifier;
use paperclip::v2::schema::Apiv2Schema;
use regex::Regex;
use std::ops::Deref;
use std::str::FromStr;

use crate::State;

#[derive(Debug, Clone)]
/// Extractor for user's IP address
pub struct Ip(IpNetwork);

impl Ip {
    pub fn into_inner(self) -> IpNetwork {
        self.0
    }
}

impl Deref for Ip {
    type Target = IpNetwork;

    fn deref(&self) -> &IpNetwork {
        &self.0
    }
}

impl FromRequest for Ip {
    type Error = IpExtractorError;

    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        ready(extract_ip(req).map(Self))
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum IpExtractorError {
    #[error("cannot extract IP address from request")]
    NoIpInRequest,
    #[error("cannot separated IP address from port: {0}")]
    IpPortSeparation(String),
    #[error("cannot parse IP address: {0}")]
    Ip(#[from] IpNetworkError),
}

impl ResponseError for IpExtractorError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl Apiv2Schema for Ip {}
impl OperationModifier for Ip {}

fn extract_ip(req: &actix_web::HttpRequest) -> Result<IpNetwork, IpExtractorError> {
    let reverse_proxy_ips = req
        .app_data::<Data<State>>()
        .map(|state| state.reverse_proxy_ips.to_owned())
        .unwrap_or_else(Vec::new);
    let connection_info = req.connection_info();
    let peer_ip_str = req
        .peer_addr()
        .ok_or(IpExtractorError::NoIpInRequest)?
        .ip()
        .to_string();

    // Check if the IP of the direct peer is trusted
    let ip_port_str = if reverse_proxy_ips.contains(&peer_ip_str) {
        // If yes, we can get user's IP from "X-Forwarded-For" or "Forwarded" headers
        connection_info
            .realip_remote_addr()
            .ok_or(IpExtractorError::NoIpInRequest)?
    } else {
        // If no, we take the peer's IP as the user's IP
        connection_info
            .remote_addr()
            .ok_or(IpExtractorError::NoIpInRequest)?
    };

    parse_ip(ip_port_str)
}

fn parse_ip(ip_port_str: &str) -> Result<IpNetwork, IpExtractorError> {
    use nom::branch::alt;
    use nom::combinator::map_res;
    use nom::IResult;
    use nom_regex::str::re_capture;

    fn parser(input: &str) -> IResult<&str, &str> {
        lazy_static! {
            static ref RE_V4: Regex = Regex::new(r"^([^:]+):\d+$").unwrap();
            static ref RE_V6: Regex = Regex::new(r"^\[(.+)\]:\d+$").unwrap();
        }

        let v4 = map_res(re_capture(RE_V4.to_owned()), |captures| {
            captures.get(1).copied().ok_or_else(|| "".to_owned())
        });
        let v6 = map_res(re_capture(RE_V6.to_owned()), |captures| {
            captures.get(1).copied().ok_or_else(|| "".to_owned())
        });
        let mut p = alt((v6, v4));

        p(input)
    }

    let ip_str = parser(ip_port_str)
        .map_err(|e| IpExtractorError::IpPortSeparation(e.to_string()))?
        .1;
    let ip = IpNetwork::from_str(ip_str)?;
    Ok(ip)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn parse_ip_v4_valid() {
        let expected = IpNetwork::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 32).unwrap();
        assert_eq!(parse_ip("127.0.0.1:1234"), Ok(expected))
    }

    #[test]
    fn parse_ip_v4_invalid_separation() {
        let input = "127.0.0.1:1234:5678";
        assert!(matches!(
            parse_ip(input),
            Err(IpExtractorError::IpPortSeparation(_))
        ))
    }

    #[test]
    fn parse_ip_v4_invalid_ip() {
        let input = "127.0.0.1234:5678";
        assert!(matches!(parse_ip(input), Err(IpExtractorError::Ip(_))))
    }

    #[test]
    fn parse_ip_v6_valid() {
        let expected = IpNetwork::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 128).unwrap();
        assert_eq!(parse_ip("[0:0:0:0:0:0:0:1]:1234"), Ok(expected))
    }

    #[test]
    fn parse_ip_v6_invalid_separation() {
        let input = "[::1]:1234:5678";
        assert!(matches!(
            parse_ip(input),
            Err(IpExtractorError::IpPortSeparation(_))
        ))
    }

    #[test]
    fn parse_ip_v6_invalid_ip() {
        let input = "[::lol]:5678";
        assert!(matches!(parse_ip(input), Err(IpExtractorError::Ip(_))))
    }
}
