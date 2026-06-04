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
1. Check DB: `sqlite3 app.db ".tables"`
2. Check indexes: `PRAGMA index_list(files);`
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
# Check locks
sqlite3 app.db "PRAGMA lock_status;"

# Backup and optimize
sqlite3 app.db ".backup temp.db"
sqlite3 temp.db "VACUUM;"
mv temp.db app.db
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

# DB backup
sqlite3 /data/app.db ".backup ${BACKUP_DIR}/app_${DATE}.db"

# Uploads backup
tar czf ${BACKUP_DIR}/uploads_${DATE}.tar.gz /data/uploads/

# Keep last 7 days
find ${BACKUP_DIR} -mtime +7 -delete
```

### Restore from Backup
```bash
# Stop application
docker-compose stop app

# Restore database
cp app_20240115_120000.db app.db

# Restore uploads
tar xzf uploads_20240115_120000.tar.gz -C /

# Start application
docker-compose start app
```

### Point-in-Time Recovery
```bash
# If using WAL mode
sqlite3 app.db ".recover" | sqlite3 app_recovered.db
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
# Weekly optimization
sqlite3 app.db "VACUUM;"
sqlite3 app.db "ANALYZE;"

# Index rebuild
sqlite3 app.db "REINDEX;"
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
