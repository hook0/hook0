---
title: Architecture
description: Hook0 instance architecture and components
keywords: [self-hosted webhooks, hook0 architecture, hook0 opentelemetry, webhook otlp, webhook observability]
faqItems:
  - question: "Does self-hosted Hook0 support OpenTelemetry?"
    answer: >-
      Yes. The Hook0 API and the output worker both export metrics, traces and
      logs over OTLP (HTTP with protobuf payloads, zstd compression). Each
      signal is off until you set its endpoint: OTLP_METRICS_ENDPOINT,
      OTLP_TRACES_ENDPOINT and OTLP_LOGS_ENDPOINT, plus OTLP_AUTHORIZATION when
      your collector expects an Authorization header.
  - question: "Which OTLP endpoint URL does Hook0 expect?"
    answer: >-
      The full URL of each signal, used as is, for example
      http://otel-collector:4318/v1/metrics for metrics. Point the three
      variables at an OpenTelemetry Collector or at any backend that accepts
      OTLP over HTTP.
  - question: "How do I tell the API and the worker apart in my telemetry?"
    answer: >-
      Every signal carries service.namespace=hook0 and service.name=api or
      service.name=output-worker, along with service.version and a
      service.instance.id shared by the metrics, traces and logs of one
      process. Set service.instance.id through OTEL_RESOURCE_ATTRIBUTES to
      choose it, otherwise Hook0 generates one at startup.
---

# Architecture

A Hook0 instance is composed of multiple parts. Let's explain what they are used for!

## Hook0 API

The central part of the system, a Rust web application.

**Dependencies:**
- PostgreSQL database

## Hook0 UI

A Vue.js front-end web application.

**Dependencies:**
- Hook0 API

## Hook0 Output Worker

Responsible for actually calling users' webhooks and gathering responses. A Rust application that does not need to accept incoming connections.

**Dependencies:**
- PostgreSQL database used by Hook0 API

:::note

There can be multiple instances of Hook0 Output Worker, work would be shared between each of them.

:::

## Observability with OpenTelemetry

The Hook0 API and the output worker export metrics, traces and logs over OTLP, the OpenTelemetry protocol. They send HTTP requests with protobuf payloads, compressed with zstd, so any OpenTelemetry Collector or OTLP-compatible backend can receive them. Nothing is exported until you configure it, and each signal has its own endpoint:

| Variable | Signal |
|----------|--------|
| `OTLP_METRICS_ENDPOINT` | Metrics (for example `http://otel-collector:4318/v1/metrics`) |
| `OTLP_TRACES_ENDPOINT` | Traces (for example `http://otel-collector:4318/v1/traces`) |
| `OTLP_LOGS_ENDPOINT` | Logs (for example `http://otel-collector:4318/v1/logs`) |
| `OTLP_AUTHORIZATION` | Optional `Authorization` header sent with every export, such as `Bearer mytoken` |

Set the same variables on the API and on every worker. Hook0 uses each URL as is, so include the `/v1/...` path. Both components label what they send with `service.namespace=hook0` and `service.name=api` or `service.name=output-worker`, plus `service.version` and one `service.instance.id` per process, so you can filter a single worker across the three signals. To choose that instance ID yourself, set `OTEL_RESOURCE_ATTRIBUTES=service.instance.id=<value>`.

The metrics cover both sides of delivery, with event ingestion and its duration on the API, delivery outcomes, abandoned deliveries, delivery lag, delivery slots and Pulsar consumer state on the worker, and database connection pool usage on both. The full list of monitoring variables, Sentry included, is in [Configuration](../reference/configuration.md).

## Email your instance sends on its own

Besides the transactional messages a user's own action triggers (address verification, password reset), a self-hosted instance sends one automated sequence: a short onboarding drip to accounts that verified their address but never ingested an event, at J+1, J+3 and J+7 after sign-up. It stops as soon as the account sends its first event, only ever targets sign-ups younger than 30 days, and every message carries a one-click opt-out link.

It is on by default because an instance that never nudges a stuck user is the more common failure. If you would rather Hook0 never emailed your users, set `ENABLE_REACTIVATION_EMAILS=false` — see [Configuration](../reference/configuration.md#reactivation) for that and the other knobs (cadence, per-pass cap, CTA URLs).

## FAQ

### Does self-hosted Hook0 support OpenTelemetry?

Yes. The Hook0 API and the output worker both export metrics, traces and logs over OTLP (HTTP with protobuf payloads, zstd compression). Each signal is off until you set its endpoint: `OTLP_METRICS_ENDPOINT`, `OTLP_TRACES_ENDPOINT` and `OTLP_LOGS_ENDPOINT`, plus `OTLP_AUTHORIZATION` when your collector expects an `Authorization` header.

### Which OTLP endpoint URL does Hook0 expect?

The full URL of each signal, used as is, for example `http://otel-collector:4318/v1/metrics` for metrics. Point the three variables at an OpenTelemetry Collector or at any backend that accepts OTLP over HTTP.

### How do I tell the API and the worker apart in my telemetry?

Every signal carries `service.namespace=hook0` and `service.name=api` or `service.name=output-worker`, along with `service.version` and a `service.instance.id` shared by the metrics, traces and logs of one process. Set `service.instance.id` through `OTEL_RESOURCE_ATTRIBUTES` to choose it, otherwise Hook0 generates one at startup.

## What's Next?

- [Bare Metal](bare-metal.md)
- [Docker Compose](docker-compose.md)
- [Kubernetes](kubernetes.md)
- [AWS](aws.md)
- [Master API Key](master-api-key.md)
