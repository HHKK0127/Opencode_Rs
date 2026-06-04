# Wave 2 Emergency Rollback Procedures

**Document Date**: 2026-05-29  
**Status**: Production Ready  
**Last Updated**: Wave 2 Day 5 Completion

## Overview

This document defines emergency rollback procedures for the OpenCode Files API v2 canary deployment. Rollbacks are automated and manual procedures triggered when production health metrics exceed thresholds or critical failures occur.

## Rollback Triggers

### Automatic Triggers (Monitoring)
| Metric | Threshold | Action |
|--------|-----------|--------|
| Error Rate | > 1% (vs stable 0.1%) | Immediate canary halt |
| P95 Latency | > 500ms (vs stable 50ms) | Gradual stable scale-up |
| Memory Growth | > 300MB consecutive samples | Container restart |
| Database Connections | > 15/20 max | Scale stable replicas |
| Disk Usage | > 80% | Investigation required |

### Manual Triggers (On-Call Decision)
- Critical security vulnerability discovered
- Data corruption detected
- Compliance breach identified
- Major feature regression reported by users
- Cascading failures across multiple endpoints

## Quick Rollback (1-2 minutes)

### Step 1: Stop Canary Deployment
```bash
cd /path/to/opencode-poc
docker-compose -f docker-compose.prod.yml stop api-canary

# Verify stopped
docker-compose -f docker-compose.prod.yml ps api-canary
# Expected output: "Exit 137" or "Exited"
```

### Step 2: Scale Stable to 3 Replicas (Max Capacity)
```bash
docker-compose -f docker-compose.prod.yml up -d --scale api-stable=3

# Verify scaling
docker-compose -f docker-compose.prod.yml ps api-stable
# Expected output: 3 running containers (api-stable_1, api-stable_2, api-stable_3)
```

### Step 3: Verify Health Status
```bash
# Health check via curl (run 3 times to confirm)
for i in {1..3}; do
  echo "Health check $i:"
  curl -s http://localhost:8082/health | jq '.status'
  sleep 2
done

# Expected output: "healthy" all 3 times
```

### Step 4: Restore Traefik to Stable Only
```bash
# Remove canary route (if configured with dynamic labels)
docker-compose -f docker-compose.prod.yml exec traefik \
  rm /etc/traefik/dynamic/canary.yml 2>/dev/null || true

# Reload Traefik
docker-compose -f docker-compose.prod.yml restart traefik

# Verify routing (should show stable replicas only)
curl -s http://localhost:8888/api/overview | jq '.routers'
```

### Step 5: Send Incident Notification
```bash
#!/bin/bash
# Send Slack alert (requires SLACK_WEBHOOK_URL env var)

SLACK_MESSAGE='{
  "text": "🚨 *OpenCode API v2 Canary Rollback Triggered*",
  "blocks": [
    {
      "type": "section",
      "text": {
        "type": "mrkdwn",
        "text": "*Wave 2 Canary Deployment Halted*\n\n*Time*: '"$(date)"'\n*Reason*: Threshold exceeded or manual trigger\n*Action*: Reverted to stable v1.0.0 (3 replicas)\n\n*Next Steps*:\n1. Investigate root cause in logs\n2. Analyze canary metrics from 15min window\n3. Review code changes in v2.0.0\n4. Schedule post-incident review"
      }
    }
  ]
}'

curl -X POST -H 'Content-type: application/json' \
  --data "$SLACK_MESSAGE" \
  "$SLACK_WEBHOOK_URL"
```

## Validation Checklist

After executing rollback, verify:

