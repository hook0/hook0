# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [cli/v1.0.1] - 2026-05-10

### Added

- Add hook0 CLI with full API coverage (cli)
- Make event ID optional and use UUIDv7 in big tables
- Web UI for webhook tester + Redis storage + cross-property navigation (play)
- Per-package release flow + monorepo tag convention (ci)

### CI/CD

- Enable CI (cli)
- Run API for E2E tests (cli)
- Fix build (cli)
- Fix build (cli)

### Documentation

- Add CLI reference documentation
- Improve CLI reference wording and structure

### Fixed

- Opt cli/play/mcp into cargo-release + serialize trigger jobs (ci)

### Other

- Update dependencies
- Update dependencies
- Update dependencies
- Sort dependencies (cli)
- Update Rust Edition to 2024 (cli)
- Update dependencies
- Update keyring (cli)

### Testing

- Do not run tests with external dependencies by default (cli)

# Changelog — cli

All notable changes to the Hook0 CLI (`hook0`) are documented here.

Tags follow the convention `cli/vX.Y.Z` — see [ADR 0004](../adr/0004-monorepo-tag-convention.md).
