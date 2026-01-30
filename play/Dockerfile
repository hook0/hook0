ARG RUST_VERSION=1.93-trixie

FROM rust:${RUST_VERSION} AS builder-rust
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY play play
COPY cli cli
COPY api api
COPY clients/rust clients/rust
COPY clients/mcp clients/mcp
COPY output-worker output-worker
COPY sentry-integration sentry-integration
COPY protobuf protobuf

RUN cargo build --locked --release -p hook0-play && \
    cp ./target/release/hook0-play /

FROM debian:trixie-slim
ARG UID=10001
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends adduser curl \
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
COPY --from=builder-rust /hook0-play /

CMD ["/hook0-play"]
