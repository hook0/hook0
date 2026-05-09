# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [output-worker/v1.0.1] - 2026-05-09

### Added

- Use crate name as app name when connecting to PG
- Implement output worker
- Implement bounded linear retry delay
- Rewrite Sentry integration & use it in both apps
- Add authentication for application endpoints
- Store worker ID and version when processing
- Slow down or stop attempting requests after given numbers of retries
- Improve JWT claims extraction & update dependencies
- Use trust dns resolver and enable anyhow integration in Sentry
- Add more trace messages
- Sign webhook calls
- Improve the way event playload content types are handled
- Upsert response error names
- Add event type in webhook requests
- Add options to allow Sentry to trace and profile transactions
- Add a Cargo feature to use rustls-native-certs for outgoing HTTPS requests
- Add dedicated workers
- Allow to set default organization workers
- Allow to periodically send monitoring heartbeats (output-worker)
- Allow to forbid HTTP requests that target IPs that are not globally reachable (output-worker)
- Handle multiple request attempts concurrently (output-worker)
- Improve resilience (output-worker)
- When there is nothing to do, close DB transaction before waiting (output-worker)
- When there is nothing to do, close DB transaction before waiting (output-worker)
- Make timeouts configurable (output-worker)
- Update dependencies
- Configurable signature header name (worker)
- Improve usage on signature_header_name (worker)
- Support v1 signature verification (clients/rust)
- Add a Pulsar-based worker mode
- Use protobuf to serialize request attempts transported by Pulsar
- Allow to load waiting request attempts from database into Pulsar on startup (output-worker)
- Add support for storing events' payload in object storage
- Enable retries in object storage client
- Add support for storing responses' body and headers in object storage
- Add OTLP metrics/traces support (output-worker)
- Allow to configure maximum number of attempts for object storage operations
- Improve retry logic (output-worker)
- Add Hook0 MCP server for AI assistant integration (mcp)
- Add comprehensive GitLab CI release automation workflow with version management and changelog generation
- Allow to configure grace period for unfound request attempts in Pulsar mode (output-worker)
- Do not block start of worker when loading waiting request attempts into Pulsar (output-worker)
- Use latest behavior of S3 client
- Add timeouts to object storage operations
- Add Pulsar metrics (output-worker)
- Improve S3 errors messages
- Periodically display throughput information in logs (output-worker)
- Migrate Rust loggers from log to tracing
- Improve log messages (output-worker)
- Make Sentry SDK debug mode OFF by default
- Allow to configure sending default PII to Sentry
- Allow to configure sending spans to Sentry
- Add sdk release workflow with manual triggers (release)
- Improve throughtput log (output-worker)
- Make sure delayed request attempts are never processed ahead of time in Pulsar mode (output-worker)
- Improve log messages (output-worker)
- Improve static retry policy (output-worker)
- Make Pulsar worker only handle its own request attempts (output-worker)
- Per-package release flow + monorepo tag convention (ci)

### Build

- Use SQLx in offline mode

### CI/CD

- Split rust job
- Check frontend & avoid downloading previous artifacts if not necessary
- Update Clever Cloud Pipeline component
- Update deployment component
- Allow release jobs to run with alpha/beta/rc versions
- Validate Dockerfile workspace coverage and migrate play.docker to BuildKit

### Changed

- Git ignore files
- Let the DB compute the delay_until date
- Split logic into several files (output-worker)
- Make imports more readable
- Fix
- Improve SQL query coding style
- Improve worker type log message (output-worker)
- Pass the whole config to the work function (output-worker)
- Add semver checks in CI, fixes #48 (repo)
- Fix Dockerfile warnings
- Format code (output-worker)
- Improve signature (output-worker)
- Improve string interpolation (output-worker)
- Minor improvements
- Improve logs related to object storage operations (api)
- Improve logs related to object storage operations
- Improve error messages related to object storage
- Merge PG worker queries (output-worker)
- Use a struct instead of a tuple (output-worker)

### Fixed

- Move Cargo profile config to workspace root
- Typo
- Clippy warning
- Attempt picking
- Upgrade cargo version
- Add license
- Cargo warning
- Regen sqlx offline data
- Regen sqlx offline data
- Allow to soft delete subscriptions
- Regen sqlx offline data
- Retries limits (output-worker)
- Typo (output-worker)
- Allow a public worker to pick work dedicated to itself (output-worker)
- Disable jobs when on schedule (gitlab-ci)
- Remove deprecated only/except & replace with rules (gitlab-ci)
- Merge glitch (output-worker)
- Add missing info in some log messages (output-worker)
- Commit instead of rolling back (output-worker)
- Avoid doing a rollback even if it does not change much (output-worker)
- Enable a missing Cargo feature (output-worker)
- Automate output-worker deploy to clever (ci)
- Replace deprecated reqwest feature
- Docker deployment
- Remove wrong ignores
- Concurrent builds with cache (docker)
- Fix(output-worker)
- Private workers (output-worker)
- Avoid crashing if header names/values are invalid (output-worker)
- Send heartbeat in pulsar mode even if worker is idle (output-worker)
- Add protoc and protobuf mount to Docker build (api,output-worker)
- Prevent pg workers to pick up request attempts from disabled or deleted subscriptions (output-worker)
- Typo in metric name (output-worker)
- TLS crypto provider
- Switch release containers from docker:dind to BuildKit rootless (ci)
- Allow to disable Pulsar consumer stats (output-worker)
- Correctly handle waiting but delayed request attempts when loading them from DB in Pulsar mode (output-worker)
- Pick up assigned-to-self work for public PG workers (output-worker)
- Add ca-certificates to Dockerfile (output-worker)
- Ensure request attempts cannot be both succeeded and failed
- Make Pulsar worker more resilient to broker errors (output-worker)
- Initialize rustls crypto provider for all queue types (output-worker)
- Avoid processing the same request attempt several times concurrently in Pulsar mode (output-worker)
- Make sure Pulsar messages are ACKed by the broker

### Other

- Update SQLx
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Rename crates
- Update dependencies
- Update dependencies
- Update dependencies
- Chore(ignore)
- Migrate to actix-web 4
- Update dependencies
- Update dependencies
- Update dependencies
- Add missing env var
- Update dependencies
- Update dependencies
- Update to Rust 2021 edition
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Improve debug compile time
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Remove support of Sentry profiling
- Update dependencies
- Update dependencies
- Remove useless folder
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Remove keycloak deps (docker)
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update to Rust Edition 2024
- Update dependencies & improve mailer
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Improve logging (output-worker)
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies & fix Dockerfiles
- Update dependencies
- Update dependencies
- Change level of some log messages (output-worker)
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update to reqwest 0.13
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update to Rust 1.94
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Disable Pulsar consumer stats by default as it still needs some work (output-worker)

### Performance

- Try to reduce stress on the database when output worker is idle (output-worker)
- Add application ID to request_attempt table
- Share retry producer instead of always recreating it (output-worker)
- Split queue (output-worker)

# Changelog — output-worker

All notable changes to the Hook0 output worker are documented here.

Tags follow the convention `output-worker/vX.Y.Z` — see [ADR 0004](../adr/0004-monorepo-tag-convention.md).
Pre-rewrite history (versions `2.x` and earlier) lives in [CHANGELOG.legacy.md](../CHANGELOG.legacy.md) at the repo root.
