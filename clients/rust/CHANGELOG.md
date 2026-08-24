# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [sdk-v2.0.0] - 2026-08-24

### Added

- Ship SDKs for eleven languages, generated from a specification that now matches the API (clients,api)

### Fixed

- Bump the version a client says it is, and re-cut 2.0.0 (release)

### Other

- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies

## [sdk-v1.1.0] - 2026-04-03

### Added

- Add a Rust client
- Improve payload handling and documentation
- Initialize a Hook0 client in Hook0 API
- Implement Hook0 client for some events
- Support the new way of handling payload content in events
- Upsert Hook0 client event types on startup
- Add a Cargo feature to use rustls-native-certs for outgoing HTTPS requests
- IAM v2
- Update api keywords (api)
- Update dependencies
- Add a function to verify the webhooks signature & unit tests (clients)
- Add 2 examples and some edits (clients)
- Add producer & consumer features (clients)
- Change version of rust sdk and add README.md (clients)
- Improve Cargo features selection and testing (clients/rust)
- Add badges to readme (clients/rust)
- Support v1 signature verification (clients/rust)
- Add comprehensive GitLab CI release automation workflow with version management and changelog generation
- Migrate Rust loggers from log to tracing
- Improve log messages (output-worker)
- Add sdk release workflow with manual triggers (release)
- Make event ID optional and use UUIDv7 in big tables

### CI/CD

- Split rust job
- Check frontend & avoid downloading previous artifacts if not necessary
- Improve semver check (client/rust)
- Fix cargo-semver-checks (clients/rust)

### Changed

- Add semver checks in CI, fixes #48 (repo)
- Refactor(clients)
- Refactor(clients)
- Remove simple.rs example (clients)
- Rename hook0-webhook-producer or consumer to consumer & producer (clients.rust)
- Refactor(clients.rust)
- Refactor(clients.rust)
- Factorize some code (clients/rust)
- Rename type parameters (clients/rust)
- Metadata and labels validation

### Documentation

- Add some API documentation
- Improve READMEs (clients)
- Add license (client/rust)
- Change copyright (client/rust)

### Fixed

- Errors and warnings
- Disable jobs when on schedule (gitlab-ci)
- Remove deprecated only/except & replace with rules (gitlab-ci)
- Replace deprecated reqwest feature
- Fix(ci)
- Pipeline fix (clients)
- Remove a useless export (clients/rust)
- Reqwest 0.13 upgrade

### Other

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
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Mention the license in README.md (clients/rust)
- Update the license on manifest (clients/rust)
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
- Update dependencies & fix Dockerfiles
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
- Update dependencies
- Update dependencies

### Testing

- Add tests for v1 signature verification (clients/rust)

[sdk-v2.0.0]: https://gitlab.com/hook0/hook0/-/compare/sdk-v1.1.0...sdk-v2.0.0
[sdk-v1.1.0]: https://gitlab.com/hook0/hook0/-/compare/sdk-v1.0.1...sdk-v1.1.0

