use clap::crate_name;
use opentelemetry::metrics::{Counter, Gauge, Histogram};
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::{
    Compression, ExporterBuildError, MetricExporter, Protocol, SpanExporter, WithExportConfig,
    WithHttpConfig,
};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::{Aggregation, Instrument, SdkMeterProvider, Stream};
use opentelemetry_sdk::trace::SdkTracerProvider;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;
use tracing::{info, warn};
use url::Url;

pub fn init(
    version: &str,
    otlp_authorization: &Option<String>,
    otlp_metrics_endpoint: &Option<Url>,
    otlp_traces_endpoint: &Option<Url>,
) -> Result<(), ExporterBuildError> {
    let resource = Resource::builder()
        .with_attributes([
            KeyValue::new("service.namespace", "hook0"),
            KeyValue::new("service.name", "api"),
            KeyValue::new("service.version", version.to_owned()),
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
            .with_resource(resource.clone())
            .build();
        global::set_meter_provider(metrics_provider.clone());

        info!("OpenTelemetry metrics will be exported to {metrics_endpoint}");
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

        info!("OpenTelemetry traces will be exported to {traces_endpoint}");
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

pub fn gather_pools_metrics(pools: &[(&'static str, PgPool)]) {
    for (name, pool) in pools {
        let max_connections = u64::from(pool.options().get_max_connections());
        let opened_connections = u64::from(pool.size());
        let idle_connections = u64::try_from(pool.num_idle())
            .inspect_err(|e| warn!("Could not convert {} to u64: {e}", pool.num_idle()))
            .ok();
        let active_connections = idle_connections.map(|idle| opened_connections - idle);

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
fn health_check_duration_view(instrument: &Instrument) -> Option<Stream> {
    if instrument.name() == "health_check.duration" {
        Stream::builder()
            .with_aggregation(Aggregation::ExplicitBucketHistogram {
                boundaries: vec![
                    0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 7.5, 10.0,
                ],
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
