# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [mcp/v1.0.1] - 2026-05-10

### Added

- Add Hook0 MCP server for AI assistant integration (mcp)
- Per-package release flow + monorepo tag convention (ci)

### CI/CD

- Fix build

### Fixed

- Convert SVG files from LFS to regular git files (frontend)
- Opt cli/play/mcp into cargo-release + serialize trigger jobs (ci)

### Other

- Update dependencies
- Avoid running MCP integration tests by default
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
- Update dependencies
- Update dependencies

### Testing

- Do not run tests with external dependencies by default (cli)

# Changelog — mcp

All notable changes to the Hook0 MCP server are documented here.

Tags follow the convention `mcp/vX.Y.Z` — see [ADR 0004](../../adr/0004-monorepo-tag-convention.md).
The legacy tag `hook0-mcp-v1.0.0` is preserved untouched for backward compatibility.
