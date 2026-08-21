---
title: Bare Metal
description: Manual setup of Hook0 on bare metal servers
---

import DevOnlyWarning from './_dev-only-warning.mdx';

# Bare Metal

This guide provides complete instructions for deploying Hook0 on bare metal servers.

<DevOnlyWarning />

## Requirements

- A PostgreSQL 18+ database
- Node.js LTS
- Rust stable toolchain

## Installation Steps

### Repository Setup

Clone the repository from GitLab and navigate to the project directory:

```bash
git clone https://gitlab.com/hook0/hook0.git
cd hook0
```

### UI Building

The frontend is built by Vite, which passes an environment variable through to the bundle only when
its name starts with `VITE_`. `frontend/vite.config.ts` sets no `envPrefix`, so that prefix is the
whole of what reaches the code. The API's base URL is `VITE_API_ENDPOINT`:

```bash
cd frontend
export VITE_API_ENDPOINT=https://your-api-url.com/api/v1
npm install
npm run build
```

The value is compiled into the bundle, so changing it means building again rather than restarting
anything.

| Variable | What it sets | Default |
|----------|--------------|---------|
| `VITE_API_ENDPOINT` | Base URL of the API the dashboard calls | empty, which leaves every request relative to wherever the dashboard is served from |
| `VITE_API_TIMEOUT` | Milliseconds before a call to the API is abandoned | `3000` |
| `VITE_ALLOWED_API_ORIGINS` | Comma-separated origins the `API_ENDPOINT` override below may point at, on top of the origin of `VITE_API_ENDPOINT` | empty |
| `VITE_PLAY_ENDPOINT` | Base URL of the webhook testing tool offered when a subscription is created | `https://play.hook0.com` |
| `VITE_CRISP_WEBSITE_ID` | Crisp website ID, which turns on the support chat widget | empty, and the widget is never loaded |

:::caution `API_ENDPOINT` without the prefix is something else
It is a query-string override read from the dashboard's own URL, meant for debugging, and it is
accepted only when it points at an origin already on the allowlist above. Exporting it in a shell
before `npm run build` does nothing at all: Vite drops every variable that is not prefixed, and the
build succeeds with an empty base URL, so the dashboard loads and every call it makes goes nowhere.
:::

### API Compilation

Using Rust's cargo tool with `SQLX_OFFLINE=true`:

```bash
cd api
SQLX_OFFLINE=true cargo build --release
```

The build generates an executable that serves as a web server, binding to `127.0.0.1:8080` by default.

### Configuration Notes

The API supports numerous configuration options via CLI parameters or environment variables, viewable through the help command:

```bash
./target/release/hook0-api --help
```

For HTTPS validation against the OS certificate store, use:

```bash
cargo build --release --no-default-features --features reqwest-rustls-tls-native-roots
```

### Logging Setup

Configure logging before running the API:

```bash
export RUST_LOG=info,sqlx=warn,actix_governor=warn
./target/release/hook0-api
```

### Output Worker

A separate worker component handles webhook delivery, compiled similarly to the API:

```bash
cd output-worker
SQLX_OFFLINE=true cargo build --release
./target/release/hook0-output-worker
```

Multiple workers can run concurrently to distribute processing load.

## Deployment Notes

Hook0 UI can be served by the API server or deployed separately as a static application.