- [ ] **Canary Container Stopped**: `docker ps` shows no `api-canary` running
- [ ] **Stable Replicas Running**: `docker ps` shows 3x `api-stable` containers healthy
- [ ] **Health Endpoints Responding**: All 3 replicas return 200 on `/health`
- [ ] **Database Connected**: `/health/db` returns `"connected": true`
- [ ] **Metrics Accessible**: `GET /metrics` returns Prometheus text format
- [ ] **API Endpoints Working**: 
  ```bash
  curl -s -X POST http://127.0.0.1:8080/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d '{"username":"testuser","password":"testpassword"}' | jq '.token' | wc -c
  # Should output token length (80+ characters)
  ```
- [ ] **File Upload Working**: Test single small file upload with auth token
- [ ] **Search Working**: `GET /api/v1/files/search?q=test` returns 200
- [ ] **Traefik Dashboard**: http://localhost:8888 shows stable services only
- [ ] **Grafana Dashboard**: OpenCode API dashboard shows stable metrics
- [ ] **Alert Notification**: Slack message delivered successfully

## Detailed Rollback Scenarios

### Scenario A: Error Rate Spike (> 1%)

**Symptom**: POST /api/v1/files/upload returns 500 errors for 5+ consecutive minutes

**Diagnosis**:
```bash
# 1. Check canary logs for panic or panic message
docker logs --tail 50 $(docker ps -q -f "name=api-canary")

# 2. Check database connectivity from canary
docker exec $(docker ps -q -f "name=api-canary") \
  curl http://localhost:8080/health/db | jq '.connected'

# 3. Check disk space (upload directory)
docker exec $(docker ps -q -f "name=api-canary") df /data/uploads
```

**Recovery**:
1. Execute **Quick Rollback** steps 1-5 above
2. Analyze canary logs for last 30 minutes:
   ```bash
   docker logs --since 30m $(docker ps -a -q -f "name=api-canary") > /tmp/canary-error.log
   ```
3. Review code changes in v2.0.0 tag:
   ```bash
   git log v1.0.0..v2.0.0 --oneline src/api/files.rs
   ```
4. Run full integration test suite against stable before re-deployment:
   ```bash
   cargo test --release -- --nocapture --test-threads=1
   ```

### Scenario B: Latency Degradation (P95 > 500ms)

**Symptom**: HTTP request latency increases from 50ms to 400ms+ over 10 minutes

**Diagnosis**:
```bash
# 1. Check canary container resource usage
docker stats --no-stream $(docker ps -q -f "name=api-canary")
# Look for: CPU > 80%, Memory > 400MB

# 2. Check database query performance (extract slow logs)
docker exec $(docker ps -q -f "name=api-canary") \
  sqlite3 /data/opencode.db "PRAGMA query_only; SELECT query_time FROM slow_query_log LIMIT 10;"

# 3. Check active connections count
curl -s http://localhost:8081/metrics | grep active_connections

# 4. Check file upload directory size
du -sh /data/uploads
```

**Recovery**:
1. If CPU/Memory high → Scale stable to 4 replicas (temporary):
   ```bash
   docker-compose -f docker-compose.prod.yml up -d --scale api-stable=4
   # Monitor for 5 minutes
   # If latency still > 500ms after 5min, proceed to full rollback
   ```
2. If database slow → Full rollback (steps 1-5)
3. Post-rollback investigation:
   - Profile canary with `perf` or sampling profiler
   - Check database index usage: `EXPLAIN QUERY PLAN SELECT ...`
   - Review any new database queries added in v2.0.0

### Scenario C: Memory Leak Detection

**Symptom**: Canary container memory increases from 100MB to 400MB+ without plateau

**Diagnosis**:
```bash
# 1. Monitor memory over time (requires monitoring setup)
docker stats --no-stream api-canary --interval 5 | awk '{print $6}'

# 2. Check for goroutine/task leaks in logs
docker logs $(docker ps -q -f "name=api-canary") | grep -i "goroutine\|task\|leak"

# 3. Get memory profile dump (requires pprof endpoint)
curl http://localhost:8081/debug/pprof/heap > /tmp/heap.prof
```

