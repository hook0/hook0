use clap::crate_name;
use opentelemetry::metrics::{Counter, Gauge, Histogram};
use opentelemetry::{Key, KeyValue, global};
use opentelemetry_otlp::{
    Compression, ExporterBuildError, MetricExporter, Protocol, SpanExporter, WithExportConfig,
    WithHttpConfig,
};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::{Aggregation, Instrument, SdkMeterProvider, Stream};
use opentelemetry_sdk::resource::EnvResourceDetector;
use opentelemetry_sdk::trace::SdkTracerProvider;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;
use tracing::{info, warn};
use url::Url;
use uuid::Uuid;

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

pub fn init(
    version: &str,
    otlp_authorization: &Option<String>,
    otlp_metrics_endpoint: &Option<Url>,
    otlp_traces_endpoint: &Option<Url>,
) -> Result<(), ExporterBuildError> {
    let service_instance_id = service_instance_id();
    let resource = Resource::builder()
        .with_attributes([
            KeyValue::new("service.namespace", "hook0"),
            KeyValue::new("service.name", "api"),
            KeyValue::new("service.version", version.to_owned()),
            KeyValue::new(SERVICE_INSTANCE_ID, service_instance_id.clone()),
        ])
        .build();
    let auth_header = otlp_authorization
        .as_ref()
        .map(|auth| HashMap::from_iter([("Authorization".to_owned(), auth.to_owned())]));

    if let Some(metrics_endpoint) = &otlp_metrics_endpoint {
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
            .with_view(health_check_duration_view)
            .with_view(authorizer_duration_view)
            .with_view(ingestion_duration_view)
            .with_view(ingestion_phase_duration_view)
            .with_resource(resource.clone())
            .build();
        global::set_meter_provider(metrics_provider.clone());

        info!(
            "OpenTelemetry metrics will be exported to {metrics_endpoint} (service.instance.id={service_instance_id})"
        );
    };

    if let Some(traces_endpoint) = &otlp_traces_endpoint {
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
    };

    Ok(())
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

/// `size()` and `num_idle()` are two separate reads of a pool that other tasks
/// keep mutating, so they never form a consistent snapshot: a connection
/// returned between the two makes `num_idle()` the larger of the pair. A plain
/// subtraction then wraps and records a `u64` gauge near `2^64`, which reads as
/// a saturation spike that never happened.
fn checked_out_connections(opened: u64, idle: u64) -> u64 {
    opened.saturating_sub(idle)
}

pub fn gather_pools_metrics(pools: &[(&'static str, PgPool)]) {
    for (name, pool) in pools {
        let max_connections = u64::from(pool.options().get_max_connections());
        let opened_connections = u64::from(pool.size());
        let idle_connections = u64::try_from(pool.num_idle())
            .inspect_err(|e| warn!("Could not convert {} to u64: {e}", pool.num_idle()))
            .ok();
        let active_connections =
            idle_connections.map(|idle| checked_out_connections(opened_connections, idle));

        let attributes = [KeyValue::new("pool_name", *name)];

        DB_MAX_CONNECTIONS.record(max_connections, &attributes);
        DB_OPENED_CONNECTIONS.record(opened_connections, &attributes);
        if let Some(value) = idle_connections {
            DB_IDLE_CONNECTIONS.record(value, &attributes);
        }
        if let Some(value) = active_connections {
            DB_ACTIVE_CONNECTIONS.record(value, &attributes);
        }
    }
}

static RATE_LIMITER_LEN: LazyLock<Gauge<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_gauge("rate_limiter.len")
        .build()
});

pub fn report_rate_limiters_metrics(rate_limiters: &[(&'static str, usize)]) {
    for (name, len) in rate_limiters {
        let len_u64 = u64::try_from(*len)
            .inspect_err(|e| warn!("Could not convert {len} to u64: {e}"))
            .ok();

        let attributes = [KeyValue::new("key", *name)];

        if let Some(value) = len_u64 {
            RATE_LIMITER_LEN.record(value, &attributes);
        }
    }
}

static CANCELLED_REQUEST_ATTEMPTS: LazyLock<Counter<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_counter("cancelled_request_attempts")
        .build()
});

pub fn report_cancelled_request_attempts(amount: u64) {
    CANCELLED_REQUEST_ATTEMPTS.add(amount, &[]);
}

static CLEANED_UP_OBJECTS: LazyLock<Counter<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_counter("object_storage.cleaned_up_objects")
        .build()
});

pub fn report_cleaned_up_objects(amount: u64) {
    CLEANED_UP_OBJECTS.add(amount, &[]);
}

