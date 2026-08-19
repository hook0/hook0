# Changelog

All notable changes to the Hook0 MCP server are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Tags follow the convention `mcp/vX.Y.Z`, described in
[ADR 0004](../../adr/0004-monorepo-tag-convention.md). The legacy tag `hook0-mcp-v1.0.0` is
preserved untouched for backward compatibility.

## [Unreleased]

The first release of this crate that is not backward compatible. Three of the changes below break
a caller, and each says what to do about it.

### Changed

- **Breaking.** Protocol revision `2026-07-28` is no longer claimed (mcp)

  The server implements `2024-11-05`, `2025-03-26`, `2025-06-18` and `2025-11-25`, advertises the
  newest of those, and negotiates a client asking for anything else down to it rather than turning
  it away. A request whose inline `_meta` names an unlisted revision is refused with
  `-32022 Unsupported protocol version`. A client that speaks `2026-07-28` and nothing older cannot
  use this server. The upgrade to rmcp 3 is what made the difference visible, since rmcp's default
  advertised every revision the library knows, including one this server does not implement.

- **Breaking.** The library surface gained a query string (mcp)

  `GeneratedToolInfo` carries a `query_parameters` field, and `get`, `post`, `put`, `patch` and
  `delete` on `Hook0Client` each take the query as an argument. Code calling `client.get(path)` as a
  library stops compiling; pass `&[]` where there is nothing to send. This is what took the crate to
  `2.0.0`, and `cargo semver-checks` reads it as `v1.0.2 -> v2.0.0 (major change)`.

- Tool definitions are read from the committed API document instead of the network (mcp)

  The build script fetched the live swagger at compile time and, whenever that fetch failed, wrote
  an empty tool list and let the build succeed. An offline build therefore produced a server that
  starts, answers and exposes no tool at all. The definitions now travel inside the published
  package, generated from `api/openapi.snapshot.json`, and anything the generator cannot turn into a
  tool fails the build instead of being skipped.

### Added

- Say which client is talking, and the retry policy behind the request (mcp)

  Every request carries `User-Agent` and `Hook0-Client-Options`, the two headers the eleven SDKs
  already sent. An instance reading its logs can now tell this server from an SDK, and tell which
  version of it is still reaching them.

- Answer `--version` and `--help` (mcp)

  Both print before any configuration is read, so they work on a machine with no token set. An
  argument the binary has no meaning for is reported and exits in failure rather than being dropped,
  so a mistyped flag no longer starts a server that looks like it took the flag.

### Fixed

- Send the query arguments the tools ask their callers for (mcp)

  Ten tools declared arguments that only make sense in a query string, the organization a listing
  selects on and the filters a request-attempt search takes among them, and then dropped them. The
  request went out bare, so the answer was to a different question than the one asked, and nothing
  said so.

- Stop announcing a JSON body on requests that carry none (mcp)

  `Content-Type` was set on the HTTP client rather than on the request, so all thirteen read tools
  declared a body they did not have. The shared conformance corpus scopes that header to a request
  carrying one.

- Point the prompts at tools that exist (mcp)

  The three guided prompts named eleven tools under names the server has never exposed, so an
  assistant following one called a tool that is not there and the answer was the only thing that
  said so. A test now holds every name a prompt cites against the generated table.

- Correct the tool table in the README (mcp)

  It listed fifteen tools under names that were never the server's; the names have always been the
  operation ids the API document carries. A test reads the table back out of the file and compares
  it with a running server.

## [mcp/v1.0.2] - 2026-05-11

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
