# ── Stage 1: build ────────────────────────────────────────────────────────────
# rust:slim-bookworm (latest stable) is required:
#   - actix-web@4.13 needs rustc >= 1.88
#   - aws-sdk-s3@1.133 needs rustc >= 1.91
#
# Corporate network / SSL interception:
#   Run `cargo vendor` locally, copy vendor/ + .cargo/ to the build dir,
#   then change the build step to: cargo build --release --offline --ignore-rust-version
FROM rust:slim-bookworm AS builder

WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config ./config
COPY migrations ./migrations
COPY opencode-core ./opencode-core
COPY opencode-llm ./opencode-llm
COPY benches ./benches
COPY tests ./tests

RUN cargo build --release

# ── Stage 2: minimal runtime ──────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 1001 -s /bin/false appuser

COPY --from=builder /build/target/release/opencode_poc /app/opencode_poc
COPY config ./config

RUN mkdir -p /data/uploads && chown -R appuser:appuser /app /data

USER appuser

EXPOSE 8080

ENV ENVIRONMENT=production \
    RUST_LOG=info

HEALTHCHECK --interval=30s --timeout=10s --start-period=15s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health/ready || exit 1

CMD ["/app/opencode_poc"]