static INGESTED_EVENTS: LazyLock<Counter<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_counter("events.ingested")
        .build()
});

pub fn report_ingested_events(amount: u64) {
    INGESTED_EVENTS.add(amount, &[]);
}

static EVENT_PAYLOADS_STORED_IN_OBJECT_STORAGE: LazyLock<Counter<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_counter("events.payloads_stored_in_object_storage")
        .build()
});

pub fn report_event_payloads_stored_in_object_storage(amount: u64) {
    EVENT_PAYLOADS_STORED_IN_OBJECT_STORAGE.add(amount, &[]);
}

static EVENT_PAYLOADS_STORED_IN_DB_FALLBACK: LazyLock<Counter<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_counter("events.payloads_stored_in_db_fallback")
        .build()
});

pub fn report_event_payloads_stored_in_db_fallback(amount: u64) {
    EVENT_PAYLOADS_STORED_IN_DB_FALLBACK.add(amount, &[]);
}

static REQUEST_ATTEMPTS_SENT_TO_PULSAR: LazyLock<Counter<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_counter("events.request_attempts_sent_to_pulsar")
        .build()
});

pub fn report_request_attempts_sent_to_pulsar(amount: u64) {
    REQUEST_ATTEMPTS_SENT_TO_PULSAR.add(amount, &[]);
}

static REPLAYED_EVENTS: LazyLock<Counter<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_counter("events.replayed")
        .build()
});

pub fn report_replayed_events(amount: u64) {
    REPLAYED_EVENTS.add(amount, &[]);
}

static HEALTH_CHECK_DURATION: LazyLock<Histogram<f64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .f64_histogram("health_check.duration")
        .with_unit("s")
        .with_description("Duration of a /health subsystem probe")
        .build()
});

pub fn report_health_check_duration(
    subsystem: &'static str,
    outcome: &'static str,
    duration: Duration,
) {
    HEALTH_CHECK_DURATION.record(
        duration.as_secs_f64(),
        &[
            KeyValue::new("subsystem", subsystem),
            KeyValue::new("outcome", outcome),
        ],
    );
}

// SDK default boundaries are tuned for milliseconds; this metric is in seconds.
// The object storage probe alone runs into the seconds, so the top boundaries go
// well past it: once the highest finite bucket is reached, every quantile above
// it reports that same boundary and the tail stops being readable.
const HEALTH_CHECK_DURATION_BOUNDARIES: &[f64] = &[
    0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 7.5, 10.0, 15.0, 30.0, 60.0,
];

fn health_check_duration_view(instrument: &Instrument) -> Option<Stream> {
    if instrument.name() == "health_check.duration" {
        Stream::builder()
            .with_aggregation(Aggregation::ExplicitBucketHistogram {
                boundaries: HEALTH_CHECK_DURATION_BOUNDARIES.to_vec(),
                record_min_max: true,
            })
            .build()
            .ok()
    } else {
        None
    }
}

static AUTHORIZER_DURATION: LazyLock<Histogram<f64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .f64_histogram("biscuit_authorizer.duration")
        .with_unit("s")
        .with_description("Time spent in the Biscuit authorizer")
        .build()
});

pub fn report_authorizer_duration(outcome: &'static str, duration: Duration) {
    AUTHORIZER_DURATION.record(duration.as_secs_f64(), &[KeyValue::new("outcome", outcome)]);
}

fn authorizer_duration_view(instrument: &Instrument) -> Option<Stream> {
    if instrument.name() == "biscuit_authorizer.duration" {
        Stream::builder()
            .with_aggregation(Aggregation::ExplicitBucketHistogram {
                boundaries: vec![
                    0.0001, 0.00025, 0.0005, 0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25,
                ],
                record_min_max: true,
            })
            .build()
            .ok()
    } else {
        None
    }
}

static INGESTION_DURATION: LazyLock<Histogram<f64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .f64_histogram("events.ingestion.duration")
        .with_unit("s")
        .with_description("Duration of a successful event ingestion (POST /event)")
        .build()
});

pub fn report_ingestion_duration(duration: Duration) {
    INGESTION_DURATION.record(duration.as_secs_f64(), &[]);
}

const INGESTION_DURATION_BOUNDARIES: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0, 60.0,
];

fn ingestion_duration_view(instrument: &Instrument) -> Option<Stream> {
    if instrument.name() == "events.ingestion.duration" {
        Stream::builder()
            .with_aggregation(Aggregation::ExplicitBucketHistogram {
                boundaries: INGESTION_DURATION_BOUNDARIES.to_vec(),
                record_min_max: true,
            })
            .build()
            .ok()
    } else {
        None
    }
}

