ARG RUST_VERSION=1.88

FROM rust:${RUST_VERSION} AS builder-rust
WORKDIR /app
ARG FEATURES=""
RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=api,target=api \
    --mount=type=bind,source=clients/rust,target=clients/rust \
    --mount=type=bind,source=output-worker,target=output-worker \
    --mount=type=bind,source=sentry-integration,target=sentry-integration \
    --mount=type=cache,sharing=private,target=/app/target/ \
    --mount=type=cache,sharing=private,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cd api || exit 1
SQLX_OFFLINE=true cargo build --locked --release --no-default-features --features "${FEATURES}"
cd .. || exit 1
cp ./target/release/hook0-api /
EOF

FROM debian:12-slim
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*
USER appuser
COPY --from=builder-rust /hook0-api /
ENV DISABLE_SERVING_WEBAPP=true

CMD ["/hook0-api"]
