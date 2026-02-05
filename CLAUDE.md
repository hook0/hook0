# Hook0 Project Guidelines

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Hook0 is an open-source webhook platform (Webhook As A Service) that enables SaaS applications to manage webhooks reliably.
It provides a REST API, modern Vue dashboard, event persistence, retry logic, and fine-grained subscription management.

## Workspace Structure

- `api/`: Core REST API (Rust + Actix Web)
- `output-worker/`: Webhook delivery worker (Rust)
- `frontend/`: Web dashboard (TypeScript + Vue.js 3)
- `documentation`: Documentation website (Docusaurus)
- `clients/rust/`: Rust SDK
- `clients/typescript/`: TypeScript SDK
- `tests-e2e/`: Playwright browser tests
- `tests-api-integrations/`: k6 integration tests
- `website/`: Static website
- `protobuf/`: Library for encoding/decoding internal Protocol Buffers messages (Rust)

## API

### API Commands

```bash
cargo fmt -p hook0-api                                          # Format
cargo check -p hook0-api                                        # Run compiler to check for errors
cargo clippy --all-features --all-targets -p hook0-api          # Lint
cargo test --all-features -p hook0-api                          # Unit tests
cd api/ && cargo sqlx prepare -- --all-targets --all-features   # Extract SQL queries result's structure from dev database
cd api/ && sqlx migration add -r migration_name                 # Add a new database migration called `migration_name`
```

### API Rules

- Each endpoint must have a corresponding variant in the `Action` enum in `api/src/iam.rs`
- Early returns should be avoided
- When editing SQL queries, the command to extract SQL queries result's structure from dev database must be run

## Output Worker

### Output Worker Commands

```bash
cargo fmt -p hook0-output-worker                                            # Format
cargo check -p hook0-output-worker                                          # Run compiler to check for errors
cargo clippy --all-features --all-targets -p hook0-output-worker            # Lint
cargo test --all-features -p hook0-output-worker                            # Unit tests
cd output-worker/ &&  cargo sqlx prepare -- --all-targets --all-features    # Extract SQL queries result's structure from dev database
```

### Output Worker Rules

- Early returns should be avoided
- When editing SQL queries, the command to extract SQL queries result's structure from dev database must be run

## Frontend

### Frontend Commands

```bash
cd frontend/ && npm install     # Install dependencies
cd frontend/ && npm run dev     # Start dev server
cd frontend/ && npm run build   # Production build
cd frontend/ && npm run check   # Check for TypeScript errors
cd frontend/ && npm run lint    # Lint
```

## Documentation

When adding new documentation files (`.md`) to the `documentation/` directory, always update `documentation/sidebars.js` to include the new pages in the navigation sidebar.

### Documentation Structure

- `documentation/concepts/`: Core concepts documentation
- `documentation/tutorials/`: Step-by-step tutorials
- `documentation/how-to-guides/`: Practical guides
- `documentation/reference/`: Technical reference
- `documentation/explanation/`: In-depth explanations
- `documentation/self-hosting/`: Self-hosting guides
- `documentation/hook0-cloud/`: Hook0 Cloud policies and procedures
- `documentation/resources/`: Additional resources

### Legal Documents

Legal documents (Privacy Policy, Terms, DPA, Subprocessors) are in `website/src/` and static assets like DPA PDFs are in `website/static/legal/`.
