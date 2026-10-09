//! Per-request HTTP server metrics: how many requests the API answered, with which
//! status, on which route, and how long each took.
//!
//! The middleware wraps the whole application (it is the outermost layer), so it also
//! counts answers produced by other middlewares before any handler runs: rate limiting
//! (429), authentication (401/403), CORS preflights, path normalization, and requests
//! that match no route at all (404).
//!
//! Label cardinality is bounded on purpose, because every distinct label set becomes a
//! series in the metrics backend:
//! - `route` is the matched route *template* (`/api/v1/applications/{application_id}`),
//!   never the raw path, which carries identifiers and arbitrary client input;
//! - `method` keeps the standard methods and folds anything else into `_OTHER`, since a
//!   client can send any token as a method;
//! - `status` is the exact code on the counter (a few dozen values at most), and only its
//!   class (`2xx`…`5xx`) on the duration histogram, which already multiplies every series
//!   by its number of buckets.

use std::sync::LazyLock;
use std::time::Instant;

use actix_web::Error;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::{Method, StatusCode};
use actix_web::middleware::Next;
use clap::crate_name;
use opentelemetry::metrics::{Counter, Histogram, Meter};
use opentelemetry::{KeyValue, global};

/// Name of the duration histogram, shared with the view that sets its buckets.
pub const HTTP_SERVER_REQUEST_DURATION: &str = "http.server.request.duration";

/// Label of a request that matched no route (including the web app files, served by the
/// default service), or whose route could not be read back.
const UNMATCHED_ROUTE: &str = "unmatched";

/// The two instruments the middleware records into.
#[derive(Debug, Clone)]
pub struct HttpServerMetrics {
    requests: Counter<u64>,
    duration: Histogram<f64>,
}

impl HttpServerMetrics {
    pub fn new(meter: &Meter) -> Self {
        Self {
            requests: meter
                .u64_counter("http.server.requests")
                .with_description(
                    "Count of HTTP responses, labelled by method, route template and status code",
                )
                .build(),
            duration: meter
                .f64_histogram(HTTP_SERVER_REQUEST_DURATION)
                .with_unit("s")
                .with_description(
                    "Time until the response head is ready, labelled by method, route template and status class",
                )
                .build(),
        }
    }

