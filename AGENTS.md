# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Context

### PoC Crate (opencode_poc — root `src/`)
**Proof of Concept** for migrating OpenCode (43K-line TypeScript) to Rust backend via Strangler Fig.

**Status**: Wave 5 完全完成 (2026-06-25) — **本番移行 GO ✅**
- Wave 1-5: 229/229 tests (100%) ✅
- JWT auth, File API, S3/MinIO, Redis, K8s, CI/CD, Canary
- See `README.md` for full details.

### OpenCode Desktop (opencode-core — `opencode-core/`)
**Rust reimplementation of OpenCode Desktop backend server** using OpenCode v2 API protocol.

**Status**: Wave 5 拡張 — **V2 API Phase 1 実装完了 ✅**
- OpenCode v2 API: SSE streaming, Session CRUD, Mock LLM prompt ✅
- Question/Permission endpoints ✅
- Event bus for real-time updates ✅
- Build: `cargo build` (opencode-core/) ✅
- See `README.md` for endpoint list.

## Development Commands

### Build & Compilation
```bash
# Workspace build (all crates)
cargo build

# Debug build (fast iteration, unoptimized)
cargo build -p opencode-core

# Release build (optimized)
cargo build --release

# Clean build artifacts
cargo clean
```

### Testing & Verification
```bash
# Run all workspace tests
cargo test

# Run tests in specific crate
cargo test -p opencode-core

# Run with backtrace for debugging
RUST_BACKTRACE=1 cargo test
```

### Running the Server

#### PoC Server (opencode_poc)
```bash
# Development (with config/development.toml)
cargo run

# Production (with config/production.toml)
ENVIRONMENT=production cargo run --release

# Docker container
docker-compose up -d
```

#### OpenCode Desktop Server (opencode-core)
```bash
# Development server
cargo run -p opencode-core

# With frontend (served from opencode-desktop/)
cargo run -p opencode-core

# Server at http://127.0.0.1:8080
```

### Database (PoC only)
```bash
# Database initializes automatically on first server start
# Creates: users table, files table
# Test user: testuser / testpassword (auto-created)

# Database location: ./poc_test.db (SQLite)
```

## Architecture Overview

### High-Level Structure
```sh
opencode_poc/
├── src/
│   ├── main.rs                 # Server initialization, DB setup, config loading
│   ├── config.rs               # Configuration system (TOML + env vars)
│   ├── models.rs               # Request/response DTOs
│   ├── error.rs                # Unified error handling (AppError enum)
│   │
│   ├── api/
│   │   ├── mod.rs              # Central router with /api/v1 scope
│   │   ├── auth.rs             # POST /auth/* endpoints (under /api/v1 scope)
│   │   ├── files.rs            # POST /files/upload endpoint
│   │   ├── users.rs            # GET /users, GET /users/{id} endpoints
│   │   ├── projects.rs         # GET /projects endpoint (placeholder)
│   │   └── health.rs           # /health, /health/db endpoints
│   │
│   ├── auth_middleware.rs      # JWT token verification middleware
│   ├── middleware_cors.rs      # CORS configuration (localhost:3000, Tauri)
│   ├── middleware_logging.rs   # Structured logging initialization
│   └── middleware_rate_limit.rs # Rate limiting placeholder
│
├── opencode-core/
│   ├── Cargo.toml
│   └── src/
│       ├── bin/server.rs       # Binary entrypoint
│       ├── lib.rs              # Crate root (5 modules)
│       ├── auth.rs             # BasicAuth middleware
│       ├── config.rs           # Env-based config
│       ├── models.rs           # Shared data models + V2 types
│       ├── server.rs           # OpenCodeServer struct
│       └── api/
│           ├── mod.rs          # Module declarations
│           ├── router.rs       # Central route registration
│           ├── events.rs       # EventBus + SSE streaming
│           ├── session.rs      # Session CRUD (V1+V2)
│           ├── prompt.rs       # Prompt handler + mock LLM
│           ├── question.rs     # Question endpoints
│           ├── permission.rs   # Permission endpoints
│           ├── health.rs       # Health check endpoints
│           ├── config.rs       # Config endpoints
│           ├── provider.rs     # Provider listing
│           ├── tools.rs        # Tool enumeration
│           ├── find.rs         # File/symbol search
│           └── static_files.rs # Frontend serving
│
├── opencode-desktop/
│   ├── Cargo.toml
│   ├── package.json           # Vite + React + TypeScript
│   ├── vite.config.ts
│   ├── index.html
│   └── src/
│       ├── main.rs            # Tauri binary (future)
│       ├── main.tsx           # React entrypoint
│       └── App.tsx            # Root component (assistant-ui)
│
├── config/
│   ├── development.toml        # Development settings (local dev)
│   └── production.toml         # Production settings (Docker/cloud)
│
├── deploy/
│   └── scripts/
│       ├── build.sh            # Docker image build
│       ├── up.sh               # docker-compose up
│       ├── down.sh             # docker-compose down
│       ├── logs.sh             # View logs
│       └── health-check.sh     # Health verification
│
├── Dockerfile                   # Multi-stage build (builder + runtime)
├── docker-compose.yml           # Service orchestration
├── docs/                        # 📚 Unified Documentation Hub
│   ├── INDEX.md                # Navigation index
│   ├── API/
│   │   └── API_SPECIFICATION.md
│   ├── Operations/
│   ├── Performance/
│   └── Planning/
├── .env.example                 # Environment template
└── Cargo.toml                   # Workspace root
```

