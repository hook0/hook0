ARG RUST_VERSION=1.72

FROM rust:${RUST_VERSION} as builder-rust
WORKDIR /app
RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=api,target=api \
    --mount=type=bind,source=clients/rust,target=clients/rust \
    --mount=type=bind,source=output-worker,target=output-worker \
    --mount=type=bind,source=sentry-integration,target=sentry-integration \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cd api || exit 1
SQLX_OFFLINE=true cargo build --locked --release
cd .. || exit 1
cp ./target/release/hook0-api /
EOF

FROM debian
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
COPY self-hosted/docker/api/run.sh /
ENV DISABLE_SERVING_WEBAPP=true

CMD ["/run.sh"]
