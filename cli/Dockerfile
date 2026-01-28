# Build stage
FROM rust:1.83-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /app

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock* ./

# Create dummy source to build dependencies
RUN mkdir src && \
    echo 'fn main() { println!("Dummy"); }' > src/main.rs && \
    echo 'pub fn dummy() {}' > src/lib.rs

# Build dependencies only
RUN cargo build --release && \
    rm -rf src target/release/deps/hook0*

# Copy actual source
COPY src ./src

# Build the actual binary
RUN cargo build --release --locked

# Runtime stage
FROM alpine:3.21

# Install runtime dependencies (CA certificates for HTTPS)
RUN apk add --no-cache ca-certificates

# Create non-root user
RUN addgroup -g 1000 hook0 && \
    adduser -u 1000 -G hook0 -D hook0

# Copy binary from builder
COPY --from=builder /app/target/release/hook0 /usr/local/bin/hook0

# Set ownership
RUN chown hook0:hook0 /usr/local/bin/hook0

# Switch to non-root user
USER hook0

# Default command
ENTRYPOINT ["hook0"]
CMD ["--help"]

# Labels
LABEL org.opencontainers.image.title="Hook0 CLI"
LABEL org.opencontainers.image.description="CLI for Hook0 webhooks platform"
LABEL org.opencontainers.image.vendor="Hook0"
LABEL org.opencontainers.image.source="https://gitlab.com/hook0/hook0"
LABEL org.opencontainers.image.documentation="https://docs.hook0.com/cli"
