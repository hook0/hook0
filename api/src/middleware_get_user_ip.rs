use actix_web::body::BoxBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage, HttpResponse};
use futures_util::future::{Ready, ok, ready};
use ipnetwork::IpNetwork;
use log::{debug, error, trace};
use std::future::Future;
use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

#[derive(Debug, Clone)]
pub struct GetUserIp {
    pub reverse_proxy_ips: Vec<IpNetwork>,
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
    reverse_proxy_ips: Vec<IpNetwork>,
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
    #[error("cannot parse IP address: {0}")]
    Ip(#[from] AddrParseError),
}

fn extract_ip(
    req: &ServiceRequest,
    reverse_proxy_ips: &[IpNetwork],
) -> Result<IpAddr, GetUserIpError> {
    let connection_info = req.connection_info();
    let peer_ip = req.peer_addr().ok_or(GetUserIpError::NoIpInRequest)?.ip();

    // Check if the IP of the direct peer is trusted
    let ip_port_str = if reverse_proxy_ips
        .iter()
        .any(|whitelisted_cidr| whitelisted_cidr.contains(peer_ip))
    {
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

fn parse_ip(ip_port_str: &str) -> Result<IpAddr, GetUserIpError> {
    let ip = ip_port_str
        .parse::<SocketAddr>()
        .map(|sa| sa.ip())
        .or_else(|_| ip_port_str.parse::<IpAddr>())?;
    Ok(ip)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::extractor_user_ip::UserIp;

    use actix_web::App;
    use actix_web::body::MessageBody;
    use actix_web::dev::ServiceFactory;
    use actix_web::test::{TestRequest, call_service, init_service, read_body};
    use actix_web::web::get;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::str::FromStr;

    #[test]
    fn parse_ip_v4_valid() {
        let expected = IpAddr::V4(Ipv4Addr::LOCALHOST);
        assert_eq!(dbg!(parse_ip("127.0.0.1:1234")), Ok(expected))
    }

    #[test]
    fn parse_ip_v4_invalid_separation() {
        let input = "127.0.0.1:1234:5678";
        assert!(matches!(parse_ip(input), Err(GetUserIpError::Ip(_))))
    }

    #[test]
    fn parse_ip_v4_invalid_ip() {
        let input = "127.0.0.1234:5678";
        assert!(matches!(dbg!(parse_ip(input)), Err(GetUserIpError::Ip(_))))
    }

    #[test]
    fn parse_ip_v4_no_port() {
        let expected = IpAddr::V4(Ipv4Addr::LOCALHOST);
        assert_eq!(dbg!(parse_ip("127.0.0.1")), Ok(expected))
    }

    #[test]
    fn parse_ip_v6_valid() {
        let expected = IpAddr::V6(Ipv6Addr::LOCALHOST);
        assert_eq!(dbg!(parse_ip("[0:0:0:0:0:0:0:1]:1234")), Ok(expected))
    }

    #[test]
    fn parse_ip_v6_no_brackets() {
        let expected = IpAddr::V6(Ipv6Addr::LOCALHOST);
        assert_eq!(dbg!(parse_ip("::1")), Ok(expected))
    }

    #[test]
    fn parse_ip_v6_invalid_separation() {
        let input = "[::1]:1234:5678";
        assert!(matches!(dbg!(parse_ip(input)), Err(GetUserIpError::Ip(_))))
    }

    #[test]
    fn parse_ip_v6_invalid_ip() {
        let input = "[::lol]:5678";
        assert!(matches!(dbg!(parse_ip(input)), Err(GetUserIpError::Ip(_))))
    }

    #[test]
    fn parse_ip_v6_no_port() {
        let expected = IpAddr::V6(Ipv6Addr::LOCALHOST);
        assert_eq!(dbg!(parse_ip("0:0:0:0:0:0:0:1")), Ok(expected))
    }

    fn test_app(
        reverse_proxy_ips: Vec<IpNetwork>,
    ) -> App<
        impl ServiceFactory<
            ServiceRequest,
            Response = ServiceResponse<impl MessageBody>,
            Config = (),
            InitError = (),
            Error = Error,
        >,
    > {
        App::new()
            .wrap(GetUserIp { reverse_proxy_ips })
            .route("/", get().to(return_user_ip))
    }

    async fn return_user_ip(UserIp(ip): UserIp) -> String {
        ip.to_string()
    }

    const FORWARDED_FOR_HEADER: &str = "X-Forwarded-For";

    #[actix_web::test]
    async fn get_user_ip_v4_peer_no_trusted_proxies() {
        let app = init_service(test_app(Vec::new())).await;
        let peer = (IpAddr::V4(Ipv4Addr::LOCALHOST), 1234).into();
        let req = TestRequest::default().peer_addr(peer).to_request();
        let expected = IpAddr::V4(Ipv4Addr::LOCALHOST);

        let res = call_service(&app, req).await;
        assert_eq!(res.status(), 200);
        let body = String::from_utf8(read_body(res).await.to_vec()).unwrap();
        let user_ip = IpAddr::from_str(&body).unwrap();
        assert_eq!(user_ip, expected)
    }

    #[actix_web::test]
    async fn get_user_ip_v6_peer_no_trusted_proxies() {
        let app = init_service(test_app(Vec::new())).await;
        let peer = (IpAddr::V6(Ipv6Addr::LOCALHOST), 1234).into();
        let req = TestRequest::default().peer_addr(peer).to_request();
        let expected = IpAddr::V6(Ipv6Addr::LOCALHOST);

        let res = call_service(&app, req).await;
        assert_eq!(res.status(), 200);
        let body = String::from_utf8(read_body(res).await.to_vec()).unwrap();
        let user_ip = IpAddr::from_str(&body).unwrap();
        assert_eq!(user_ip, expected)
    }

    #[actix_web::test]
    async fn get_user_ip_v4_peer_trusted_forwarded_for() {
        let reverse_proxy_ips = vec![IpNetwork::from(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)))];
        let app = init_service(test_app(reverse_proxy_ips)).await;
        let peer = (IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)), 1234).into();
        let req = TestRequest::default()
            .peer_addr(peer)
            .insert_header((FORWARDED_FOR_HEADER, "10.0.0.1"))
            .to_request();
        let expected = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