### Core Patterns

#### Middleware Stack (src/main.rs)
```rust
App::new()
    .wrap(middleware_cors::configure_cors())          // CORS headers
    .wrap(actix_web::middleware::Logger::default())   // Request logging
    .wrap(auth_middleware::AuthMiddleware)            // JWT verification
    .configure(api::configure)                        // All API routes under /api/v1
```

**Middleware Ordering**: CORS → Logging → Auth (exempts /api/v1/auth/*, /health)

#### API Routing (src/api/mod.rs)
```rust
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(health::configure)
            .configure(auth::configure)
            .configure(files::configure)
            .configure(users::configure)
            .configure(projects::configure)
    );
}
```

All endpoints are automatically under `/api/v1` scope for clean version management.

#### Error Handling
All endpoints return `AppResult<HttpResponse>` which maps to:
- `200`: Success with JSON payload
- `400`: BadRequest (validation errors)
- `401`: Unauthorized (invalid credentials or missing JWT)
- `500`: Internal server error (DB failures)

#### Security Implementation
- **Passwords**: Argon2id hashing (100-200ms per operation)
- **JWT**: HS256 with 24-hour expiry
- **File uploads**: 10MB size limit, filename sanitization (alphanumeric + . - _)
- **Public endpoints**: /health, /api/v1/auth/login, /api/v1/auth/register, /api/v1/auth/refresh

## Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| actix-web | 4.5 | HTTP server framework, middleware |
| tokio | 1.35 | Async runtime with full features |
| sqlx | 0.7 | Compile-time checked SQL queries |
| argon2 | 0.5 | Password hashing algorithm |
| jsonwebtoken | 9.2 | JWT encoding/decoding |
| actix-cors | 0.7 | CORS middleware configuration |
| tracing-subscriber | 0.3 | Structured logging with filters |
| config | 0.14 | Configuration management (TOML + env vars) |
| dotenvy | 0.15 | .env file loading for env vars |
| chrono | 0.4 | Date/time handling |
| uuid | 1.6 | UUID generation for IDs |
| serde | 1.0 | Serialization/deserialization |

## Database Schema

Initialized automatically in `main.rs`:

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE files (
    id TEXT PRIMARY KEY,
    filename TEXT NOT NULL,
    size INTEGER NOT NULL,
    path TEXT NOT NULL,
    uploaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## API Endpoints (Currently Implemented)

### Authentication
- `POST /api/v1/auth/login` — Get JWT token (no auth required)
- `POST /api/v1/auth/register` — Create user (no auth required)
- `POST /api/v1/auth/refresh` — Get new token (no auth required)
- `POST /api/v1/auth/reset-password` — Reset password (stub, no auth required)

### Files (Wave 2 — Day 1-3 Complete)

#### Basic Operations (Day 1)
- `POST /api/v1/files/upload` — Upload single file (requires JWT auth)
- `GET /api/v1/files/{id}` — Get file metadata (requires JWT auth)
- `GET /api/v1/files/{id}/download` — Download file (requires JWT auth)
- `DELETE /api/v1/files/{id}` — Delete file (requires JWT auth)
- `GET /api/v1/files?page=1&per_page=20` — List files with pagination (requires JWT auth)

#### Chunked Upload (Day 2)
- `POST /api/v1/files/upload/init` — Initialize chunked upload session (requires JWT auth)
- `POST /api/v1/files/upload/chunk` — Upload file chunk (requires JWT auth)
- `POST /api/v1/files/upload/complete/{session_id}` — Complete chunked upload (requires JWT auth)
- `GET /api/v1/files/upload/progress/{session_id}` — Get upload progress (requires JWT auth)

#### Range Requests & Search (Day 3)
- `GET /api/v1/files/{id}/download` with Range header — Get partial content (206 Partial Content, requires JWT auth)
- `GET /api/v1/files/search` — Search with filters (requires JWT auth)
  - Query params: `q` (keyword), `mime_type`, `size_min`, `size_max`, `created_after`, `sort`, `order`, `page`, `per_page`
- `GET /api/v1/files/stats` — Get file statistics (requires JWT auth)

### Health
- `GET /health` — Overall health status (no auth required)
- `GET /health/db` — Database connectivity check (no auth required)

## Testing Workflow

### Manual API Testing (with curl or Postman)
```bash
# 1. Login
curl -X POST http://127.0.0.1:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"testpassword"}'

# Response includes "token" field

# 2. Use token to upload file
curl -X POST http://127.0.0.1:8080/api/v1/files/upload \
  -H "Authorization: Bearer <TOKEN>" \
  -F "file=@myfile.txt"

# 3. Health check (no auth)
curl http://127.0.0.1:8080/health
```

### Automated Testing
```bash
cargo test                      # Runs 2 middleware tests
cargo test -- --nocapture      # Show println! output
cargo test -- --test-threads=1 # Serial execution
```

## Performance Characteristics

- **Binary Size**: 8.64 MB (release build)
- **Startup Time**: ~300ms to bind port
- **API Response Time**: < 10ms (measured)
- **Password Hashing**: 100-200ms per operation (Argon2)
- **Supported Concurrency**: 1000+ concurrent connections

## Configuration System (Day 4-5)

### Config Files
Located in `config/` directory with TOML format:
- `config/development.toml` — Local development settings (4 workers, debug logging, 10MB uploads)
- `config/production.toml` — Production settings (8 workers, info logging, 50MB uploads)

### Environment Variables
Environment variables override config file settings. Use `OPENCODE__` prefix with `__` for nested keys:

```bash
ENVIRONMENT=production
JWT_SECRET=your-secret-here
OPENCODE__SERVER__HOST=0.0.0.0
OPENCODE__SERVER__PORT=8080
OPENCODE__DATABASE__MAX_CONNECTIONS=20
RUST_LOG=info
```

### Loading Configuration
```rust
// In main.rs and endpoint handlers
let settings = crate::config::Settings::new()
    .unwrap_or_else(|_| crate::config::Settings::default());

println!("Server: {}:{}", settings.server.host, settings.server.port);
```

## Deployment (Day 5)

### Docker Support

**Multi-stage Build** (Dockerfile):
1. **Builder Stage**: Compiles Rust code with dependencies
2. **Runtime Stage**: Minimal Debian image with binary and ca-certificates

**Image Size**: ~150 MB (includes Debian + binary + certs)

### Quick Start
```bash
# Build image
./deploy/scripts/build.sh latest

# Start with docker-compose
./deploy/scripts/up.sh

# Verify health
./deploy/scripts/health-check.sh

# Stop services
./deploy/scripts/down.sh
```

### Docker Compose Services
- **opencode-api**: Main API server (port 8080)
- **redis** (optional): Cache server with `--profile with-redis`

See `docs/Operations/DEPLOYMENT.md` for detailed setup, security checklist, monitoring, and production deployment guidance. See `docs/Operations/CANARY_RELEASE_PLAN.md` for staged rollout strategy and `docs/Operations/MONITORING.md` for observability setup.

## Future Implementation (Wave 2-4)

**Week 2 (Days 6-10)**: 
- Database migration management
- Integration tests
- Performance optimization
- E2E testing

**Week 3+ (Wave 2-4)**:
- Complete API Gateway with canary releases
- Redis caching layer integration
- Additional module migration (TypeScript → Rust)

See user's memory for full timeline.

## Important Notes

### Database Initialization
- Database file (`poc_test.db`) is created on first server startup
- Test user is automatically created with credentials: `testuser` / `testpassword`
- Drop the database file and restart to reset to clean state

### CORS Configuration
Allowed origins: `localhost:3000`, `localhost:5173`, `tauri://localhost` (frontend dev servers)

### Logging
Structured logging initialized at startup with `tracing-subscriber`. Set `RUST_LOG=debug` for verbose output.

### Rate Limiting
Currently a placeholder. Governor crate dependency exists but not integrated into middleware yet.

## Common Development Patterns

**Adding a new endpoint**:
1. Add request/response models to `models.rs`
2. Create module (e.g., `src/api_users.rs`) with `#[post()]` or `#[get()]` handler
3. Implement handler returning `AppResult<HttpResponse>`
4. Add `pub fn configure()` with `cfg.service(handler_name)`
5. Call `.configure(module::configure)` in `main.rs` App builder

**Adding middleware**:
1. Create `src/middleware_name.rs` implementing `Transform<S, ServiceRequest>` trait
2. Add `.wrap(middleware_name)` in App builder (order matters)
3. Public endpoints should check path prefix and call `next.call(req).await` to skip middleware
