use chrono::{DateTime, Utc};
use clap::crate_name;
use opentelemetry::global::BoxedSpan;
use opentelemetry::metrics::{Counter, Gauge, Histogram};
use opentelemetry::trace::noop::NoopTracerProvider;
use opentelemetry::trace::{Span, Tracer};
use opentelemetry::{Key, KeyValue, global};
use opentelemetry_otlp::{
    Compression, ExporterBuildError, MetricExporter, Protocol, SpanExporter, WithExportConfig,
    WithHttpConfig,
};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::metrics::{Aggregation, Instrument, SdkMeterProvider, Stream};
use opentelemetry_sdk::resource::EnvResourceDetector;
use opentelemetry_sdk::trace::SdkTracerProvider;
use pulsar::proto::CommandConsumerStatsResponse;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;
use tracing::{info, warn};
use uuid::Uuid;

use crate::work::{Response, ResponseError};
use crate::{Config, RequestAttempt};

pub struct OtlpExporters {
    metrics: MetricsExporter,
    traces: TracesExporter,
}

impl OtlpExporters {
    pub fn shutdown(&self) -> OTelSdkResult {
        match &self.metrics {
            MetricsExporter::Actual(exporter) => exporter.shutdown(),
            MetricsExporter::Noop => Ok(()),
        }?;
        match &self.traces {
            TracesExporter::Actual(exporter) => exporter.shutdown(),
            TracesExporter::Noop => Ok(()),
        }?;
        Ok(())
    }
}

enum MetricsExporter {
    Actual(SdkMeterProvider),
    Noop,
}

enum TracesExporter {
    Actual(SdkTracerProvider),
    Noop,
}

const SERVICE_INSTANCE_ID: &str = "service.instance.id";

fn service_instance_id() -> String {
    let detected = Resource::builder_empty()
        .with_detector(Box::new(EnvResourceDetector::new()))
        .build()
        .get(&Key::from_static_str(SERVICE_INSTANCE_ID))
        .map(|value| value.as_str().into_owned());

    pick_service_instance_id(detected.as_deref())
}

fn pick_service_instance_id(detected: Option<&str>) -> String {
    match detected.map(str::trim) {
        Some(id) if !id.is_empty() => id.to_owned(),
        _ => Uuid::now_v7().to_string(),
    }
}

pub fn init(config: &Config, version: &str) -> Result<OtlpExporters, ExporterBuildError> {
    let service_instance_id = service_instance_id();
    let resource = Resource::builder()
        .with_attributes([
            KeyValue::new("service.namespace", "hook0"),
            KeyValue::new("service.name", "output-worker"),
            KeyValue::new("service.version", version.to_owned()),
            KeyValue::new("worker.name", config.worker_name.clone()),
            KeyValue::new(SERVICE_INSTANCE_ID, service_instance_id.clone()),
        ])
        .build();
    let auth_header = config
        .otlp_authorization
        .as_ref()
        .map(|auth| HashMap::from_iter([("Authorization".to_owned(), auth.to_owned())]));

    let metrics_exporter = if let Some(metrics_endpoint) = &config.otlp_metrics_endpoint {
        let mut builder = MetricExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .with_compression(Compression::Zstd)
            .with_endpoint(metrics_endpoint.as_str())
            .with_timeout(Duration::from_secs(1));
        if let Some(auth) = &auth_header {
            builder = builder.with_headers(auth.clone());
        }
        let otlp_exporter = builder.build()?;
        let metrics_provider = SdkMeterProvider::builder()
            .with_periodic_exporter(otlp_exporter)
            .with_view(worker_delivery_lag_seconds_view)
            .with_resource(resource.clone())
            .build();
        global::set_meter_provider(metrics_provider.clone());

        info!(
            "OpenTelemetry metrics will be exported to {metrics_endpoint} (service.instance.id={service_instance_id})"
        );
        MetricsExporter::Actual(metrics_provider)
    } else {
        MetricsExporter::Noop
    };

    let traces_exporter = if let Some(traces_endpoint) = &config.otlp_traces_endpoint {
        let mut builder = SpanExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .with_compression(Compression::Zstd)
            .with_endpoint(traces_endpoint.as_str())
            .with_timeout(Duration::from_secs(1));
        if let Some(auth) = auth_header {
            builder = builder.with_headers(auth);
        }
        let otlp_exporter = builder.build()?;
        let tracer_provider = SdkTracerProvider::builder()
            .with_batch_exporter(otlp_exporter)
            .with_resource(resource)
            .build();
        global::set_tracer_provider(tracer_provider.clone());

        info!(
            "OpenTelemetry traces will be exported to {traces_endpoint} (service.instance.id={service_instance_id})"
        );
        TracesExporter::Actual(tracer_provider)
    } else {
        let tracer_provider = NoopTracerProvider::new();
        global::set_tracer_provider(tracer_provider);
        TracesExporter::Noop
    };

    Ok(OtlpExporters {
        metrics: metrics_exporter,
        traces: traces_exporter,
    })
}

