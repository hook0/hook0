//! Per-request HTTP server metrics: how long the API took to answer each request, on which
//! route and with which status, following the OpenTelemetry HTTP semantic conventions
//! (`http.server.request.duration`). Request counts and error rates come from the
//! histogram's count.
//!
//! The middleware sits just inside path normalization and outside everything else, so it
//! also times answers produced by other middlewares before any handler runs: rate limiting
//! (429), authentication (401/403), CORS preflights, and requests that match no route at
//! all (404).
//!
//! Label cardinality is bounded on purpose, because every distinct label set becomes a
//! series in the metrics backend, multiplied by the number of buckets:
//! - `http.route` is the matched route *template* (`/api/v1/applications/{application_id}`),
//!   never the raw path, which carries identifiers and arbitrary client input; it is left
//!   out when no route matched (including the web app files, served by the default service);
//! - `http.request.method` keeps the standard methods and folds anything else into
//!   `_OTHER`, since a client can send any token as a method;
//! - `http.response.status_code` is the exact code, a few dozen values at most.
//!
//! `url.scheme` and `error.type` are left out: behind the reverse proxy the scheme is either
//! constant or read from headers the client controls, and the error type would only repeat
//! the status code.

use std::sync::LazyLock;
use std::time::Instant;

use actix_web::Error;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::Method;
use actix_web::middleware::Next;
use clap::crate_name;
use opentelemetry::metrics::{Histogram, Meter};
use opentelemetry::{KeyValue, global};

/// Name of the duration histogram, shared with the view that sets its buckets.
pub const HTTP_SERVER_REQUEST_DURATION: &str = "http.server.request.duration";

/// The instrument the middleware records into.
#[derive(Debug, Clone)]
pub struct HttpServerMetrics {
    duration: Histogram<f64>,
}

impl HttpServerMetrics {
    pub fn new(meter: &Meter) -> Self {
        Self {
            duration: meter
                .f64_histogram(HTTP_SERVER_REQUEST_DURATION)
                .with_unit("s")
                .with_description(
                    "Time until the response head is ready, labelled by method, route template and status code",
                )
                .build(),
        }
    }

    /// Run the rest of the chain and record the answer it produced.
    ///
    /// The response is returned untouched. An `Err` coming up from an inner middleware is
    /// recorded with its route and the status actix will turn it into, then passed through
    /// as is.
    pub async fn observe<B: MessageBody>(
        &self,
        req: ServiceRequest,
        next: Next<B>,
    ) -> Result<ServiceResponse<B>, Error> {
        let started = Instant::now();
        let method = method_label(req.method());
        // A lookup of the (already normalized) path in the route table: it needs no routing,
        // so it is known before the call and survives an `Err`, which does not hand the
        // request back.
        let route = req.match_pattern();

        let result = next.call(req).await;

        let status = match &result {
            Ok(res) => res.status(),
            Err(e) => e.as_response_error().status_code(),
        };

        let mut attributes = vec![
            KeyValue::new("http.request.method", method),
            KeyValue::new("http.response.status_code", i64::from(status.as_u16())),
        ];
        if let Some(route) = route {
            attributes.push(KeyValue::new("http.route", route));
        }
        self.duration
            .record(started.elapsed().as_secs_f64(), &attributes);

        result
    }
}

// Built on first request, which always comes after `opentelemetry::init()` has set the
// global meter provider (see the note above the instruments in `opentelemetry.rs`).
static HTTP_SERVER_METRICS: LazyLock<HttpServerMetrics> =
    LazyLock::new(|| HttpServerMetrics::new(&global::meter(crate_name!())));

/// The middleware itself, to be installed with `actix_web::middleware::from_fn`.
pub async fn record_http_server_metrics<B: MessageBody>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error> {
    HTTP_SERVER_METRICS.observe(req, next).await
}