    /// Run the rest of the chain and record the answer it produced.
    ///
    /// The response is returned untouched. An `Err` coming up from an inner middleware is
    /// counted with the status actix will turn it into, then passed through as is.
    pub async fn observe<B: MessageBody>(
        &self,
        req: ServiceRequest,
        next: Next<B>,
    ) -> Result<ServiceResponse<B>, Error> {
        let started = Instant::now();
        let method = method_label(req.method());

        let result = next.call(req).await;

        // Routing happens inside the chain, so the route template is only known on the
        // way out, from the request the response carries. A middleware that answered
        // before the request reached its resource (a rate limiter on a scope) still gets
        // the template: `match_pattern` then matches the path against the route table.
        // On the `Err` path that request
        // is gone: the request cannot be cloned beforehand to keep it, because inner
        // middlewares (path normalization) need to be its only owner to rewrite it.
        let (route, status) = match &result {
            Ok(res) => (
                res.request()
                    .match_pattern()
                    .unwrap_or_else(|| UNMATCHED_ROUTE.to_owned()),
                res.status(),
            ),
            Err(e) => (
                UNMATCHED_ROUTE.to_owned(),
                e.as_response_error().status_code(),
            ),
        };

        self.requests.add(
            1,
            &[
                KeyValue::new("method", method),
                KeyValue::new("route", route.clone()),
                KeyValue::new("status", i64::from(status.as_u16())),
            ],
        );
        self.duration.record(
            started.elapsed().as_secs_f64(),
            &[
                KeyValue::new("method", method),
                KeyValue::new("route", route),
                KeyValue::new("status_class", status_class(status)),
            ],
        );

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

fn status_class(status: StatusCode) -> &'static str {
    match status.as_u16() {
        100..=199 => "1xx",
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        _ => "5xx",
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

    /// Serve `requests` through an application wrapped like the real one (metrics
    /// outermost, path normalization inside), then return every recorded series:
    /// counter values, and histogram sample counts.
    async fn record(
        requests: Vec<http_test::TestRequest>,
    ) -> (Vec<(Labels, u64)>, Vec<(Labels, u64)>) {
        let exporter = InMemoryMetricExporter::default();
        let provider = SdkMeterProvider::builder()
            .with_periodic_exporter(exporter.clone())
            .build();
        let metrics = HttpServerMetrics::new(&provider.meter("test"));

        let app = http_test::init_service(
            App::new()
                .wrap(NormalizePath::trim())
                .wrap(from_fn(move |req, next| {
                    let metrics = metrics.clone();
                    async move { metrics.observe(req, next).await }
                }))
                .service(
                    web::scope("/api/v1")
                        .route(
                            "/applications/{application_id}",
                            web::get().to(|| async { HttpResponse::Ok().finish() }),
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
                // Stands for the rate limiter when it cannot tell who is calling: it then
                // refuses the request with an `Err` instead of building the response.
                .service(
                    web::scope("/refused")
                        .wrap(from_fn(
                            |_req: ServiceRequest, _next: Next<BoxBody>| async {
                                Err::<ServiceResponse, _>(actix_web::error::ErrorUnauthorized(
                                    "refused",
                                ))
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
            // An `Err` is what the middleware must still count, not a test failure.
            let _ = http_test::try_call_service(&app, req.to_request()).await;
        }

        provider.force_flush().expect("flush metrics");
        let mut counter = Vec::new();
        let mut histogram = Vec::new();
        for resource_metrics in exporter.get_finished_metrics().expect("read metrics") {
            for scope in resource_metrics.scope_metrics() {
                for metric in scope.metrics() {
                    match metric.data() {
                        AggregatedMetrics::U64(MetricData::Sum(sum)) => {
                            for point in sum.data_points() {
                                counter.push((labels(point.attributes()), point.value()));
                            }
                        }
                        AggregatedMetrics::F64(MetricData::Histogram(h)) => {
                            for point in h.data_points() {
                                histogram.push((labels(point.attributes()), point.count()));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        counter.sort();
        histogram.sort();
        (counter, histogram)
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
        let (counter, histogram) = record(vec![
            http_test::TestRequest::get().uri("/api/v1/applications/0d3c6c4e"),
            http_test::TestRequest::get().uri("/api/v1/applications/7a1f9e2b"),
            // A trailing slash is normalized away before routing, so it lands on the same series.
            http_test::TestRequest::get().uri("/api/v1/applications/7a1f9e2b/"),
        ])
        .await;

        let route = ("route", "/api/v1/applications/{application_id}");
        assert_eq!(
            counter,
            vec![(series(&[("method", "GET"), route, ("status", "200")]), 3)]
        );
        assert_eq!(
            histogram,
            vec![(
                series(&[("method", "GET"), route, ("status_class", "2xx")]),
                3
            )]
        );
    }

    #[actix_web::test]
    async fn client_and_server_errors_are_counted_with_their_status() {
        let (counter, _) = record(vec![
            http_test::TestRequest::post().uri("/api/v1/event"),
            http_test::TestRequest::get().uri("/api/v1/broken"),
        ])
        .await;

        assert_eq!(
            counter,
            vec![
                (
                    series(&[
                        ("method", "GET"),
                        ("route", "/api/v1/broken"),
                        ("status", "500")
                    ]),
                    1
                ),
                (
                    series(&[
                        ("method", "POST"),
                        ("route", "/api/v1/event"),
                        ("status", "429")
                    ]),
                    1
                ),
            ]
        );
    }

    #[actix_web::test]
    async fn unknown_paths_and_methods_cannot_grow_the_label_set() {
        let (counter, _) = record(vec![
            http_test::TestRequest::get().uri("/wp-admin/install.php"),
            http_test::TestRequest::get().uri("/api/v1/nope/123"),
            http_test::TestRequest::default()
                .method(Method::from_bytes(b"PROPFIND").unwrap())
                .uri("/api/v1/applications/1"),
        ])
        .await;

        assert_eq!(
            counter,
            vec![
                (
                    series(&[
                        ("method", "GET"),
                        ("route", UNMATCHED_ROUTE),
                        ("status", "404")
                    ]),
                    2
                ),
                // A path that matches with a method no route accepts is a 404 in actix,
                // and the method is folded rather than kept as sent.
                (
                    series(&[
                        ("method", "_OTHER"),
                        ("route", "/api/v1/applications/{application_id}"),
                        ("status", "404")
                    ]),
                    1
                ),
            ]
        );
    }

    #[actix_web::test]
    async fn answers_given_before_routing_still_carry_the_route_template() {
        let (counter, _) = record(vec![http_test::TestRequest::get().uri("/limited/1")]).await;

        assert_eq!(
            counter,
            vec![(
                series(&[
                    ("method", "GET"),
                    ("route", "/limited/{id}"),
                    ("status", "429")
                ]),
                1
            )]
        );
    }

    #[actix_web::test]
    async fn errors_raised_by_an_inner_middleware_are_counted_with_the_status_they_become() {
        let (counter, _) = record(vec![http_test::TestRequest::get().uri("/refused/1")]).await;

        // The request that carried the matched route does not come back with an `Err`.
        assert_eq!(
            counter,
            vec![(
                series(&[
                    ("method", "GET"),
                    ("route", UNMATCHED_ROUTE),
                    ("status", "401")
                ]),
                1
            )]
        );
    }

    #[test]
    fn status_classes_cover_every_code() {
        assert_eq!(status_class(StatusCode::CONTINUE), "1xx");
        assert_eq!(status_class(StatusCode::NO_CONTENT), "2xx");
        assert_eq!(status_class(StatusCode::NOT_MODIFIED), "3xx");
        assert_eq!(status_class(StatusCode::UNPROCESSABLE_ENTITY), "4xx");
        assert_eq!(status_class(StatusCode::GATEWAY_TIMEOUT), "5xx");
    }
}
