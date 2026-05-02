---
title: Changelog
description: Hook0 changelog and release notes — breaking changes, new features, fixes, and SDK migration guidance for each release.
keywords:
  - hook0 changelog
  - hook0 release notes
  - hook0 migration
  - cursor pagination
---

# Changelog

For older versions, see [GitHub Releases](https://github.com/hook0/hook0/releases).

## Unreleased

### Added — Cursor pagination on `/event_types` and `/subscriptions`

Issue [#45](https://github.com/hook0/hook0/issues/45). The list endpoints `GET /api/v1/event_types` and `GET /api/v1/subscriptions` now support **cursor-based pagination**, matching the existing contract on `GET /api/v1/request_attempts`.

- New optional query parameter `pagination_cursor` (opaque base64 token).
- New optional query parameter `limit` (default: 100, min: 1, max: 100). Out-of-range values return HTTP 400.
- Responses carry `Link: <…>; rel="next"` and `Link: <…>; rel="prev"` headers ([RFC 8288](https://www.rfc-editor.org/rfc/rfc8288)) when more pages exist.

The previously documented `offset` and `sort` query parameters were never wired to these handlers and have been removed from the documentation. A `?sort=` parameter is filed as a follow-up.

See the [pagination contract](/openapi/intro#pagination) for the full specification, and the [event types](/concepts/event-types) and [subscriptions](/concepts/subscriptions) concept pages for usage examples.

#### Migrating

**SDK users.** Upgrade to the [TypeScript SDK 1.2.0+](/reference/sdk/javascript). The SDK transparently follows `Link: rel="next"` and returns the full result set — no caller-side change required.

**Direct API consumers.** Replace any reliance on the (never-implemented) `offset`/`sort` parameters with the cursor flow:

1. Call the list endpoint without `pagination_cursor` to fetch the first page.
2. Read the `Link` HTTP header. Parse the `rel="next"` URL using any RFC 8288 parser (or a small regex: `<([^>]+)>;\s*rel="next"`).
3. If a `rel="next"` URL is present, GET it. The next-link URL preserves your `limit`.
4. Repeat until no `rel="next"` is present.

Treat `pagination_cursor` as opaque — do not parse, mutate, or persist it. It encodes a position only and is not an authorization token.
