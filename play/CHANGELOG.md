# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [play/v1.0.2] - 2026-05-10

### Added

- Add hook0 CLI with full API coverage (cli)
- Web UI for webhook tester + Redis storage + cross-property navigation (play)
- SEO comparison pages, glossary, schema markup (website,docs)
- Per-package release flow + monorepo tag convention (ci)

### CI/CD

- Validate Dockerfile workspace coverage and migrate play.docker to BuildKit

### Fixed

- Replace Traefik IngressRoute with Kubernetes Ingress (kingress) (play)
- Deploy job, smoke tests, E2E Playwright CI, auto-select first webhook, CLI link fix, pullPolicy fix (play)
- Opt cli/play/mcp into cargo-release + serialize trigger jobs (ci)

### Other

- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Sort dependencies (play)
- Update dependencies

## [play/v1.0.1] - 2026-05-10

### Added

- Add hook0 CLI with full API coverage (cli)
- Web UI for webhook tester + Redis storage + cross-property navigation (play)
- SEO comparison pages, glossary, schema markup (website,docs)
- Per-package release flow + monorepo tag convention (ci)

### CI/CD

- Validate Dockerfile workspace coverage and migrate play.docker to BuildKit

### Fixed

- Replace Traefik IngressRoute with Kubernetes Ingress (kingress) (play)
- Deploy job, smoke tests, E2E Playwright CI, auto-select first webhook, CLI link fix, pullPolicy fix (play)
- Opt cli/play/mcp into cargo-release + serialize trigger jobs (ci)

### Other

- Update dependencies
- Update dependencies
- Update dependencies
- Update dependencies
- Sort dependencies (play)
- Update dependencies

# Changelog — play

All notable changes to Hook0 Play (`play.hook0.com`) are documented here.

Tags follow the convention `play/vX.Y.Z` — see [ADR 0004](../adr/0004-monorepo-tag-convention.md).
