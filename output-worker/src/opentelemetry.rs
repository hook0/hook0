use clap::crate_name;
use log::info;
use opentelemetry::global::BoxedSpan;
use opentelemetry::trace::noop::NoopTracerProvider;
use opentelemetry::trace::{Span, Tracer};
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::{
    ExporterBuildError, Protocol, SpanExporter, WithExportConfig, WithHttpConfig,
};
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use std::collections::HashMap;

use crate::work::Response;
use crate::{Config, RequestAttempt};

pub fn init(config: &Config) -> Result<(), ExporterBuildError> {
    if let Some(endpoint) = &config.otlp_endpoint {
        let mut builder = SpanExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .with_endpoint(endpoint.as_str());
        if let Some(auth) = &config.otlp_authorization {
            builder = builder.with_headers(HashMap::from_iter([(
                "Authorization".to_owned(),
                auth.to_owned(),
            )]));
        }
        let otlp_exporter = builder.build()?;
        let tracer_provider = SdkTracerProvider::builder()
            .with_batch_exporter(otlp_exporter)
            .with_resource(
                Resource::builder()
                    .with_attributes([
                        KeyValue::new("service.name", "hook0"),
                        KeyValue::new("worker.name", config.worker_name.clone()),
                    ])
                    .build(),
            )
            .build();
        global::set_tracer_provider(tracer_provider);

        info!("OpenTelemetry spans will be exported to {endpoint}");
    } else {
        global::set_tracer_provider(NoopTracerProvider::new());
    };

    Ok(())
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
