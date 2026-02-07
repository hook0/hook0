use clap::crate_name;
use log::{info, warn};
use opentelemetry::global::BoxedSpan;
use opentelemetry::trace::noop::NoopTracerProvider;
use opentelemetry::trace::{Span, Tracer};
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::{
    Compression, ExporterBuildError, MetricExporter, Protocol, SpanExporter, WithExportConfig,
    WithHttpConfig,
};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use pulsar::proto::CommandConsumerStatsResponse;
use sqlx::PgPool;
use std::collections::HashMap;
use std::time::Duration;

use crate::work::Response;
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

pub fn init(config: &Config, version: &str) -> Result<OtlpExporters, ExporterBuildError> {
    let resource = Resource::builder()
        .with_attributes([
            KeyValue::new("service.namespace", "hook0"),
            KeyValue::new("service.name", "output-worker"),
            KeyValue::new("service.version", version.to_owned()),
            KeyValue::new("worker.name", config.worker_name.clone()),
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
            .with_resource(resource.clone())
            .build();
        global::set_meter_provider(metrics_provider.clone());

        info!("OpenTelemetry metrics will be exported to {metrics_endpoint}");
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

        info!("OpenTelemetry traces will be exported to {traces_endpoint}");
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

pub fn gather_pool_metrics(pool: &PgPool) {
    let max_connections = u64::from(pool.options().get_max_connections());
    let opened_connections = u64::from(pool.size());
    let idle_connections = u64::try_from(pool.num_idle())
        .inspect_err(|e| warn!("Could not convert {} to u64: {e}", pool.num_idle()))
        .ok();
    let active_connections = idle_connections.map(|idle| opened_connections - idle);

    let meter = global::meter(crate_name!());
    meter
        .u64_gauge("db.max_connections")
        .build()
        .record(max_connections, &[]);
    meter
        .u64_gauge("db.opened_connections")
        .build()
        .record(opened_connections, &[]);
    if let Some(value) = idle_connections {
        meter
            .u64_gauge("db.idle_connections")
            .build()
            .record(value, &[]);
    }
    if let Some(value) = active_connections {
        meter
            .u64_gauge("db.active_connections")
            .build()
            .record(value, &[]);
    }
}

pub fn gather_pulsar_consumer_metrics(stats: &[CommandConsumerStatsResponse]) {
    let meter = global::meter(crate_name!());

    for stat in stats {
        if let Some(value) = stat.unacked_messages {
            meter
                .u64_gauge("pulsar.request_attempt_consumer.unacked_messages")
                .build()
                .record(value, &[]);
        }
        if let Some(value) = stat.blocked_consumer_on_unacked_msgs {
            meter
                .u64_gauge("pulsar.request_attempt_consumer.blocked_consumer_on_unacked_msgs")
                .build()
                .record(u64::from(value), &[]);
        }
        if let Some(value) = stat.msg_rate_out {
            meter
                .f64_gauge("pulsar.request_attempt_consumer.msg_rate_out")
                .build()
                .record(value, &[]);
        }
        if let Some(value) = stat.msg_throughput_out {
            meter
                .f64_gauge("pulsar.request_attempt_consumer.msg_throughput_out")
                .build()
                .record(value, &[]);
        }
        if let Some(value) = stat.msg_rate_redeliver {
            meter
                .f64_gauge("pulsar.request_attempt_consumer.msg_rate_redeliver")
                .build()
                .record(value, &[]);
        }
        if let Some(value) = stat.message_ack_rate {
            meter
                .f64_gauge("pulsar.request_attempt_consumer.message_ack_rate")
                .build()
                .record(value, &[]);
        }
        if let Some(value) = stat.available_permits {
            meter
                .u64_gauge("pulsar.request_attempt_consumer.available_permits")
                .build()
                .record(value, &[]);
        }
    }
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
