use actix_web::body::{EitherBody, MessageBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::{FORWARDED, HeaderMap, HeaderValue, X_FORWARDED_FOR};
use actix_web::{Error, HttpMessage, HttpResponse};
use futures_util::future::{Ready, ok, ready};
use ipnetwork::IpNetwork;
use std::future::Future;
use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::ops::ControlFlow;
use std::pin::Pin;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::LazyLock;
use std::task::{Context, Poll};
use tracing::{debug, error, trace};

const CLOUDFLARE_IP_HEADER: &str = "CF-Connecting-IP";

// Manually exported from https://www.cloudflare.com/ips/
const CLOUDFLARE_CIDRS: &[&str] = &[
    "173.245.48.0/20",
    "103.21.244.0/22",
    "103.22.200.0/22",
    "103.31.4.0/22",
    "141.101.64.0/18",
    "108.162.192.0/18",
    "190.93.240.0/20",
    "188.114.96.0/20",
    "197.234.240.0/22",
    "198.41.128.0/17",
    "162.158.0.0/15",
    "104.16.0.0/13",
    "104.24.0.0/14",
    "172.64.0.0/13",
    "131.0.72.0/22",
    "2400:cb00::/32",
    "2606:4700::/32",
    "2803:f800::/32",
    "2405:b500::/32",
    "2405:8100::/32",
    "2a06:98c0::/29",
    "2c0f:f248::/32",
];

/// [`CLOUDFLARE_CIDRS`], parsed once for the life of the process.
static CLOUDFLARE_NETWORKS: LazyLock<Vec<IpNetwork>> = LazyLock::new(|| {
    CLOUDFLARE_CIDRS
        .iter()
        .map(|cidr| IpNetwork::from_str(cidr).expect("CLOUDFLARE_CIDRS only holds valid CIDRs"))
        .collect()
});

#[derive(Debug, Clone)]
pub struct GetUserIp {
    pub reverse_proxy_cidrs: Vec<IpNetwork>,
    pub behind_cloudflare: bool,
}

impl<S, B> Transform<S, ServiceRequest> for GetUserIp
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = GetUserIpMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        trace!("Initialize GetUserIpMiddleware");
        ok(GetUserIpMiddleware {
            service: Rc::new(service),
            reverse_proxy_cidrs: self.reverse_proxy_cidrs.to_owned(),
            behind_cloudflare: self.behind_cloudflare,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GetUserIpMiddleware<S> {
    service: Rc<S>,
    reverse_proxy_cidrs: Vec<IpNetwork>,
    behind_cloudflare: bool,
}

impl<S, B> Service<ServiceRequest> for GetUserIpMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        match extract_ip(&req, &self.reverse_proxy_cidrs, self.behind_cloudflare) {
            Ok(ip) => {
                debug!("User IP is {}", &ip);
                req.extensions_mut().insert(ip);
                replace_forwarding_headers(req.headers_mut(), ip);
                let response = self.service.call(req);
                Box::pin(async move { response.await.map(ServiceResponse::map_into_left_body) })
            }
            Err(err) => {
                let e = format!("GetUserIpMiddleware cannot find the user IP: {}", err);
                error!("{}", &e);
                Box::pin(ready(Ok(req
                    .into_response(HttpResponse::InternalServerError().body(e))
                    .map_into_right_body())))
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

/// Resolve the address of the client a request comes from.
///
/// The direct peer is the client unless it is one of the trusted reverse proxies. Then the
/// `X-Forwarded-For` chain is walked from its right end: every entry there was written by the hop
/// to its right, so an entry is only as believable as that hop, and the walk goes on for as long as
/// it meets trusted proxies. The first address that is not one is the client. Whatever stands to
/// its left was written by the client itself, which is why the leftmost entry (the one
/// `ConnectionInfo::realip_remote_addr` reads) is never taken on trust.
///
/// A chain made only of trusted proxies resolves to its leftmost entry, and one an unreadable
/// entry cuts short resolves to the hop that wrote that entry: both are addresses a trusted hop
/// vouched for. `Forwarded` is not read at all. A proxy maintains one of the two headers, and
/// reading both would let a client fill in whichever one its proxy leaves alone.
///
/// Behind Cloudflare, reaching one of its addresses ends the walk too, and the client is the one
/// Cloudflare names in a header of its own.
///
/// An IPv4-mapped address is matched against the IPv4 networks of both lists as well, so the
/// proxies and Cloudflare are still recognised when a dual-stack listener reports them that way.
fn extract_ip(
    req: &ServiceRequest,
    reverse_proxy_cidrs: &[IpNetwork],
    behind_cloudflare: bool,
) -> Result<IpAddr, GetUserIpError> {
    let peer_ip = req.peer_addr().ok_or(GetUserIpError::NoIpInRequest)?.ip();

    let is_cloudflare = |ip: IpAddr| behind_cloudflare && is_within(&CLOUDFLARE_NETWORKS, ip);
    let is_trusted_proxy = |ip: IpAddr| is_within(reverse_proxy_cidrs, ip);

    let walk = std::iter::once(peer_ip)
        .chain(forwarded_for_hops(req.headers()))
        .try_fold(peer_ip, |_, hop| {
            if is_cloudflare(hop) || !is_trusted_proxy(hop) {
                ControlFlow::Break(hop)
            } else {
                ControlFlow::Continue(hop)
            }
        });
    let client_hop = match walk {
        ControlFlow::Break(hop) | ControlFlow::Continue(hop) => hop,
    };

    let ip = if is_cloudflare(client_hop) {
        cloudflare_connecting_ip(req).unwrap_or(client_hop)
    } else {
        client_hop
    };
    Ok(ip)
}

/// The addresses of the `X-Forwarded-For` chain, nearest hop first.
///
/// Several lines of the header form one chain, in order, so the lines are read from the last one
/// and each line from its end. Entries are split on raw bytes rather than read line by line as
/// text: a line a client filled with bytes that are not text would otherwise be dropped whole,
/// taking with it the entry a proxy appended after them, and leaving the client's own entries
/// as the nearest ones.
///
/// Reading stops at the first entry that is not an address, since nothing to the left of it can
/// be attributed to a hop the walk could still trust.
fn forwarded_for_hops(headers: &HeaderMap) -> impl Iterator<Item = IpAddr> + '_ {
    headers
        .get_all(X_FORWARDED_FOR)
        .rev()
        .flat_map(|line| line.as_bytes().rsplit(|byte| *byte == b','))
        .map_while(|entry| {
            std::str::from_utf8(entry)
                .ok()
                .and_then(|entry| parse_ip(entry.trim()).ok())
        })
}

/// Whether `ip` falls in one of `networks`, in either of the forms a dual-stack listener can
/// report an IPv4 caller under.
///
/// A listener bound on `::` sees an IPv4 peer as `::ffff:a.b.c.d`, which an IPv4 network never
/// contains, so the IPv4 address it carries is tried too. The address is also tried as given, so
/// a list that already spells such peers as IPv4-mapped networks keeps matching them.
fn is_within(networks: &[IpNetwork], ip: IpAddr) -> bool {
    let canonical = ip.to_canonical();
    networks
        .iter()
        .any(|network| network.contains(ip) || network.contains(canonical))
}

/// The client address Cloudflare names, when the request carries one that can be read.
fn cloudflare_connecting_ip(req: &ServiceRequest) -> Option<IpAddr> {
    req.headers()
        .get(CLOUDFLARE_IP_HEADER)
        .and_then(|hv| {
            hv.to_str()
                .inspect_err(|e| trace!("Could not read {CLOUDFLARE_IP_HEADER} header: {e}"))
                .ok()
        })
        .and_then(|str| IpAddr::from_str(str).inspect_err(|e| trace!("Could not parse an IP address from {CLOUDFLARE_IP_HEADER} header (value='{str}'): {e}")).ok())
}

/// Leave the resolved address as the only one the request still announces.
///
/// `ConnectionInfo::realip_remote_addr` reads `Forwarded`, then the leftmost `X-Forwarded-For`
/// entry, with no notion of which peers are trusted, and the request span records it as
/// `http.client_ip`. Without this, the logs would repeat whatever address a client chose to
/// announce, whatever the rate limiters were told. `ConnectionInfo` is computed on its first read
/// and kept with the request, so this only reaches readers that run after this middleware.
fn replace_forwarding_headers(headers: &mut HeaderMap, ip: IpAddr) {
    headers.remove(FORWARDED);
    if let Ok(value) = HeaderValue::from_str(&ip.to_string()) {
        headers.insert(X_FORWARDED_FOR, value);
    } else {
        headers.remove(X_FORWARDED_FOR);
    }
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

    use actix_web::body::MessageBody;
    use actix_web::dev::ServiceFactory;
    use actix_web::test::{TestRequest, call_service, init_service, read_body};
    use actix_web::web::get;
    use actix_web::{App, HttpRequest};
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
        reverse_proxy_cidrs: Vec<IpNetwork>,
        behind_cloudflare: bool,
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
            .wrap(GetUserIp {
                reverse_proxy_cidrs,
                behind_cloudflare,
            })
            .route("/", get().to(return_user_ip))
            .route("/realip", get().to(return_realip_remote_addr))
    }

    async fn return_user_ip(UserIp(ip): UserIp) -> String {
        ip.to_string()
    }

    /// What a reader of `ConnectionInfo` that runs after the middleware, like the request span,
    /// takes the client address to be.
    async fn return_realip_remote_addr(req: HttpRequest) -> String {
        req.connection_info()
            .realip_remote_addr()
            .unwrap_or_default()
            .to_owned()
    }

    const FORWARDED_FOR_HEADER: &str = "X-Forwarded-For";

    /// The only trusted proxy of most cases below, and the peer they are sent from.
    const PROXY: Ipv4Addr = Ipv4Addr::new(192, 168, 0, 1);

    fn proxy_peer() -> SocketAddr {
        SocketAddr::from((PROXY, 1234))
    }

    fn only_the_proxy() -> Vec<IpNetwork> {
        vec![IpNetwork::from(IpAddr::V4(PROXY))]
    }

    /// Every address of 192.168.0.0/24 is a trusted proxy.
    fn a_chain_of_proxies() -> Vec<IpNetwork> {
        vec![IpNetwork::new(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0)), 24).unwrap()]
    }

    fn ip(str: &str) -> IpAddr {
        IpAddr::from_str(str).unwrap()
    }

    /// A peer at `address`, on an arbitrary port.
    fn peer_at(address: &str) -> SocketAddr {
        SocketAddr::from((ip(address), 1234))
    }

    /// The address the middleware resolves for `request`, which must be served.
    async fn user_ip_for(
        reverse_proxy_cidrs: Vec<IpNetwork>,
        behind_cloudflare: bool,
        request: TestRequest,
    ) -> IpAddr {
        let app = init_service(test_app(reverse_proxy_cidrs, behind_cloudflare)).await;
        let res = call_service(&app, request.uri("/").to_request()).await;
        assert_eq!(res.status(), 200);
        let body = String::from_utf8(read_body(res).await.to_vec()).unwrap();
        IpAddr::from_str(&body).unwrap()
    }

    #[actix_web::test]
    async fn get_user_ip_v4_peer_no_trusted_proxies() {
        let app = init_service(test_app(Vec::new(), false)).await;
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
        let app = init_service(test_app(Vec::new(), false)).await;
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
        let reverse_proxy_cidrs = vec![IpNetwork::from(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)))];
        let app = init_service(test_app(reverse_proxy_cidrs, false)).await;
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
        let reverse_proxy_cidrs = vec![IpNetwork::from(IpAddr::V6(Ipv6Addr::new(
            0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff,
        )))];
        let app = init_service(test_app(reverse_proxy_cidrs, false)).await;
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
        let reverse_proxy_cidrs = vec![IpNetwork::from(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)))];
        let app = init_service(test_app(reverse_proxy_cidrs, false)).await;
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
        let reverse_proxy_cidrs = vec![IpNetwork::from(IpAddr::V6(Ipv6Addr::new(
            0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff,
        )))];
        let app = init_service(test_app(reverse_proxy_cidrs, false)).await;
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
    async fn get_user_ip_no_cf() {
        let reverse_proxy_cidrs = vec![
            IpNetwork::from(IpAddr::V6(Ipv6Addr::new(
                0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff,
            ))),
            IpNetwork::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8).unwrap(),
        ];
        let app = init_service(test_app(reverse_proxy_cidrs, false)).await;

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

    #[actix_web::test]
    async fn get_user_ip_with_cf() {
        let reverse_proxy_cidrs = vec![
            IpNetwork::from(IpAddr::V6(Ipv6Addr::new(
                0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff,
            ))),
            IpNetwork::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8).unwrap(),
        ];
        let app = init_service(test_app(reverse_proxy_cidrs, true)).await;

        let tests = [
            (
                SocketAddr::from((
                    IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff)),
                    1234,
                )),
                Some("::ffff:192.10.2.1"),
                None,
                IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x201)),
            ),
            (
                SocketAddr::from((IpAddr::V4(Ipv4Addr::new(127, 0, 1, 200)), 1234)),
                Some("10.10.10.10"),
                None,
                IpAddr::V4(Ipv4Addr::new(10, 10, 10, 10)),
            ),
            (
                SocketAddr::from((IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 1234)),
                Some("10.10.10.10"),
                None,
                IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            ),
            //
            (
                SocketAddr::from((IpAddr::V4(Ipv4Addr::new(173, 245, 48, 1)), 1234)),
                None,
                Some("10.0.0.1"),
                IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            ),
            (
                SocketAddr::from((IpAddr::V4(Ipv4Addr::new(127, 2, 3, 4)), 1234)),
                Some("173.245.48.1"),
                Some("10.0.0.1"),
                IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            ),
            (
                SocketAddr::from((IpAddr::V4(Ipv4Addr::new(10, 2, 3, 4)), 1234)),
                None,
                Some("10.0.0.1"),
                IpAddr::V4(Ipv4Addr::new(10, 2, 3, 4)),
            ),
            (
                SocketAddr::from((IpAddr::V4(Ipv4Addr::new(127, 2, 3, 4)), 1234)),
                Some("10.2.3.4"),
                Some("10.0.0.1"),
                IpAddr::V4(Ipv4Addr::new(10, 2, 3, 4)),
            ),
        ];
        for (peer, forwarded_for, cf_header, expected) in tests {
            let mut req = TestRequest::default().peer_addr(peer);
            if let Some(ff) = forwarded_for {
                req = req.insert_header((FORWARDED_FOR_HEADER, ff));
            }
            if let Some(cfh) = cf_header {
                req = req.insert_header((CLOUDFLARE_IP_HEADER, cfh));
            }
            let req = req.to_request();
            dbg!(&req.headers());

            let res = call_service(&app, req).await;
            assert_eq!(res.status(), 200);
            let body = String::from_utf8(read_body(res).await.to_vec()).unwrap();
            let user_ip = IpAddr::from_str(&body).unwrap();
            assert_eq!(user_ip, expected)
        }
    }

    /// An appending proxy keeps whatever the client wrote and adds its peer at the end, so the
    /// client's own entries are on the left, and only the rightmost one is the proxy's.
    #[actix_web::test]
    async fn an_entry_written_by_the_client_is_not_believed() {
        let request = TestRequest::default()
            .peer_addr(proxy_peer())
            .insert_header((FORWARDED_FOR_HEADER, "203.0.113.7, 198.51.100.1"));
        assert_eq!(
            user_ip_for(only_the_proxy(), false, request).await,
            ip("198.51.100.1")
        );
    }

    /// Some proxies add a line of their own rather than extending the client's, and Sozu extends
    /// only the last of several client lines.
    #[actix_web::test]
    async fn separate_header_lines_form_one_chain() {
        let request = TestRequest::default()
            .peer_addr(proxy_peer())
            .append_header((FORWARDED_FOR_HEADER, "203.0.113.7"))
            .append_header((FORWARDED_FOR_HEADER, "203.0.113.8, 198.51.100.1"));
        assert_eq!(
            user_ip_for(only_the_proxy(), false, request).await,
            ip("198.51.100.1")
        );
    }

    #[actix_web::test]
    async fn a_chain_of_trusted_proxies_is_walked_to_the_client() {
        let request = TestRequest::default()
            .peer_addr(proxy_peer())
            .insert_header((
                FORWARDED_FOR_HEADER,
                "203.0.113.7, 198.51.100.1, 192.168.0.3, 192.168.0.2",
            ));
        assert_eq!(
            user_ip_for(a_chain_of_proxies(), false, request).await,
            ip("198.51.100.1")
        );
    }

    /// Only a trusted hop can have written the leftmost entry of such a chain, because the client
    /// it came from would otherwise have been appended after it.
    #[actix_web::test]
    async fn a_chain_made_only_of_trusted_proxies_resolves_to_its_leftmost_entry() {
        let request = TestRequest::default()
            .peer_addr(proxy_peer())
            .insert_header((FORWARDED_FOR_HEADER, "192.168.0.3, 192.168.0.2"));
        assert_eq!(
            user_ip_for(a_chain_of_proxies(), false, request).await,
            ip("192.168.0.3")
        );
    }

    #[actix_web::test]
    async fn the_forwarded_header_is_not_read() {
        let alone = TestRequest::default()
            .peer_addr(proxy_peer())
            .insert_header(("Forwarded", "for=203.0.113.7"));
        assert_eq!(
            user_ip_for(only_the_proxy(), false, alone).await,
            IpAddr::V4(PROXY)
        );

        let beside_forwarded_for = TestRequest::default()
            .peer_addr(proxy_peer())
            .insert_header(("Forwarded", "for=203.0.113.7"))
            .insert_header((FORWARDED_FOR_HEADER, "198.51.100.1"));
        assert_eq!(
            user_ip_for(only_the_proxy(), false, beside_forwarded_for).await,
            ip("198.51.100.1")
        );
    }

    /// Nothing to the left of an entry that is not an address can be tied to a hop the walk
    /// trusts, so the client is taken to be the hop that wrote it, and the request is still
    /// served.
    #[actix_web::test]
    async fn an_unreadable_entry_ends_the_walk_at_the_hop_that_wrote_it() {
        let next_to_the_peer = TestRequest::default()
            .peer_addr(proxy_peer())
            .insert_header((FORWARDED_FOR_HEADER, "198.51.100.1, unknown"));
        assert_eq!(
            user_ip_for(only_the_proxy(), false, next_to_the_peer).await,
            IpAddr::V4(PROXY)
        );

        let further_out = TestRequest::default()
            .peer_addr(proxy_peer())
            .insert_header((FORWARDED_FOR_HEADER, "198.51.100.1, unknown, 192.168.0.2"));
        assert_eq!(
            user_ip_for(a_chain_of_proxies(), false, further_out).await,
            ip("192.168.0.2")
        );

        let not_text = TestRequest::default()
            .peer_addr(proxy_peer())
            .insert_header((
                FORWARDED_FOR_HEADER,
                HeaderValue::from_bytes(b"198.51.100.1, \xff").unwrap(),
            ));
        assert_eq!(
            user_ip_for(only_the_proxy(), false, not_text).await,
            IpAddr::V4(PROXY)
        );
    }

    /// A line that is not text as a whole still holds the entry the proxy appended after the
    /// client's bytes, and that entry is the one that counts.
    #[actix_web::test]
    async fn bytes_that_are_not_text_do_not_hide_the_entry_a_proxy_appended() {
        let request = TestRequest::default()
            .peer_addr(proxy_peer())
            .insert_header((
                FORWARDED_FOR_HEADER,
                HeaderValue::from_bytes(b"\xff203.0.113.7, 198.51.100.1").unwrap(),
            ));
        assert_eq!(
            user_ip_for(only_the_proxy(), false, request).await,
            ip("198.51.100.1")
        );
    }

    #[actix_web::test]
    async fn an_entry_may_carry_a_port() {
        for (entry, expected) in [
            ("198.51.100.1:4711", ip("198.51.100.1")),
            ("[2001:db8::1]:4711", ip("2001:db8::1")),
        ] {
            let request = TestRequest::default()
                .peer_addr(proxy_peer())
                .insert_header((FORWARDED_FOR_HEADER, entry));
            assert_eq!(
                user_ip_for(only_the_proxy(), false, request).await,
                expected,
                "{entry}"
            );
        }
    }

    /// Client, Cloudflare, then a load balancer of our own: the walk stops at the Cloudflare
    /// address the load balancer appended, and Cloudflare's own header names the client.
    #[actix_web::test]
    async fn cloudflare_behind_a_trusted_proxy_names_the_client() {
        let request = TestRequest::default()
            .peer_addr(proxy_peer())
            .insert_header((
                FORWARDED_FOR_HEADER,
                "203.0.113.7, 198.51.100.9, 173.245.48.1",
            ))
            .insert_header((CLOUDFLARE_IP_HEADER, "198.51.100.9"));
        assert_eq!(
            user_ip_for(only_the_proxy(), true, request).await,
            ip("198.51.100.9")
        );

        let without_its_header = TestRequest::default()
            .peer_addr(proxy_peer())
            .insert_header((FORWARDED_FOR_HEADER, "198.51.100.9, 173.245.48.1"));
        assert_eq!(
            user_ip_for(only_the_proxy(), true, without_its_header).await,
            ip("173.245.48.1")
        );
    }

    /// A client that skips Cloudflare and writes a Cloudflare address itself does not get its
    /// `CF-Connecting-IP` read: the load balancer appends the client's real address after it.
    #[actix_web::test]
    async fn a_cloudflare_address_written_by_the_client_is_not_believed() {
        let request = TestRequest::default()
            .peer_addr(proxy_peer())
            .insert_header((FORWARDED_FOR_HEADER, "173.245.48.1, 198.51.100.66"))
            .insert_header((CLOUDFLARE_IP_HEADER, "203.0.113.7"));
        assert_eq!(
            user_ip_for(only_the_proxy(), true, request).await,
            ip("198.51.100.66")
        );
    }

    /// A listener bound on `::` reports an IPv4 peer as an IPv4-mapped address, and the proxy it
    /// stands for is listed under its IPv4 address.
    #[actix_web::test]
    async fn a_trusted_proxy_reported_as_ipv4_mapped_is_recognised() {
        let request = TestRequest::default()
            .peer_addr(peer_at("::ffff:192.168.0.1"))
            .insert_header((FORWARDED_FOR_HEADER, "198.51.100.1"));
        assert_eq!(
            user_ip_for(only_the_proxy(), false, request).await,
            ip("198.51.100.1")
        );
    }

    /// A proxy on a dual-stack listener of its own writes its peer in the mapped form too.
    #[actix_web::test]
    async fn an_ipv4_mapped_hop_in_the_chain_is_recognised() {
        let request = TestRequest::default()
            .peer_addr(proxy_peer())
            .insert_header((FORWARDED_FOR_HEADER, "198.51.100.1, ::ffff:192.168.0.2"));
        assert_eq!(
            user_ip_for(a_chain_of_proxies(), false, request).await,
            ip("198.51.100.1")
        );
    }

    #[actix_web::test]
    async fn a_cloudflare_peer_reported_as_ipv4_mapped_is_recognised() {
        let request = TestRequest::default()
            .peer_addr(peer_at("::ffff:173.245.48.1"))
            .insert_header((CLOUDFLARE_IP_HEADER, "203.0.113.7"));
        assert_eq!(
            user_ip_for(Vec::new(), true, request).await,
            ip("203.0.113.7")
        );
    }

    /// Reading the IPv4 address a mapped one carries must not widen the list.
    #[actix_web::test]
    async fn an_ipv4_mapped_peer_outside_the_list_is_still_the_client() {
        let request = TestRequest::default()
            .peer_addr(peer_at("::ffff:192.168.0.2"))
            .insert_header((FORWARDED_FOR_HEADER, "198.51.100.1"));
        assert_eq!(
            user_ip_for(only_the_proxy(), false, request).await,
            ip("::ffff:192.168.0.2")
        );
    }

    /// The request span reads `http.client_ip` from `ConnectionInfo`, after this middleware has
    /// run. Whatever the client wrote in `Forwarded` or at the left of `X-Forwarded-For` must not
    /// reach it.
    #[actix_web::test]
    async fn later_readers_of_the_request_see_the_resolved_address() {
        let app = init_service(test_app(only_the_proxy(), false)).await;

        for (peer, forwarded_for, expected) in [
            (
                SocketAddr::from(([10, 0, 0, 1], 1234)),
                "203.0.113.7",
                "10.0.0.1",
            ),
            (proxy_peer(), "203.0.113.7, 198.51.100.1", "198.51.100.1"),
        ] {
            let req = TestRequest::default()
                .uri("/realip")
                .peer_addr(peer)
                .insert_header(("Forwarded", "for=9.9.9.9"))
                .insert_header((FORWARDED_FOR_HEADER, forwarded_for))
                .to_request();

            let res = call_service(&app, req).await;
            assert_eq!(res.status(), 200);
            let body = String::from_utf8(read_body(res).await.to_vec()).unwrap();
            assert_eq!(
                body, expected,
                "peer {peer}, X-Forwarded-For {forwarded_for}"
            );
        }
    }

    #[test]
    fn check_internal_cloudflare_cidrs() {
        assert_eq!(CLOUDFLARE_NETWORKS.len(), CLOUDFLARE_CIDRS.len())
    }
}