// These instruments are built once on first use and stay bound to the global
// meter provider that exists at that moment. This is safe because `init()` sets
// the provider during startup, before any of the functions below can be called.
// A new caller that runs before `init()` would bind its instrument to the no-op
// provider permanently.
static DB_MAX_CONNECTIONS: LazyLock<Gauge<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_gauge("db.max_connections")
        .build()
});
static DB_OPENED_CONNECTIONS: LazyLock<Gauge<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_gauge("db.opened_connections")
        .build()
});
static DB_IDLE_CONNECTIONS: LazyLock<Gauge<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_gauge("db.idle_connections")
        .build()
});
static DB_ACTIVE_CONNECTIONS: LazyLock<Gauge<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_gauge("db.active_connections")
        .build()
});

pub fn gather_pool_metrics(pool: &PgPool) {
    let max_connections = u64::from(pool.options().get_max_connections());
    let opened_connections = u64::from(pool.size());
    let idle_connections = u64::try_from(pool.num_idle())
        .inspect_err(|e| warn!("Could not convert {} to u64: {e}", pool.num_idle()))
        .ok();
    let active_connections = idle_connections.map(|idle| opened_connections - idle);

    DB_MAX_CONNECTIONS.record(max_connections, &[]);
    DB_OPENED_CONNECTIONS.record(opened_connections, &[]);
    if let Some(value) = idle_connections {
        DB_IDLE_CONNECTIONS.record(value, &[]);
    }
    if let Some(value) = active_connections {
        DB_ACTIVE_CONNECTIONS.record(value, &[]);
    }
}

static PULSAR_UNACKED_MESSAGES: LazyLock<Gauge<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_gauge("pulsar.request_attempt_consumer.unacked_messages")
        .build()
});
static PULSAR_BLOCKED_CONSUMER_ON_UNACKED_MSGS: LazyLock<Gauge<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_gauge("pulsar.request_attempt_consumer.blocked_consumer_on_unacked_msgs")
        .build()
});
static PULSAR_MSG_RATE_OUT: LazyLock<Gauge<f64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .f64_gauge("pulsar.request_attempt_consumer.msg_rate_out")
        .build()
});
static PULSAR_MSG_THROUGHPUT_OUT: LazyLock<Gauge<f64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .f64_gauge("pulsar.request_attempt_consumer.msg_throughput_out")
        .build()
});
static PULSAR_MSG_RATE_REDELIVER: LazyLock<Gauge<f64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .f64_gauge("pulsar.request_attempt_consumer.msg_rate_redeliver")
        .build()
});
static PULSAR_MESSAGE_ACK_RATE: LazyLock<Gauge<f64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .f64_gauge("pulsar.request_attempt_consumer.message_ack_rate")
        .build()
});
static PULSAR_AVAILABLE_PERMITS: LazyLock<Gauge<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_gauge("pulsar.request_attempt_consumer.available_permits")
        .build()
});