**Recovery**:
1. Immediate rollback (memory leaks are critical):
   ```bash
   docker-compose -f docker-compose.prod.yml stop api-canary
   docker-compose -f docker-compose.prod.yml up -d --scale api-stable=3
   ```
2. Post-incident: Add memory profiling to v2.0.0 before re-deployment
   ```bash
   # In Cargo.toml: [profile.release] debug = true for better profiling
   # Run with sampling profiler: perf record -F 99 ./target/release/opencode_poc
   ```

### Scenario D: Database Corruption or Connection Failure

**Symptom**: `/health/db` returns `"connected": false` or database file is corrupted

**Diagnosis**:
```bash
# 1. Check database integrity
docker exec $(docker ps -q -f "name=api-canary") \
  sqlite3 /data/opencode.db "PRAGMA integrity_check;"

# 2. Check database file size (sudden growth = problem)
ls -lh /data/opencode.db

# 3. Check database journal files
ls -la /data/*.db* | grep -E "wal|journal"
```

**Recovery**:
1. **If integrity check passes** (connection issue only):
   - Restart canary database connection pool:
   ```bash
   docker-compose -f docker-compose.prod.yml restart api-canary
   # Wait 30 seconds for reconnection
   curl http://localhost:8081/health/db
   ```

2. **If integrity check fails** (corruption):
   - Immediate full rollback (stable uses separate database)
   ```bash
   docker-compose -f docker-compose.prod.yml stop api-canary
   docker-compose -f docker-compose.prod.yml up -d --scale api-stable=3
   ```
   - Backup corrupted database:
   ```bash
   mv /data/opencode.db /data/opencode.db.corrupted.$(date +%s)
   ```
   - Restore from backup (requires point-in-time recovery setup):
   ```bash
   # If backup available:
   cp /backups/opencode.db.2026-05-29T12:00:00Z /data/opencode.db
   chmod 644 /data/opencode.db
   ```

## Post-Incident Procedures

### Immediate (Within 1 hour)
1. [ ] Stop the bleeding - Rollback complete
2. [ ] Notify stakeholders - Team lead, DevOps, Product
3. [ ] Collect logs and metrics - 30 minute window around incident
4. [ ] Create incident ticket with root cause hypothesis
5. [ ] Document timeline: detection → diagnosis → rollback → recovery

### Short-term (Within 24 hours)
1. **Root Cause Analysis**
   ```bash
   # Gather artifacts
   mkdir -p /tmp/incident-2026-05-29
   
   # Canary logs
   docker logs $(docker ps -a -q -f "name=api-canary") \
     > /tmp/incident-2026-05-29/canary-logs.txt
   
   # Prometheus metrics (30min around incident)
   # Query: rate(http_requests_total{job="opencode-api"}[5m])
   # Export as CSV from Grafana
   
   # Database slow query log
   sqlite3 /data/opencode.db "SELECT * FROM slow_query_log;" \
     > /tmp/incident-2026-05-29/slow-queries.txt
   
   # Code diff between v1 and v2
   git diff v1.0.0..v2.0.0 > /tmp/incident-2026-05-29/code-changes.patch
   ```

2. **Incident Review Meeting**
   - What happened? (Timeline)
   - Why did it happen? (Root cause)
   - What broke? (Affected systems)
   - How did we fix it? (Recovery steps)
   - How do we prevent it? (Preventive measures)

3. **Action Items**
   - Fix: Implement code changes to address root cause
   - Test: Add regression tests to prevent recurrence
   - Monitor: Add new metrics/alerts to detect earlier
   - Document: Update runbooks and knowledge base

### Long-term (Before next deployment)
1. **Code Changes to v2.0.0**
   - Implement fixes from RCA
   - Run full test suite: `cargo test --release`
   - Run integration tests: `./scripts/integration-tests.sh`
   - Performance benchmarks: `./scripts/benchmark.sh`

