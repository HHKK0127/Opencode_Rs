# RUNBOOK - OpenCode Production Operations

**最終更新**: 2026-06-26  
**対象**: DevOps Team, On-Call Engineers

---

## 緊急時の判断フロー

```
異常検出
    ↓
[エラー率 > 5%?]
    ├─ YES → 即座にロールバック
    └─ NO → 次へ
    ↓
[p95 > 500ms?]
    ├─ YES → スケーリング試行 (5分間隔で確認)
    └─ NO → 監視継続
    ↓
[メモリリーク?]
    ├─ YES → プロセス再起動
    └─ NO → 正常運用
```

---

## Quick Reference

### サーバー状態確認（最初の30秒）

```bash
# 1. ヘルスチェック
curl http://localhost:8080/health

# 2. メトリクス確認
curl http://localhost:8080/api/v1/metrics | grep http_requests_total

# 3. 接続数確認
curl http://localhost:8080/api/v1/metrics | grep active_connections
```

### よくある問題と対応

| 問題 | 症状 | 対応 | 時間 |
|------|------|------|------|
| メモリリーク | メモリが1時間連続増加 | `docker-compose restart app` | 1分 |
| DB ロック | クエリ応答なし | `SELECT * FROM pg_stat_activity WHERE wait_event_type = 'Lock';` 確認→ `SELECT pg_terminate_backend(pid);` | 5分 |
| ディスク満杯 | `df -h` で 100% | `find ./uploads -mtime +30 -delete` | 2分 |
| ポート競合 | bind エラー | `lsof -i :8080` で確認→ PID kill | 1分 |

---

## Incident Response

### Step 1: 異常検出 (0分)
```bash
# Slack #incidents に投稿
[時刻] API エラー率 > 5% 検出
詳細: [メトリクス値] 継続 [時間]
```

### Step 2: 初期診断 (1分以内)
```bash
# ログ確認
docker-compose logs app | tail -100 | grep ERROR

# メトリクス詳細
curl http://localhost:8080/api/v1/metrics | \
  grep -E "(5..|duration|memory|connections)"
```

### Step 3: 対応判定 (2分以内)
- **軽微** (エラー率 < 2%): 監視継続
- **中程度** (エラー率 2-5%): スケーリング or チューニング
- **深刻** (エラー率 > 5%): ロールバック

### Step 4: ロールバック (3分以内)
```bash
# 前バージョンに戻す
docker-compose down
git checkout v2.0.0
docker-compose build
docker-compose up -d

# 確認
curl http://localhost:8080/health
```

### Step 5: 事後分析 (24時間以内)
```
Post-mortem 報告内容:
- 問題の兆候
- 原因特定
- 今後の対策
```

---

## Maintenance Windows (計画停止)

### スケジュール
**毎週日曜日 深夜 01:00-02:00 (UTC)**

### 実施内容

```bash
# 1. アナウンス（24時間前）
# Slack #status に投稿

# 2. メンテナンスモード開始
# (実装済みの場合)
curl -X POST http://localhost:8080/api/v1/admin/maintenance

# 3. DBメンテナンス (PostgreSQL)
psql -U opencode -d opencode -c "VACUUM ANALYZE;"
psql -U opencode -d opencode -c "REINDEX DATABASE opencode;"

# 4. ログローテーション
logrotate /etc/logrotate.d/opencode

# 5. 監視再開
curl http://localhost:8080/health

# 6. 完了アナウンス
# Slack #status に投稿
```

---

## Performance Optimization

### パフォーマンス低下時の対応

```bash
# 1. スロークエリ確認
RUST_LOG=debug docker-compose logs app | grep "duration" | sort -t= -k2 -nr | head -20

# 2. インデックス確認
psql -U opencode -d opencode -c "SELECT indexname, indexdef FROM pg_indexes WHERE tablename = 'files';"

# 3. テーブル統計確認
psql -U opencode -d opencode -c "SELECT column_name, data_type, is_nullable FROM information_schema.columns WHERE table_name = 'files';"

# 4. 最適化実行
psql -U opencode -d opencode -c "ANALYZE;"

# 5. メトリクス確認
curl http://localhost:8080/api/v1/metrics | grep "duration"
```

### スケーリング判定基準

| メトリクス | 閾値 | 対応 |
|----------|------|------|
| p95 Latency | > 150ms | スケールアップ検討 |
| Error Rate | > 1% | スケールアウト実施 |
| Memory | > 85% | 再起動or スケール |
| CPU | > 80% | ワーカー数増加 |

---

## Escalation Path

### オンコール体制

```
Level 1: Automated Alerts
  └─ (5分以内に対応なし)
Level 2: On-Call Engineer (Slack @oncall)
  └─ (15分以内に対応なし)
Level 3: Tech Lead / Management
  └─ (1時間以内に対応なし)
Level 4: CEO Notification
```

### Slack通知設定

```bash
# #incidents チャンネル
@oncall エラー率 > 5% 検出

# #alerts チャンネル
Performance degradation p95 > 200ms

# @direct message (緊急)
Critical: Immediate rollback required
```

---

## Backup Verification

### 日次確認

```bash
# 1. 最新バックアップ確認
ls -lh /backups/opencode/*.sql | tail -5

# 2. バックアップサイズ確認
du -h /backups/opencode/ | tail -1

# 3. 整合性チェック (バックアップファイルのサイズ確認)
wc -l /backups/opencode/latest.sql
```

### 週次リストア テスト

```bash
# 1. テストDB作成
createdb opencode_test_restore
psql -U opencode -d opencode_test_restore -f /backups/opencode/latest.sql

# 2. リストア確認
psql -U opencode -d opencode_test_restore -c "SELECT COUNT(*) FROM users;"
psql -U opencode -d opencode_test_restore -c "SELECT COUNT(*) FROM files;"

# 3. クリーンアップ
dropdb opencode_test_restore
```

---

## Monitoring Checklist

### 毎日（朝9時）

```bash
# 1. Server Status
docker-compose ps

# 2. Latest Errors
docker-compose logs app --tail=100 | grep ERROR | wc -l

# 3. Disk Space
df -h / | grep -v Filesystem

# 4. Database Size
psql -U opencode -d opencode -c "SELECT pg_size_pretty(pg_database_size('opencode'));"

# 5. Metrics Summary
curl http://localhost:8080/api/v1/metrics | head -20
```

### 毎週（月曜朝）

```bash
# 1. Backup Status
ls -lh /backups/opencode/ | tail -10

# 2. Error Trends
grep ERROR app.log | wc -l

# 3. Performance Trends
# (Grafana dashboard review)

# 4. Security Audit
grep "401\|403" app.log | wc -l
```

---

## Documentation References

- **DEPLOYMENT.md** - デプロイ手順
- **[[OPERATIONS_GUIDE.md](./OPERATIONS_GUIDE.md)]** - 日常運用ガイド
- **[[CANARY_RELEASE_PLAN.md](./CANARY_RELEASE_PLAN.md)]** - 本番リリース手順
- **[MONITORING.md](./MONITORING.md)** - 監視設定

---

**RUNBOOK Complete**  
**Last Updated**: 2026-06-26  
**Location**: docs/Operations/RUNBOOK.md