pub fn gather_pulsar_consumer_metrics(stats: &[CommandConsumerStatsResponse]) {
    for stat in stats {
        if let Some(value) = stat.unacked_messages {
            PULSAR_UNACKED_MESSAGES.record(value, &[]);
        }
        if let Some(value) = stat.blocked_consumer_on_unacked_msgs {
            PULSAR_BLOCKED_CONSUMER_ON_UNACKED_MSGS.record(u64::from(value), &[]);
        }
        if let Some(value) = stat.msg_rate_out {
            PULSAR_MSG_RATE_OUT.record(value, &[]);
        }
        if let Some(value) = stat.msg_throughput_out {
            PULSAR_MSG_THROUGHPUT_OUT.record(value, &[]);
        }
        if let Some(value) = stat.msg_rate_redeliver {
            PULSAR_MSG_RATE_REDELIVER.record(value, &[]);
        }
        if let Some(value) = stat.message_ack_rate {
            PULSAR_MESSAGE_ACK_RATE.record(value, &[]);
        }
        if let Some(value) = stat.available_permits {
            PULSAR_AVAILABLE_PERMITS.record(value, &[]);
        }
    }
}

static SLOTS_HP_AVAILABLE: LazyLock<Gauge<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_gauge("slots.hp_available")
        .build()
});
static SLOTS_LP_AVAILABLE: LazyLock<Gauge<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_gauge("slots.lp_available")
        .build()
});
static SLOTS_DYNAMIC_AVAILABLE: LazyLock<Gauge<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_gauge("slots.dynamic_available")
        .build()
});

pub fn gather_slot_metrics(hp_available: u64, lp_available: u64, dynamic_available: u64) {
    SLOTS_HP_AVAILABLE.record(hp_available, &[]);
    SLOTS_LP_AVAILABLE.record(lp_available, &[]);
    SLOTS_DYNAMIC_AVAILABLE.record(dynamic_available, &[]);
}

pub fn start_request_attempt_span(attempt: &RequestAttempt) -> BoxedSpan {
    let tracer = global::tracer(crate_name!());
    let mut span = tracer.start("request_attempt");
    span.set_attributes([
        KeyValue::new("application_id", attempt.application_id.to_string()),
        KeyValue::new("request_attempt_id", attempt.request_attempt_id.to_string()),
        KeyValue::new("event_id", attempt.event_id.to_string()),
        KeyValue::new("event_received_at", attempt.event_received_at.to_rfc3339()),
        KeyValue::new("subscription_id", attempt.subscription_id.to_string()),
        KeyValue::new("created_at", attempt.created_at.to_rfc3339()),
        KeyValue::new("retry_count", i64::from(attempt.retry_count)),
    ]);
    span
}

pub fn end_request_attempt_span(mut span: BoxedSpan, response: &Response) {
    span.set_attributes([
        KeyValue::new("success", response.is_success()),
        KeyValue::new("error", response.response_error__name().unwrap_or_default()),
        KeyValue::new(
            "http.response.status_code",
            response.http_code.map(i64::from).unwrap_or_default(),
        ),
    ]);
    span.end();
}

// Delivery lag: seconds between when an attempt was scheduled to be delivered and
// when a worker actually picked it up. This is emitted on the real delivery path
// (both the Postgres-polling and the Pulsar consumer), unlike the Pulsar consumer
// stats gauges which are gated behind an interval and only exist in Pulsar mode.
static WORKER_DELIVERY_LAG_SECONDS: LazyLock<Histogram<f64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .f64_histogram("worker.delivery_lag_seconds")
        .with_unit("s")
        .with_description("Time between an attempt's scheduled delivery and its pickup by a worker")
        .build()
});

pub fn report_worker_delivery_lag(lag_seconds: f64) {
    WORKER_DELIVERY_LAG_SECONDS.record(lag_seconds, &[]);
}

// SDK default boundaries are tuned for milliseconds; this metric is in seconds and
// ranges from sub-second up to the multi-minute backlogs we need to catch.
fn worker_delivery_lag_seconds_view(instrument: &Instrument) -> Option<Stream> {
    if instrument.name() == "worker.delivery_lag_seconds" {
        Stream::builder()
            .with_aggregation(Aggregation::ExplicitBucketHistogram {
                boundaries: vec![
                    0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0, 1800.0, 3600.0,
                ],
                record_min_max: true,
            })
            .build()
            .ok()
    } else {
        None
    }
}

/// Fractional seconds between an attempt's scheduled delivery time and `now`,
/// clamped to be non-negative. The scheduled time is `delay_until` when set,
/// otherwise `created_at`. Clamping guards against clock skew and attempts whose
/// delivery is scheduled in the future relative to the picking worker's clock.
pub fn compute_delivery_lag_seconds(
    now: DateTime<Utc>,
    created_at: DateTime<Utc>,
    delay_until: Option<DateTime<Utc>>,
) -> f64 {
    let scheduled = delay_until.unwrap_or(created_at);
    let lag_seconds = (now - scheduled).num_milliseconds() as f64 / 1000.0;
    lag_seconds.max(0.0)
}

