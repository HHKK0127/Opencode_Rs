# Canary Release Plan - Wave 2 Day 5 Production Deployment

## 1. Executive Summary

### Deployment Strategy
段階的ロールアウト（3-Phase Canary Release）を採用し、リスクを最小化しながら本番環境へ安全にデプロイします。

### Risk Assessment
| Risk Level | Item | Mitigation |
|------------|------|------------|
| 高 | データベース互換性 | マイグレーション事前検証 + バックアップ |
| 中 | パフォーマンス劣化 | メトリクス監視 + 自動ロールバック基準 |
| 低 | 設定不備 | Pre-deployment checklist 徹底 |

### Timeline
```
Phase 1 (Internal Testing): 1-2時間
Phase 2 (Canary 50%):      2-4時間  
Phase 3 (GA 100%):         30分
Total:                     4-7時間
```

---

## 2. Pre-Deployment Checklist

### Infrastructure
- [ ] Docker Compose ファイル最新版確認
- [ ] 本番DBバックアップ完了
- [ ] ロールバック用イメージタグ作成
- [ ] サーバーリソース確認（CPU/Memory/Disk）

### Monitoring & Alerting
- [ ] Prometheus スクレイプ設定有効
- [ ] Grafana ダッシュボード確認
- [ ] Slack Webhook 動作確認
- [ ] /api/v1/metrics エンドポイント応答確認

### Security
- [ ] TLS証明書有効期限確認
- [ ] 環境変数（secrets）設定確認
- [ ] ファイアウォール設定確認

### Backup
- [ ] PostgreSQL DB バックアップ: `pg_dump -U opencode opencode > /backups/app_backup_$(date +%Y%m%d_%H%M%S).sql`
- [ ] uploads/ ディレクトリバックアップ確認
- [ ] バックアップリストア手動テスト（直近1週間以内）

---

## 3. Phase 1: Internal Testing (10% Traffic)

### Duration
**1-2時間**

### Deployment Steps

```bash
# 1. デプロイ前バックアップ
cp app.db app.db.backup.pre_canary
docker-compose -f docker-compose.prod.yml pull

# 2. 新バージョンデプロイ（10%トラフィック）
docker-compose -f docker-compose.prod.yml up -d

# 3. ヘルスチェック（最大5分待機）
for i in {1..30}; do
  curl -sf http://localhost:8080/health && echo "OK" && break
  sleep 10
done
```

### Health Check Commands

```bash
# Basic health
curl http://localhost:8080/health

# Metrics endpoint
curl http://localhost:8080/api/v1/metrics | grep http_requests_total

# Database connectivity
curl http://localhost:8080/api/v1/files | head -c 100
```

### Success Criteria
| Metric | Threshold |
|--------|-----------|
| Error Rate | < 0.5% |
| p95 Latency | < 100ms |
| Memory Usage | < 80% |
| CPU Usage | < 70% |

### Rollback Trigger
- Error Rate > 1%
- p95 Latency > 200ms
- Memory leak detected
- ヘルスチェック失敗

### Rollback Command

```bash
# Rollback to previous version
docker-compose down
git checkout <previous-tag>
docker-compose build
docker-compose up -d

# Verify
curl http://localhost:8080/health
```

---

## 4. Phase 2: Canary (50% Traffic)

### Duration
**2-4時間**

### Traffic Routing

```bash
# Assume load balancer / reverse proxy setup
# Example nginx upstream:
upstream backend {
  server new-instance:8080 weight=1;  # 50%
  server old-instance:8080 weight=1;  # 50%
}
```

Alternatively, for single-instance canary:

```bash
# 1. Scale to 2 instances
docker-compose up -d --scale app=2

# 2. Route 50% to new container
# (Manual or via load balancer)
```

### Monitoring during Canary

```bash
# Monitor error rate
while true; do
  curl http://localhost:8080/api/v1/metrics | grep -E "requests_total|error" 
  sleep 30
done

# Check both instances
docker-compose ps app
docker-compose logs --tail=50 app
```

### Success Criteria
| Metric | Threshold |
|--------|-----------|
| Error Rate | < 0.1% |
| p95 Latency | < 100ms |
| Database OK | No locks |
| Memory Stable | < 80% |

### Decision Point
If all metrics good for 30+ minutes:
- ✅ Proceed to Phase 3
- ❌ Increase monitoring, investigate issues
- 🔄 If issues persist > 1 hour, execute rollback

---

## 5. Phase 3: General Availability (100% Traffic)

### Duration
**30分**

### Deployment Steps

