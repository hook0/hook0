---
title: Bare Metal
description: Manual setup of Hook0 on bare metal servers
---

# Bare Metal

This guide provides complete instructions for deploying Hook0 on bare metal servers.

## Requirements

- A PostgreSQL 15+ database (it might work with an earlier version)
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

The frontend requires the `API_ENDPOINT` environment variable set to the API's base URL:

```bash
cd frontend
export API_ENDPOINT=https://your-api-url.com
npm install
npm run build
```

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
