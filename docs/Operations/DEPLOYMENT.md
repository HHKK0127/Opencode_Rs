# OpenCode PoC Deployment Guide

## Quick Start with Docker Compose

### Prerequisites
- Docker 20.10+
- Docker Compose 2.0+
- curl (for health checks)

### 1. Setup Environment

```bash
cp .env.example .env
# Edit .env and set JWT_SECRET to a strong value
nano .env
```

### 2. Build Docker Image

```bash
./deploy/scripts/build.sh latest
```

### 3. Start Services

```bash
./deploy/scripts/up.sh
```

Services will start in the background:
- **API**: http://localhost:8080
- **Redis** (optional): localhost:6379 with `--profile with-redis`

### 4. Verify Health

```bash
./deploy/scripts/health-check.sh
```

### 5. View Logs

```bash
./deploy/scripts/logs.sh
```

### 6. Stop Services

```bash
./deploy/scripts/down.sh
```

---

## Manual Docker Build and Run

### Build Image

```bash
docker build -t opencode-api:latest .
```

### Run Container

```bash
docker run -d \
  --name opencode-api \
  -p 8080:8080 \
  -e ENVIRONMENT=production \
  -e JWT_SECRET=your-secret-key \
  -v opencode-data:/data \
  opencode-api:latest
```

### Check Logs

```bash
docker logs -f opencode-api
```

---

## Production Deployment Checklist

### Security
- [ ] Generate strong JWT_SECRET (use `openssl rand -hex 32`)
- [ ] Use HTTPS/TLS in production (via reverse proxy)
- [ ] Set RUST_LOG=info or warn (not debug)
- [ ] Use separate production database file
- [ ] Configure firewall rules
- [ ] Run container as non-root user (configured in Dockerfile)

### Performance
- [ ] Use release build (default in Dockerfile)
- [ ] Set appropriate worker count: `OPENCODE__SERVER__WORKERS=8`
- [ ] Configure max connections: `OPENCODE__DATABASE__MAX_CONNECTIONS=20`
- [ ] Enable Redis caching: `docker-compose --profile with-redis up`

### Monitoring (Wave 2 Day 4+)
- [ ] Setup health check monitoring (endpoint: `/health`)
- [ ] Configure Prometheus scraping (endpoint: `/api/v1/metrics`)
- [ ] Setup Grafana dashboards (see [MONITORING.md](./MONITORING.md))
- [ ] Configure alerting (Slack webhook for critical alerts)

### Backup & Recovery
- [ ] Configure automated database backups (see [[OPERATIONS_GUIDE.md](./OPERATIONS_GUIDE.md)])
- [ ] Test restore procedures (weekly)
- [ ] Setup file upload backups (uploads/ directory)

---

## Metrics Endpoint

The `/api/v1/metrics` endpoint provides Prometheus-compatible metrics:

```bash
curl http://localhost:8080/api/v1/metrics
```

**Metrics Provided**:
- `http_requests_total` - Total requests by status/method
- `http_request_duration_seconds` - Request latency histogram
- `db_connections_active` - Active database connections
- `process_resident_memory_bytes` - Process memory usage
- `file_upload_size_bytes` - File upload sizes
- Other system metrics (CPU, file descriptors)

See [MONITORING.md](./MONITORING.md) for Prometheus configuration.

---

## Canary Deployment (Wave 2 Day 5+)

For production deployment with risk mitigation, follow [[CANARY_RELEASE_PLAN.md](./CANARY_RELEASE_PLAN.md)]:

### 3-Phase Rollout
1. **Phase 1**: Internal Testing (10% traffic, 1-2 hours)
2. **Phase 2**: Canary (50% traffic, 2-4 hours)
3. **Phase 3**: GA (100% traffic, 30 minutes)

Each phase includes health checks and rollback procedures.

---

## Troubleshooting

### Common Issues

**Container won't start**:
```bash
docker logs opencode-api
# Check JWT_SECRET, database permissions
```

**Health check fails**:
```bash
curl -v http://localhost:8080/health
# Check port binding, process logs
```

**High latency**:
```bash
# Check database indexes
psql -U opencode -d opencode -c "SELECT indexname, indexdef FROM pg_indexes WHERE tablename = 'files';"

# Check metrics
curl http://localhost:8080/api/v1/metrics | grep duration
```

See [[OPERATIONS_GUIDE.md](./OPERATIONS_GUIDE.md)] for more troubleshooting.

---

## Production Environment Variables

```bash
ENVIRONMENT=production
JWT_SECRET=your-secret-here
RUST_LOG=info
OPENCODE__SERVER__HOST=0.0.0.0
OPENCODE__SERVER__PORT=8080
OPENCODE__SERVER__WORKERS=8
OPENCODE__DATABASE__MAX_CONNECTIONS=20
```

See `config/production.toml` for full configuration.

---

**Last Updated**: 2026-05-30  
**Location**: docs/Operations/DEPLOYMENT.md
