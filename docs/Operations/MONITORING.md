# Monitoring Setup Guide

## Prometheus Configuration

### Installation
```bash
# docker-compose.monitoring.yml
version: '3.8'
services:
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'

volumes:
  prometheus_data:
```

### Scraping Configuration
```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'opencode'
    static_configs:
      - targets: ['app:8080']
    metrics_path: '/api/v1/metrics'
    scrape_interval: 5s
```

---

## Metrics Reference

### HTTP Metrics
| Name | Type | Description |
|------|------|-------------|
| http_requests_total | Counter | Total requests by method/status |
| http_request_duration_seconds | Histogram | Request latency distribution |
| http_request_size_bytes | Histogram | Request body size |
| http_response_size_bytes | Histogram | Response body size |

### Database Metrics
| Name | Type | Description |
|------|------|-------------|
| db_query_duration_seconds | Histogram | Query execution time |
| db_connections_active | Gauge | Active connections |
| db_connections_idle | Gauge | Idle connections |

### PostgreSQL Metrics (psql確認)
| 確認項目 | SQL |
|---------|-----|
| アクティブ接続数 | `SELECT count(*) FROM pg_stat_activity WHERE state = 'active';` |
| ロック待機 | `SELECT count(*) FROM pg_stat_activity WHERE wait_event_type = 'Lock';` |
| DB サイズ | `SELECT pg_size_pretty(pg_database_size('opencode'));` |
| テーブルサイズ | `SELECT pg_size_pretty(pg_total_relation_size('files'));` |
| スロークエリ | `SELECT query, mean_exec_time FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;` |
| デッドタプル | `SELECT relname, n_dead_tup FROM pg_stat_user_tables ORDER BY n_dead_tup DESC;` |

### PostgreSQL アラート閾値
| メトリクス | 閾値 | 対応 |
|-----------|------|------|
| アクティブ接続数 | > 50 | コネクションプール見直し |
| ロック待機 | > 0 (継続3分) | ロック元クエリ特定・Kill |
| デッドタプル | > 10000 | `VACUUM ANALYZE` 実行 |
| DB サイズ | > 10GB | アーカイブ・パーティション検討 |

### System Metrics
| Name | Type | Description |
|------|------|-------------|
| process_resident_memory_bytes | Gauge | Memory usage |
| process_cpu_seconds_total | Counter | CPU time |
| process_open_fds | Gauge | Open file descriptors |

### Custom Metrics
| Name | Type | Description |
|------|------|-------------|
| file_upload_size_bytes | Histogram | Uploaded file sizes |
| file_count_total | Gauge | Total files |
| active_users_total | Gauge | Active users |

---

## Grafana Dashboard

### Dashboard JSON Structure
```json
{
  "dashboard": {
    "title": "OpenCode Overview",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [{
          "expr": "rate(http_requests_total[5m])"
        }]
      },
      {
        "title": "p95 Latency",
        "targets": [{
          "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))"
        }]
      }
    ]
  }
}
```

### Key Panels
1. **Traffic Overview**: RPS, error rate
2. **Latency**: p50, p95, p99 percentiles
3. **Database**: Query time, connections
4. **Resources**: Memory, CPU, disk
5. **Business**: File count, uploads

---

## Alert Rules

### Prometheus Alert Rules
```yaml
# alert_rules.yml
groups:
  - name: opencode
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.01
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"

      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "p95 latency above 100ms"

      - alert: HighMemoryUsage
        expr: process_resident_memory_bytes / 1024 / 1024 > 1024
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Memory usage above 1GB"

      - alert: PostgreSQLConnectionSaturation
        expr: pg_stat_activity_count > 40
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "PostgreSQL connection count above 40"

      - alert: PostgreSQLDeadlock
        expr: increase(pg_stat_database_deadlocks[5m]) > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "PostgreSQL deadlock detected"
```

---

## Slack Integration

### Alertmanager Configuration
```yaml
# alertmanager.yml
route:
  receiver: 'slack'
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h

receivers:
- name: 'slack'
  slack_configs:
  - api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'
    channel: '#alerts'
    title: '{{ range .Alerts }}{{ .Annotations.summary }}{{ end }}'
    text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'
```

### Notification Template
```
🔴 Critical: High error rate
   Error rate: {{ $value }}%
   Duration: 2 minutes

🟡 Warning: High latency
   p95: {{ $value }}ms
   Duration: 5 minutes
```

---

## Health Check Monitoring

### Endpoint Monitoring
```bash
# Simple health check script
#!/bin/bash
HEALTH_URL="http://localhost:8080/health"
METRICS_URL="http://localhost:8080/api/v1/metrics"

# Check health
if ! curl -sf $HEALTH_URL > /dev/null; then
  echo "Health check failed"
  # Send alert
fi

# Check metrics
curl -sf $METRICS_URL | grep -q "http_requests_total" || {
  echo "Metrics endpoint failed"
}
```

---

## Log Aggregation

### Promtail Configuration (for Loki)
```yaml
# promtail.yml
server:
  http_listen_port: 9080

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: opencode
    static_configs:
      - targets:
          - localhost
        labels:
          job: opencode
          __path__: /var/log/opencode/*.log
```

---

## Troubleshooting

### Metrics Not Appearing
1. Check endpoint: `curl http://app:8080/api/v1/metrics`
2. Verify Prometheus targets: http://localhost:9090/targets
3. Check scrape configuration

### High Cardinality
- Avoid high-cardinality labels (user_id, request_id)
- Use buckets for histograms
- Limit label values

### Missing Data
- Check clock synchronization
- Verify network connectivity
- Review retention settings

---

**Monitoring Setup Guide**  
**Last Updated**: 2026-05-30  
**Location**: docs/Operations/MONITORING.md