static DELIVERY_OUTCOMES: LazyLock<Counter<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_counter("webhook.delivery.outcomes")
        .with_description("Count of webhook delivery attempts by bounded outcome")
        .build()
});

/// Bounded set of delivery outcomes. The metric attribute is derived exclusively
/// from this enum so the `outcome` label can never take an unbounded value (raw
/// status codes or error strings would blow up cardinality).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryOutcome {
    Success,
    Timeout,
    Http4xx,
    Http5xx,
    ConnectionError,
}

impl DeliveryOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeliveryOutcome::Success => "success",
            DeliveryOutcome::Timeout => "timeout",
            DeliveryOutcome::Http4xx => "http_4xx",
            DeliveryOutcome::Http5xx => "http_5xx",
            DeliveryOutcome::ConnectionError => "connection_error",
        }
    }
}

pub fn report_delivery_outcome(outcome: DeliveryOutcome) {
    DELIVERY_OUTCOMES.add(1, &[KeyValue::new("outcome", outcome.as_str())]);
}

/// Total mapping from a delivery `Response` to exactly one bounded `DeliveryOutcome`.
/// A success maps to `Success`; an HTTP error with a 4xx/5xx code maps to the
/// matching class; anything else falls back to the transport error (`Timeout` for a
/// timeout, `ConnectionError` otherwise).
pub fn classify_outcome(response: &Response) -> DeliveryOutcome {
    if response.is_success() {
        return DeliveryOutcome::Success;
    }
    if let Some(code) = response.http_code {
        match code {
            400..=499 => return DeliveryOutcome::Http4xx,
            500..=599 => return DeliveryOutcome::Http5xx,
            _ => {}
        }
    }
    match response.response_error {
        Some(ResponseError::Timeout) => DeliveryOutcome::Timeout,
        _ => DeliveryOutcome::ConnectionError,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    /// The complete, closed set of labels the `outcome` attribute may ever take.
    const OUTCOME_LABELS: [&str; 5] = [
        "success",
        "timeout",
        "http_4xx",
        "http_5xx",
        "connection_error",
    ];

    fn response(response_error: Option<ResponseError>, http_code: Option<u16>) -> Response {
        Response {
            response_error,
            http_code,
            headers: None,
            body: None,
            elapsed_time: Duration::from_secs(0),
        }
    }

    fn error_variant(sel: u8) -> Option<ResponseError> {
        match sel % 7 {
            0 => None,
            1 => Some(ResponseError::Unknown),
            2 => Some(ResponseError::InvalidHeader),
            3 => Some(ResponseError::InvalidTarget),
            4 => Some(ResponseError::Connection),
            5 => Some(ResponseError::Timeout),
            _ => Some(ResponseError::Http),
        }
    }

    #[test]
    fn classifies_common_statuses() {
        assert_eq!(
            classify_outcome(&response(Some(ResponseError::Http), Some(404))),
            DeliveryOutcome::Http4xx
        );
        assert_eq!(
            classify_outcome(&response(Some(ResponseError::Http), Some(503))),
            DeliveryOutcome::Http5xx
        );
        assert_eq!(
            classify_outcome(&response(None, Some(200))),
            DeliveryOutcome::Success
        );
        assert_eq!(
            classify_outcome(&response(Some(ResponseError::Timeout), None)),
            DeliveryOutcome::Timeout
        );
        assert_eq!(
            classify_outcome(&response(Some(ResponseError::Connection), None)),
            DeliveryOutcome::ConnectionError
        );
    }

    #[test]
    fn delivery_outcome_labels_are_the_closed_bounded_set() {
        let labels: BTreeSet<&str> = [
            DeliveryOutcome::Success,
            DeliveryOutcome::Timeout,
            DeliveryOutcome::Http4xx,
            DeliveryOutcome::Http5xx,
            DeliveryOutcome::ConnectionError,
        ]
        .iter()
        .map(DeliveryOutcome::as_str)
        .collect();
        let expected: BTreeSet<&str> = OUTCOME_LABELS.into_iter().collect();
        assert_eq!(labels, expected);
        // Each variant maps to a distinct label (no collisions).
        assert_eq!(labels.len(), 5);
    }

    #[test]
    fn future_scheduled_delivery_lag_clamps_to_zero() {
        let now = DateTime::from_timestamp(1_700_000_000, 0).expect("valid timestamp");
        let future = now + chrono::TimeDelta::seconds(120);
        assert_eq!(compute_delivery_lag_seconds(now, future, None), 0.0);
        assert_eq!(compute_delivery_lag_seconds(now, now, Some(future)), 0.0);
    }

    #[test]
    fn past_scheduled_delivery_lag_is_the_delta() {
        let now = DateTime::from_timestamp(1_700_000_000, 0).expect("valid timestamp");
        let past = now - chrono::TimeDelta::seconds(30);
        assert_eq!(compute_delivery_lag_seconds(now, past, None), 30.0);
        // `delay_until` wins over `created_at` when set.
        assert_eq!(
            compute_delivery_lag_seconds(now, now - chrono::TimeDelta::seconds(9999), Some(past)),
            30.0
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        // M1: an attempt scheduled in the future relative to `now` yields exactly
        // zero lag (never negative); a past-scheduled attempt yields the true
        // non-negative delta. `delay_until` (when set) is the scheduled time,
        // otherwise `created_at`.
        #[test]
        fn delivery_lag_clamps_future_and_matches_past(
            base_secs in 1_600_000_000i64..2_000_000_000i64,
            offset_ms in -10_000_000i64..10_000_000i64,
            use_delay in any::<bool>(),
        ) {
            let now = DateTime::from_timestamp(base_secs, 0).expect("valid timestamp");
            // Positive offset => scheduled in the past; negative => in the future.
            let scheduled = now - chrono::TimeDelta::milliseconds(offset_ms);

            let (created_at, delay_until) = if use_delay {
                // An arbitrary (unrelated) creation time must be ignored in favour of delay_until.
                (now - chrono::TimeDelta::seconds(9_999), Some(scheduled))
            } else {
                (scheduled, None)
            };

            let lag = compute_delivery_lag_seconds(now, created_at, delay_until);

            prop_assert!(lag >= 0.0, "lag must never be negative, got {lag}");
            if offset_ms <= 0 {
                prop_assert_eq!(lag, 0.0, "future/now scheduling must clamp to zero");
            } else {
                let expected = offset_ms as f64 / 1000.0;
                prop_assert!(
                    (lag - expected).abs() < 1e-6,
                    "past scheduling lag {lag} != expected {expected}"
                );
            }
        }

        // M2: for every possible HTTP status and every transport-error case,
        // `classify_outcome` returns a variant whose label is inside the fixed
        // bounded set — the attribute can never take an unbounded value.
        #[test]
        fn classify_outcome_label_is_always_bounded(
            code in any::<u16>(),
            err_sel in any::<u8>(),
            has_code in any::<bool>(),
        ) {
            let http_code = if has_code { Some(code) } else { None };
            let outcome = classify_outcome(&response(error_variant(err_sel), http_code));
            prop_assert!(
                OUTCOME_LABELS.contains(&outcome.as_str()),
                "outcome label {} escaped the bounded set",
                outcome.as_str()
            );
        }
    }

    #[test]
    fn detected_instance_id_wins() {
        assert_eq!(
            pick_service_instance_id(Some("worker-pulsar-2")),
            "worker-pulsar-2"
        );
    }

    #[test]
    fn blank_detected_instance_id_falls_back_to_a_uuid() {
        // `OTEL_RESOURCE_ATTRIBUTES=service.instance.id=` yields an empty value.
        // Honouring it would give every process the same empty `instance` label,
        // which is exactly the collision this attribute exists to prevent.
        assert!(Uuid::parse_str(&pick_service_instance_id(Some("  "))).is_ok());
    }

    #[test]
    fn generated_instance_ids_are_unique() {
        let first = pick_service_instance_id(None);
        let second = pick_service_instance_id(None);

        assert!(Uuid::parse_str(&first).is_ok());
        assert_ne!(first, second);
    }
}