static INGESTION_PHASE_DURATION: LazyLock<Histogram<f64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .f64_histogram("events.ingestion.phase.duration")
        .with_unit("s")
        .with_description("Duration of one phase of a successful event ingestion")
        .build()
});

pub fn report_ingestion_phase_durations(phases: &[(&'static str, Duration)]) {
    for (phase, duration) in phases {
        INGESTION_PHASE_DURATION.record(duration.as_secs_f64(), &[KeyValue::new("phase", *phase)]);
    }
}

// Boundaries start lower than the ones of the whole ingestion: some phases are a
// single fast query and would otherwise all fall into the first bucket. They also
// have to end no earlier, because a phase is by construction shorter than the
// ingestion that contains it: stopping short would make the slowest phase drop
// into the overflow bucket while the whole request is still being measured, and
// the phase that carries the tail would be exactly the one we could not read.
const INGESTION_PHASE_DURATION_BOUNDARIES: &[f64] = &[
    0.0005, 0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 15.0,
    30.0, 60.0,
];

fn ingestion_phase_duration_view(instrument: &Instrument) -> Option<Stream> {
    if instrument.name() == "events.ingestion.phase.duration" {
        Stream::builder()
            .with_aggregation(Aggregation::ExplicitBucketHistogram {
                boundaries: INGESTION_PHASE_DURATION_BOUNDARIES.to_vec(),
                record_min_max: true,
            })
            .build()
            .ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_connections_above_opened_do_not_wrap_the_active_gauge() {
        // `size()` and `num_idle()` are read one after the other, so a connection
        // returned in between yields idle > opened. Subtracting would record a
        // gauge near `2^64` instead of an idle pool.
        assert_eq!(checked_out_connections(3, 5), 0);
    }

    #[test]
    fn checked_out_connections_are_opened_minus_idle() {
        assert_eq!(checked_out_connections(60, 0), 60);
        assert_eq!(checked_out_connections(27, 21), 6);
    }

    #[test]
    fn every_phase_of_an_ingestion_stays_measurable_up_to_the_whole_request() {
        // A phase is contained in the ingestion it belongs to, so it can never be
        // the longer of the two. If the phase histogram stopped earlier than the
        // whole-request one, a slow phase would land in the overflow bucket while
        // the request itself was still being measured — and every quantile above
        // that point would report the top boundary instead of the real duration,
        // hiding precisely the phase that carries the tail.
        let phase_ceiling = *INGESTION_PHASE_DURATION_BOUNDARIES.last().unwrap();
        let request_ceiling = *INGESTION_DURATION_BOUNDARIES.last().unwrap();

        assert!(
            phase_ceiling >= request_ceiling,
            "phase histogram stops at {phase_ceiling}s, whole request at {request_ceiling}s"
        );
    }

    #[test]
    fn duration_histograms_outlast_a_backend_that_hangs_for_ten_seconds() {
        // Object storage is on the request path of both an ingestion and a health
        // probe, and it is the slowest thing either of them waits on. Boundaries
        // that ended at ten seconds put a hung backend in the overflow bucket,
        // where its duration is no longer readable at all.
        for (name, boundaries) in [
            ("ingestion", INGESTION_DURATION_BOUNDARIES),
            ("ingestion phase", INGESTION_PHASE_DURATION_BOUNDARIES),
            ("health check", HEALTH_CHECK_DURATION_BOUNDARIES),
        ] {
            let ceiling = *boundaries.last().unwrap();
            assert!(
                ceiling > 10.0,
                "{name} histogram stops at {ceiling}s, too early to read a stalled backend"
            );
        }
    }

    #[test]
    fn duration_histogram_boundaries_are_strictly_increasing() {
        // The SDK takes the boundaries as given; an unordered or duplicated list
        // yields buckets that silently never match, so the guard above could pass
        // on a ceiling no sample can ever reach.
        for (name, boundaries) in [
            ("ingestion", INGESTION_DURATION_BOUNDARIES),
            ("ingestion phase", INGESTION_PHASE_DURATION_BOUNDARIES),
            ("health check", HEALTH_CHECK_DURATION_BOUNDARIES),
        ] {
            assert!(
                boundaries.windows(2).all(|pair| pair[0] < pair[1]),
                "{name} histogram boundaries are not strictly increasing: {boundaries:?}"
            );
            assert!(
                boundaries.first().is_some_and(|first| *first > 0.0),
                "{name} histogram starts at a non-positive boundary: {boundaries:?}"
            );
        }
    }

    #[test]
    fn detected_instance_id_wins() {
        assert_eq!(pick_service_instance_id(Some("api-7")), "api-7");
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
