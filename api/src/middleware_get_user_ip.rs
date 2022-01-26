use actix_web::body::BoxBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage, HttpResponse};
use futures_util::future::{ok, ready, Ready};
use ipnetwork::{IpNetwork, IpNetworkError};
use lazy_static::lazy_static;
use log::{debug, error, trace};
use regex::Regex;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::str::FromStr;
use std::task::{Context, Poll};

#[derive(Debug, Clone)]
pub struct GetUserIp {
    pub reverse_proxy_ips: Vec<String>,
}

impl<S> Transform<S, ServiceRequest> for GetUserIp
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = GetUserIpMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        trace!("Initialize GetUserIpMiddleware");
        ok(GetUserIpMiddleware {
            service: Rc::new(service),
            reverse_proxy_ips: self.reverse_proxy_ips.to_owned(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct GetUserIpMiddleware<S> {
    service: Rc<S>,
    reverse_proxy_ips: Vec<String>,
}

impl<S> Service<ServiceRequest> for GetUserIpMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        match extract_ip(&req, &self.reverse_proxy_ips) {
            Ok(ip) => {
                debug!("User IP is {}", &ip);
                {
                    let mut extensions = req.extensions_mut();
                    extensions.insert(ip);
                }
                Box::pin(self.service.call(req))
            }
            Err(err) => {
                let e = format!("GetUserIpMiddleware cannot find the user IP: {}", &err);
                error!("{}", &e);
                Box::pin(ready(Ok(
                    req.into_response(HttpResponse::InternalServerError().body(e))
                )))
            }
        }
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum GetUserIpError {
    #[error("cannot extract IP address from request")]
    NoIpInRequest,
    #[error("cannot separate IP address from port: {0}")]
    IpPortSeparation(String),
    #[error("cannot parse IP address: {0}")]
    Ip(#[from] IpNetworkError),
}

fn extract_ip(
    req: &ServiceRequest,
    reverse_proxy_ips: &[String],
) -> Result<IpNetwork, GetUserIpError> {
    let connection_info = req.connection_info();
    let peer_ip_str = req
        .peer_addr()
        .ok_or(GetUserIpError::NoIpInRequest)?
        .ip()
        .to_string();

    // Check if the IP of the direct peer is trusted
    let ip_port_str = if reverse_proxy_ips.contains(&peer_ip_str) {
        // If yes, we can get user's IP from "X-Forwarded-For" or "Forwarded" headers
        connection_info
            .realip_remote_addr()
            .ok_or(GetUserIpError::NoIpInRequest)?
    } else {
        // If no, we take the peer's IP as the user's IP
        connection_info
            .peer_addr()
            .ok_or(GetUserIpError::NoIpInRequest)?
    };

    parse_ip(ip_port_str)
}

fn parse_ip(ip_port_str: &str) -> Result<IpNetwork, GetUserIpError> {
    use nom::branch::alt;
    use nom::combinator::map_res;
    use nom::IResult;
    use nom_regex::str::re_capture;

    fn parser(input: &str) -> IResult<&str, &str> {
        lazy_static! {
            static ref RE_V4: Regex = Regex::new(r"^([^:]+)(?:[:]\d+)?$").unwrap();
            static ref RE_V6: Regex = Regex::new(r"^\[(.+)\](?:[:]\d+)?$").unwrap();
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
        .map_err(|e| GetUserIpError::IpPortSeparation(e.to_string()))?
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
            Err(GetUserIpError::IpPortSeparation(_))
        ))
    }

    #[test]
    fn parse_ip_v4_invalid_ip() {
        let input = "127.0.0.1234:5678";
        assert!(matches!(parse_ip(input), Err(GetUserIpError::Ip(_))))
    }

    #[test]
    fn parse_ip_v4_no_port() {
        let expected = IpNetwork::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 32).unwrap();
        assert_eq!(parse_ip("127.0.0.1"), Ok(expected))
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
            Err(GetUserIpError::IpPortSeparation(_))
        ))
    }

    #[test]
    fn parse_ip_v6_invalid_ip() {
        let input = "[::lol]:5678";
        assert!(matches!(parse_ip(input), Err(GetUserIpError::Ip(_))))
    }

    #[test]
    fn parse_ip_v6_no_port() {
        let expected = IpNetwork::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 128).unwrap();
        assert_eq!(parse_ip("[0:0:0:0:0:0:0:1]"), Ok(expected))
    }
}
