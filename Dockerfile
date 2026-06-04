# Stage 1: Builder - Linux環境でRustコンパイル
FROM rust:latest as builder

WORKDIR /build

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY config ./config

# Build release binary for Linux
RUN cargo build --release

# Stage 2: Runtime
FROM ubuntu:24.04

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 appuser

# Copy binary from builder stage (Linux binary)
COPY --from=builder /build/target/release/opencode_poc /app/opencode_poc

# Copy configuration files
COPY config ./config

# Create data directory
RUN mkdir -p /data/uploads && chown -R appuser:appuser /app /data

# Switch to non-root user
USER appuser

EXPOSE 8080

# Environment variables
ENV ENVIRONMENT=production \
    RUST_LOG=info \
    DATABASE_PATH=/data/poc.db

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./opencode_poc"]
