# Operations Guide - OpenCode Production

## Server Management

### Start Server
```bash
# Development
cargo run

# Production
docker-compose -f docker-compose.prod.yml up -d
```

### Stop Server
```bash
# Graceful shutdown
docker-compose -f docker-compose.prod.yml stop

# Force stop (emergency)
docker-compose -f docker-compose.prod.yml kill
```

### Restart
```bash
docker-compose -f docker-compose.prod.yml restart
```

### Status Check
```bash
docker-compose ps
docker-compose logs app --tail=50
```

---

## Log Management

### View Logs
```bash
# Real-time
docker-compose logs -f app

# Last 100 lines
docker-compose logs --tail=100 app

# With timestamp
docker-compose logs -f -t app
```

### Log Levels

Change via environment variable:
```bash
# .env file
RUST_LOG=info

# Levels: error, warn, info, debug, trace
RUST_LOG=debug cargo run
```

### Log Rotation
```bash
# Configure logrotate
/etc/logrotate.d/opencode
```

---

## Metrics Monitoring

### Check Metrics Endpoint
```bash
curl http://localhost:8080/api/v1/metrics
```

### Key Metrics
| Metric | Command |
|--------|---------|
| Request rate | `grep http_requests_total` |
| Latency p95 | `grep http_request_duration_seconds` |
| Memory usage | `grep process_resident_memory_bytes` |
| DB connections | `grep db_connections_active` |

### Prometheus Query Examples
```promql
# Request rate per second
rate(http_requests_total[5m])

# Error rate
rate(http_requests_total{status=~"5.."}[5m])

# 95th percentile latency
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))
```

---

## Troubleshooting

### High Latency
1. Check DB: `psql -U opencode -d opencode -c "\dt"`
2. Check indexes: `psql -U opencode -d opencode -c "SELECT indexname FROM pg_indexes WHERE tablename = 'files';"`
3. Review slow queries in logs

### Memory Issues
```bash
# Check memory usage
docker stats opencode-app-1

# Restart if needed
docker-compose restart app
```

### Database Locked
```bash
# Check locks (PostgreSQL)
psql -U opencode -d opencode -c "SELECT pid, state, wait_event_type, query FROM pg_stat_activity WHERE wait_event_type = 'Lock';"

# Kill blocking queries if needed
psql -U opencode -d opencode -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE wait_event_type = 'Lock' AND state = 'active';"

# Optimize
psql -U opencode -d opencode -c "VACUUM ANALYZE;"
```

### Disk Space Full
```bash
# Check usage
df -h

# Clean old uploads (caution)
find ./uploads -mtime +30 -delete

# Clean Docker
docker system prune -a
```

---

## Scaling Procedures

### Horizontal Scaling (Docker Swarm)
```bash
# Initialize swarm
docker swarm init

# Deploy stack
docker stack deploy -c docker-compose.prod.yml opencode

# Scale
docker service scale opencode_app=3
```

### Vertical Scaling
```yaml
# docker-compose.prod.yml
services:
  app:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
```

---

## Backup & Recovery

### Automated Backup Script
```bash
#!/bin/bash
# backup.sh
BACKUP_DIR="/backups/opencode"
DATE=$(date +%Y%m%d_%H%M%S)

# DB backup (PostgreSQL)
pg_dump -U opencode opencode > ${BACKUP_DIR}/app_${DATE}.sql

# Uploads backup
tar czf ${BACKUP_DIR}/uploads_${DATE}.tar.gz /data/uploads/

# Keep last 7 days
find ${BACKUP_DIR} -mtime +7 -delete
```

### Restore from Backup
```bash
# Stop application
docker-compose stop app

# Restore database (PostgreSQL)
psql -U opencode opencode < app_20240115_120000.sql

# Restore uploads
tar xzf uploads_20240115_120000.tar.gz -C /

# Start application
docker-compose start app
```

### Point-in-Time Recovery
```bash
# PostgreSQL WAL-based recovery (if configured)
# Restore base backup + replay WAL logs
pg_basebackup -U opencode -D /tmp/base_backup
# Apply WAL archives up to desired point in time
```

---

## Maintenance Windows

### Scheduled Maintenance
1. Announce 24h in advance
2. Set maintenance mode (if implemented)
3. Execute maintenance tasks
4. Verify functionality
5. Announce completion

### Database Maintenance
```bash
# Weekly optimization (PostgreSQL)
psql -U opencode -d opencode -c "VACUUM ANALYZE;"

# Index rebuild
psql -U opencode -d opencode -c "REINDEX DATABASE opencode;"
```

---

## Security Operations

### Certificate Renewal
```bash
# Let's Encrypt renewal
certbot renew

# Restart to pick up new cert
docker-compose restart nginx
```

### Secret Rotation
```bash
# Update .env
# Restart services
docker-compose up -d
```

### Access Log Review
```bash
# Failed login attempts
grep "401" /var/log/opencode/access.log

# Suspicious activity
grep -E "(DROP|DELETE|INSERT)" /var/log/opencode/access.log
```

---

## Emergency Procedures

### Complete Outage
1. Check infrastructure status
2. Verify database connectivity
3. Check external dependencies
4. Execute rollback if needed
5. Notify stakeholders

### Data Corruption
1. Stop application immediately
2. Assess corruption scope
3. Restore from last known good backup
4. Verify data integrity
5. Resume operations

### Security Incident
1. Isolate affected systems
2. Preserve logs
3. Assess impact
4. Execute incident response plan
5. Post-incident review

---

**Operations Guide Complete**  
Last Updated: 2026-05-30  
Location: docs/Operations/OPERATIONS_GUIDE.md
