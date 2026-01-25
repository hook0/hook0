use clap::crate_name;
use log::{info, warn};
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::{
    Compression, ExporterBuildError, MetricExporter, Protocol, SpanExporter, WithExportConfig,
    WithHttpConfig,
};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use sqlx::PgPool;
use std::collections::HashMap;
use std::time::Duration;
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

pub fn gather_pools_metrics(pools: &[(&'static str, PgPool)]) {
    for (name, pool) in pools {
        let max_connections = u64::from(pool.options().get_max_connections());
        let opened_connections = u64::from(pool.size());
        let idle_connections = u64::try_from(pool.num_idle())
            .inspect_err(|e| warn!("Could not convert {} to u64: {e}", pool.num_idle()))
            .ok();
        let active_connections = idle_connections.map(|idle| opened_connections - idle);

        let meter = global::meter(crate_name!());
        let attributes = [KeyValue::new("pool_name", *name)];

        meter
            .u64_gauge("db.max_connections")
            .build()
            .record(max_connections, &attributes);
        meter
            .u64_gauge("db.opened_connections")
            .build()
            .record(opened_connections, &attributes);
        if let Some(value) = idle_connections {
            meter
                .u64_gauge("db.idle_connections")
                .build()
                .record(value, &attributes);
        }
        if let Some(value) = active_connections {
            meter
                .u64_gauge("db.active_connections")
                .build()
                .record(value, &attributes);
        }
    }
}

pub fn report_rate_limiters_metrics(rate_limiters: &[(&'static str, usize)]) {
    for (name, len) in rate_limiters {
        let len_u64 = u64::try_from(*len)
            .inspect_err(|e| warn!("Could not convert {len} to u64: {e}"))
            .ok();

        let meter = global::meter(crate_name!());
        let attributes = [KeyValue::new("key", *name)];

        if let Some(value) = len_u64 {
            meter
                .u64_gauge("rate_limiter.len")
                .build()
                .record(value, &attributes);
        }
    }
}

pub fn report_cancelled_request_attempts(amount: u64) {
    let meter = global::meter(crate_name!());
    let counter = meter.u64_counter("cancelled_request_attempts").build();
    counter.add(amount, &[]);
}

pub fn report_cleaned_up_objects(amount: u64) {
    let meter = global::meter(crate_name!());
    let counter = meter
        .u64_counter("object_storage.cleaned_up_objects")
        .build();
    counter.add(amount, &[]);
}

pub fn report_ingested_events(amount: u64) {
    let meter = global::meter(crate_name!());
    let counter = meter.u64_counter("events.ingested").build();
    counter.add(amount, &[]);
}

pub fn report_event_payloads_stored_in_object_storage(amount: u64) {
    let meter = global::meter(crate_name!());
    let counter = meter
        .u64_counter("events.payloads_stored_in_object_storage")
        .build();
    counter.add(amount, &[]);
}

pub fn report_request_attempts_sent_to_pulsar(amount: u64) {
    let meter = global::meter(crate_name!());
    let counter = meter
        .u64_counter("events.request_attempts_sent_to_pulsar")
        .build();
    counter.add(amount, &[]);
}

pub fn report_replayed_events(amount: u64) {
    let meter = global::meter(crate_name!());
    let counter = meter.u64_counter("events.replayed").build();
    counter.add(amount, &[]);
}
