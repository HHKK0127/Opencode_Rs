# ── Stage 1: dependency cache ─────────────────────────────────────────────────
# Cache Cargo.toml/Cargo.lock as a separate layer so source changes don't
# invalidate the expensive dependency compilation step.
FROM rust:1.78-slim-bookworm AS deps

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests only — source excluded to maximise cache hit rate
COPY Cargo.toml Cargo.lock ./

# Create a stub main so cargo can compile dependencies without real source
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && \
    mkdir -p src && echo "" > src/lib.rs && \
    cargo build --release && \
    rm -rf src

# ── Stage 2: compile ──────────────────────────────────────────────────────────
FROM deps AS builder

# Copy real source + config
COPY src ./src
COPY migrations ./migrations
COPY config ./config

# Touch main.rs to force recompile (avoids stale artifact from stub above)
RUN touch src/main.rs && cargo build --release

# ── Stage 3: minimal runtime ──────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Only what the binary needs at runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 1001 -s /bin/false appuser

# Copy binary and config
COPY --from=builder /build/target/release/opencode_poc /app/opencode_poc
COPY config ./config

# Data directory (SQLite DB + uploads)
RUN mkdir -p /data/uploads && chown -R appuser:appuser /app /data

USER appuser

EXPOSE 8080

ENV ENVIRONMENT=production \
    RUST_LOG=info \
    DATABASE_PATH=/data/poc.db

# Use /health/ready for liveness — returns 503 if DB unavailable
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health/ready || exit 1

CMD ["/app/opencode_poc"]
