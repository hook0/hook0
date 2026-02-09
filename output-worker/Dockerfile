ARG RUST_VERSION=1.93-trixie

FROM rust:${RUST_VERSION} AS builder-rust
WORKDIR /app

RUN apt-get update && apt-get install -y protobuf-compiler

RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=api,target=api \
    --mount=type=bind,source=clients/rust,target=clients/rust \
    --mount=type=bind,source=clients/mcp,target=clients/mcp \
    --mount=type=bind,source=output-worker,target=output-worker \
    --mount=type=bind,source=sentry-integration,target=sentry-integration \
    --mount=type=bind,source=protobuf,target=protobuf \
    --mount=type=cache,sharing=private,target=/app/target/ \
    --mount=type=cache,sharing=private,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cd output-worker || exit 1
SQLX_OFFLINE=true cargo build --locked --release
cd .. || exit 1
cp ./target/release/hook0-output-worker /
EOF

FROM debian:trixie-slim
ARG UID=10001
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends adduser curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser
COPY --from=builder-rust /hook0-output-worker /

CMD ["/hook0-output-worker"]