fn method_label(method: &Method) -> &'static str {
    match *method {
        Method::GET => "GET",
        Method::HEAD => "HEAD",
        Method::POST => "POST",
        Method::PUT => "PUT",
        Method::DELETE => "DELETE",
        Method::CONNECT => "CONNECT",
        Method::OPTIONS => "OPTIONS",
        Method::TRACE => "TRACE",
        Method::PATCH => "PATCH",
        _ => "_OTHER",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::body::BoxBody;
    use actix_web::middleware::{NormalizePath, from_fn};
    use actix_web::test as http_test;
    use actix_web::{App, HttpResponse, web};
    use opentelemetry::metrics::MeterProvider;
    use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData};
    use opentelemetry_sdk::metrics::{InMemoryMetricExporter, SdkMeterProvider};
    use std::collections::BTreeMap;

    type Labels = BTreeMap<String, String>;

    // Spelled out rather than shared with the middleware, so a typo there fails here.
    const METHOD: &str = "http.request.method";
    const ROUTE: &str = "http.route";
    const STATUS: &str = "http.response.status_code";

    /// Serve `requests` through an application wrapped like the real one (path
    /// normalization outermost, metrics just inside it), then return every recorded series
    /// with its sample count.
    async fn record(requests: Vec<http_test::TestRequest>) -> Vec<(Labels, u64)> {
        let exporter = InMemoryMetricExporter::default();
        let provider = SdkMeterProvider::builder()
            .with_periodic_exporter(exporter.clone())
            .build();
        let metrics = HttpServerMetrics::new(&provider.meter("test"));

        let app = http_test::init_service(
            App::new()
                .wrap(from_fn(move |req, next| {
                    let metrics = metrics.clone();
                    async move { metrics.observe(req, next).await }
                }))
                .wrap(NormalizePath::trim())
                .service(
                    web::scope("/api/v1")
                        // Registered like the real resources, so a method no route accepts
                        // gets the resource's 405.
                        .service(
                            web::resource("/applications/{application_id}")
                                .route(web::get().to(|| async { HttpResponse::Ok().finish() })),
                        )
                        .route(
                            "/event",
                            web::post().to(|| async { HttpResponse::TooManyRequests().finish() }),
                        )
                        .route(
                            "/broken",
                            web::get().to(|| async {
                                Err::<HttpResponse, _>(actix_web::error::ErrorInternalServerError(
                                    "boom",
                                ))
                            }),
                        ),
                )
                // Stands for the rate limiter refusing an exceeded quota: it answers from
                // the scope, before the request is routed to a resource.
                .service(
                    web::scope("/limited")
                        .wrap(from_fn(|req: ServiceRequest, _next: Next<BoxBody>| async {
                            Ok::<_, actix_web::Error>(
                                req.into_response(HttpResponse::TooManyRequests().finish()),
                            )
                        }))
                        .route(
                            "/{id}",
                            web::get().to(|| async { HttpResponse::Ok().finish() }),
                        ),
                )
                // Stands for the rate limiter when it cannot find the key it limits on (the
                // caller's IP or token): it then raises an `Err` instead of building the
                // response.
                .service(
                    web::scope("/refused")
                        .wrap(from_fn(
                            |_req: ServiceRequest, _next: Next<BoxBody>| async {
                                Err::<ServiceResponse, _>(
                                    actix_web::error::ErrorInternalServerError("no key"),
                                )
                            },
                        ))
                        .route(
                            "/{id}",
                            web::get().to(|| async { HttpResponse::Ok().finish() }),
                        ),
                ),
        )
        .await;
        for req in requests {
            // An `Err` is what the middleware must still record, not a test failure.
            let _ = http_test::try_call_service(&app, req.to_request()).await;
        }

        provider.force_flush().expect("flush metrics");
        let mut recorded = Vec::new();
        for resource_metrics in exporter.get_finished_metrics().expect("read metrics") {
            for scope in resource_metrics.scope_metrics() {
                for metric in scope.metrics() {
                    if let AggregatedMetrics::F64(MetricData::Histogram(h)) = metric.data() {
                        for point in h.data_points() {
                            recorded.push((labels(point.attributes()), point.count()));
                        }
                    }
                }
            }
        }
        recorded.sort();
        recorded
    }

    fn labels<'a>(attributes: impl Iterator<Item = &'a KeyValue>) -> Labels {
        attributes
            .map(|kv| (kv.key.to_string(), kv.value.to_string()))
            .collect()
    }

    fn series(pairs: &[(&str, &str)]) -> Labels {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    #[actix_web::test]
    async fn requests_on_one_route_share_its_template_whatever_the_identifier() {
        let recorded = record(vec![
            http_test::TestRequest::get().uri("/api/v1/applications/0d3c6c4e"),
            http_test::TestRequest::get().uri("/api/v1/applications/7a1f9e2b"),
            // A trailing slash is normalized away before routing, so it lands on the same series.
            http_test::TestRequest::get().uri("/api/v1/applications/7a1f9e2b/"),
        ])
        .await;

        assert_eq!(
            recorded,
            vec![(
                series(&[
                    (METHOD, "GET"),
                    (ROUTE, "/api/v1/applications/{application_id}"),
                    (STATUS, "200")
                ]),
                3
            )]
        );
    }

    #[actix_web::test]
    async fn client_and_server_errors_are_counted_with_their_status() {
        let recorded = record(vec![
            http_test::TestRequest::post().uri("/api/v1/event"),
            http_test::TestRequest::get().uri("/api/v1/broken"),
        ])
        .await;

        assert_eq!(
            recorded,
            vec![
                (
                    series(&[(METHOD, "GET"), (ROUTE, "/api/v1/broken"), (STATUS, "500")]),
                    1
                ),
                (
                    series(&[(METHOD, "POST"), (ROUTE, "/api/v1/event"), (STATUS, "429")]),
                    1
                ),
            ]
        );
    }

    #[actix_web::test]
    async fn unknown_paths_and_methods_cannot_grow_the_label_set() {
        let recorded = record(vec![
            http_test::TestRequest::get().uri("/wp-admin/install.php"),
            http_test::TestRequest::get().uri("/api/v1/nope/123"),
            http_test::TestRequest::default()
                .method(Method::from_bytes(b"PROPFIND").unwrap())
                .uri("/api/v1/applications/1"),
        ])
        .await;

        assert_eq!(
            recorded,
            vec![
                // Paths no route matches carry no route at all, whatever they were.
                (series(&[(METHOD, "GET"), (STATUS, "404")]), 2),
                // A path that matches with a method no route accepts is a 405, and the
                // method is folded rather than kept as sent.
                (
                    series(&[
                        (METHOD, "_OTHER"),
                        (ROUTE, "/api/v1/applications/{application_id}"),
                        (STATUS, "405")
                    ]),
                    1
                ),
            ]
        );
    }

    #[actix_web::test]
    async fn answers_given_before_routing_still_carry_the_route_template() {
        let recorded = record(vec![http_test::TestRequest::get().uri("/limited/1")]).await;

        assert_eq!(
            recorded,
            vec![(
                series(&[(METHOD, "GET"), (ROUTE, "/limited/{id}"), (STATUS, "429")]),
                1
            )]
        );
    }

    #[actix_web::test]
    async fn errors_raised_by_an_inner_middleware_keep_their_route_and_the_status_they_become() {
        let recorded = record(vec![http_test::TestRequest::get().uri("/refused/1")]).await;

        // The route is read before the call, so losing the request to the `Err` loses nothing.
        assert_eq!(
            recorded,
            vec![(
                series(&[(METHOD, "GET"), (ROUTE, "/refused/{id}"), (STATUS, "500")]),
                1
            )]
        );
    }
}