        let res = call_service(&app, req).await;
        assert_eq!(res.status(), 200);
        let body = String::from_utf8(read_body(res).await.to_vec()).unwrap();
        let user_ip = IpAddr::from_str(&body).unwrap();
        assert_eq!(user_ip, expected)
    }

    #[actix_web::test]
    async fn get_user_ip_v6_peer_trusted_forwarded_for() {
        let reverse_proxy_ips = vec![IpNetwork::from(IpAddr::V6(Ipv6Addr::new(
            0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff,
        )))];
        let app = init_service(test_app(reverse_proxy_ips)).await;
        let peer = (
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff)),
            1234,
        )
            .into();
        let req = TestRequest::default()
            .peer_addr(peer)
            .insert_header((FORWARDED_FOR_HEADER, "::ffff:192.10.2.1"))
            .to_request();
        let expected = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x201));

        let res = call_service(&app, req).await;
        assert_eq!(res.status(), 200);
        let body = String::from_utf8(read_body(res).await.to_vec()).unwrap();
        let user_ip = IpAddr::from_str(&body).unwrap();
        assert_eq!(user_ip, expected)
    }

    #[actix_web::test]
    async fn get_user_ip_v4_peer_untrusted_forwarded_for() {
        let reverse_proxy_ips = vec![IpNetwork::from(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)))];
        let app = init_service(test_app(reverse_proxy_ips)).await;
        let peer = (IpAddr::V4(Ipv4Addr::new(192, 168, 0, 2)), 1234).into();
        let req = TestRequest::default()
            .peer_addr(peer)
            .insert_header((FORWARDED_FOR_HEADER, "10.0.0.1"))
            .to_request();
        let expected = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 2));

        let res = call_service(&app, req).await;
        assert_eq!(res.status(), 200);
        let body = String::from_utf8(read_body(res).await.to_vec()).unwrap();
        let user_ip = IpAddr::from_str(&body).unwrap();
        assert_eq!(user_ip, expected)
    }

    #[actix_web::test]
    async fn get_user_ip_v6_peer_untrusted_forwarded_for() {
        let reverse_proxy_ips = vec![IpNetwork::from(IpAddr::V6(Ipv6Addr::new(
            0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff,
        )))];
        let app = init_service(test_app(reverse_proxy_ips)).await;
        let peer = (
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2fe)),
            1234,
        )
            .into();
        let req = TestRequest::default()
            .peer_addr(peer)
            .insert_header((FORWARDED_FOR_HEADER, "::ffff:192.10.2.1"))
            .to_request();
        let expected = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2fe));

        let res = call_service(&app, req).await;
        assert_eq!(res.status(), 200);
        let body = String::from_utf8(read_body(res).await.to_vec()).unwrap();
        let user_ip = IpAddr::from_str(&body).unwrap();
        assert_eq!(user_ip, expected)
    }

    #[actix_web::test]
    async fn get_user_ip() {
        let reverse_proxy_ips = vec![
            IpNetwork::from(IpAddr::V6(Ipv6Addr::new(
                0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff,
            ))),
            IpNetwork::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8).unwrap(),
        ];
        let app = init_service(test_app(reverse_proxy_ips)).await;

        let tests = [
            (
                SocketAddr::from((
                    IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff)),
                    1234,
                )),
                "::ffff:192.10.2.1",
                IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x201)),
            ),
            (
                SocketAddr::from((IpAddr::V4(Ipv4Addr::new(127, 0, 1, 200)), 1234)),
                "10.10.10.10",
                IpAddr::V4(Ipv4Addr::new(10, 10, 10, 10)),
            ),
            (
                SocketAddr::from((IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 1234)),
                "10.10.10.10",
                IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            ),
        ];
        for (peer, forwarded_for, expected) in tests {
            let req = TestRequest::default()
                .peer_addr(peer)
                .insert_header((FORWARDED_FOR_HEADER, forwarded_for))
                .to_request();

            let res = call_service(&app, req).await;
            assert_eq!(res.status(), 200);
            let body = String::from_utf8(read_body(res).await.to_vec()).unwrap();
            let user_ip = IpAddr::from_str(&body).unwrap();
            assert_eq!(user_ip, expected)
        }
    }
}