2. **Enhanced Testing**
   - Add chaos engineering tests (kill connections, fill disk, etc)
   - Add soak tests (run for 24+ hours under load)
   - Add memory profiling to CI/CD pipeline

3. **Monitoring Improvements**
   - Add alerting for: error rate spike, latency increase, memory growth
   - Lower alert thresholds if missed detection window
   - Add custom metrics for specific code paths (e.g., file upload success rate)

4. **Documentation Updates**
   - Update troubleshooting guide with specific solutions
   - Add new scenarios to this ROLLBACK.md
   - Create runbook for on-call team

## Automated Rollback Script (Optional)

For critical threshold violations, automated rollback can be triggered:

```bash
#!/bin/bash
# scripts/auto-rollback.sh

ERROR_RATE_THRESHOLD=1.0
P95_LATENCY_THRESHOLD=500

# Query Prometheus for current metrics
ERROR_RATE=$(curl -s 'http://localhost:9090/api/v1/query?query=rate(http_requests_total%7Bstatus%3D%225xx%22%7D%5B5m%5D)' \
  | jq '.data.result[0].value[1]' | tr -d '"' | awk '{print $1 * 100}')

P95_LATENCY=$(curl -s 'http://localhost:9090/api/v1/query?query=histogram_quantile(0.95,rate(http_request_duration_seconds_bucket%5B5m%5D))' \
  | jq '.data.result[0].value[1]' | tr -d '"' | awk '{print $1 * 1000}')

echo "Current metrics:"
echo "  Error Rate: ${ERROR_RATE}%"
echo "  P95 Latency: ${P95_LATENCY}ms"

if (( $(echo "$ERROR_RATE > $ERROR_RATE_THRESHOLD" | bc -l) )); then
  echo "ERROR RATE EXCEEDED! Triggering rollback..."
  docker-compose -f docker-compose.prod.yml stop api-canary
  docker-compose -f docker-compose.prod.yml up -d --scale api-stable=3
  exit 1
fi

if (( $(echo "$P95_LATENCY > $P95_LATENCY_THRESHOLD" | bc -l) )); then
  echo "LATENCY EXCEEDED! Investigating..."
  # Try scaling stable before full rollback
  docker-compose -f docker-compose.prod.yml up -d --scale api-stable=4
  sleep 300
  
  P95_LATENCY_CHECK=$(curl -s 'http://localhost:9090/api/v1/query?query=...' | jq '...')
  if (( $(echo "$P95_LATENCY_CHECK > $P95_LATENCY_THRESHOLD" | bc -l) )); then
    echo "LATENCY STILL HIGH! Full rollback..."
    docker-compose -f docker-compose.prod.yml stop api-canary
    exit 1
  fi
fi

echo "All metrics within threshold. Continuing deployment..."
exit 0
```

Usage:
```bash
chmod +x scripts/auto-rollback.sh
./scripts/auto-rollback.sh || echo "Rollback executed - manual verification required"
```

## Contacts & Escalation

| Role | Name | Phone | Email | Slack |
|------|------|-------|-------|-------|
| On-Call Engineer | TBD | TBD | TBD | @oncall |
| Team Lead | TBD | TBD | TBD | @lead |
| DevOps Lead | TBD | TBD | TBD | @devops |
| Incident Commander | TBD | TBD | TBD | @ic |

**Escalation Path**:
1. Threshold triggered → Page on-call engineer
2. On-call unable to resolve in 15min → Escalate to team lead
3. Major outage (> 5min down) → Page incident commander + DevOps
4. Suspected security breach → Page all + CISO

## Sign-off

- [ ] Rollback procedures tested in staging: ___________
- [ ] Team trained on manual procedures: ___________
- [ ] Automated rollback script deployed: ___________
- [ ] Incident response contacts updated: ___________
- [ ] Approved for production deployment: ___________

---

**Last Tested**: (Fill after staging validation)  
**Next Review**: 2026-06-29 (Monthly)