```bash
# 1. Stop old instances (if dual-deploy used)
docker-compose stop old-instance

# 2. Scale to 1 instance (new version)
docker-compose up -d --scale app=1

# 3. Verify all traffic on new version
for i in {1..10}; do
  curl http://localhost:8080/health
  sleep 5
done

# 4. Final health check
curl http://localhost:8080/api/v1/metrics | grep http_requests_total
```

### Success Criteria
- ✅ All traffic on new version
- ✅ No 5xx errors
- ✅ Latency normal
- ✅ Database healthy

### Post-Deployment

```bash
# Document completion
echo "[$(date)] Deployment complete - Phase 3 GA" >> deploy.log

# Update DNS / load balancer if needed
# Cleanup old images
docker image prune -a
```

---

## 6. Rollback Procedures

### Phase 1 Rollback (< 10 min)

```bash
# Immediate stop
docker-compose kill app

# Restore from backup
docker-compose down
cp app.db.backup.pre_canary app.db

# Start old version
git checkout <previous-tag>
docker-compose build
docker-compose up -d

# Verify
curl http://localhost:8080/health
```

### Phase 2 Rollback (< 15 min)

```bash
# 1. Remove all traffic from new version
docker-compose stop app-new

# 2. Ensure old version still running
docker-compose start app-old

# 3. Verify
curl http://localhost:8080/health
```

### Phase 3 Rollback (Emergency Only)

```bash
# Only if critical issues detected post-GA
# 1. Backup current state (for forensics)
pg_dump -U opencode opencode > /backups/app_corrupted_$(date +%Y%m%d_%H%M%S).sql

# 2. Restore previous database
psql -U opencode opencode < /backups/app_backup_pre_canary.sql

# 3. Downgrade version
docker-compose down
git checkout <previous-tag>
docker-compose build --no-cache
docker-compose up -d

# 4. Verify fully
curl http://localhost:8080/health
curl http://localhost:8080/api/v1/metrics
```

---

## 7. Monitoring & Alerting Setup

### Prometheus Configuration

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'opencode-canary'
    static_configs:
      - targets: ['app:8080']
    metrics_path: '/api/v1/metrics'
    scrape_interval: 5s
```

### Alert Rules

```yaml
# alert_rules.yml
groups:
  - name: canary_deployment
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.01
        for: 2m
        labels:
          severity: critical
      
      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 0.1
        for: 5m
        labels:
          severity: warning
```

### Slack Notifications

```bash
# Send alert to Slack
curl -X POST https://hooks.slack.com/services/YOUR/WEBHOOK/URL \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "🟡 Canary Deployment Phase 1 - Metrics within threshold",
    "blocks": [{
      "type": "section",
      "text": {
        "type": "mrkdwn",
        "text": "*Error Rate*: 0.05%\n*p95 Latency*: 85ms\n*Status*: OK ✅"
      }
    }]
  }'
```

---

## 8. Communication Plan

### Pre-Deployment (24h before)
- [ ] Notify team: "Canary deployment scheduled for [TIME]"
- [ ] Share CANARY_RELEASE_PLAN.md with stakeholders
- [ ] Confirm backup status with ops team

### Phase 1 Start (Internal Testing)
- [ ] Post in #incidents: "Phase 1 starting - internal testing"
- [ ] Monitor metrics actively
- [ ] No customer-facing announcement

### Phase 2 Start (50% Canary)
- [ ] Post in #status: "Beginning gradual rollout (50% traffic)"
- [ ] Share metrics dashboard link
- [ ] Set up escalation path (if issues found)

### Phase 3 Start (GA 100%)
- [ ] Post in #status: "New version now live (100% traffic)"
- [ ] Log deployment in changelog
- [ ] Schedule post-mortem if any issues found

### Completion
- [ ] Final status update: "Deployment complete ✅"
- [ ] Document any issues discovered
- [ ] Update DEPLOYMENT.md with lessons learned

---

## 9. Appendix: Rollback Decision Tree

```
異常検出？
  ├─ YES: エラー率 > 5%？
  │   ├─ YES → 即座にロールバック (< 5分)
  │   └─ NO: p95 > 500ms？
  │       ├─ YES → スケーリング試行、監視継続
  │       └─ NO: 続行、監視継続
  └─ NO: 順次フェーズへ進行
```

---

## 10. Success Metrics (Post-Deployment)

After 24 hours of GA operation:

- ✅ Error rate < 0.1% sustained
- ✅ p95 latency < 100ms sustained
- ✅ No data corruption detected
- ✅ Database integrity verified
- ✅ All metrics normal

If all pass: **Deployment successful** 🎉

---

**Canary Release Plan - Wave 2 Day 5 Production**  
**Last Updated**: 2026-05-30  
**Location**: docs/Operations/CANARY_RELEASE_PLAN.md
