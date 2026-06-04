# 本番デプロイ手順書（Wave 2）

**バージョン**: 2.0.0  
**作成日**: 2026-05-28  
**適用**: Wave 2 Day 7 以降

---

## 前提条件チェックリスト

デプロイ前に以下を確認してください：

### コード品質
- [ ] Wave 2 実装完了（全タスク完了）
- [ ] 全テストパス（30+ テスト）
- [ ] テストカバレッジ > 80%
- [ ] コードレビュー完了
- [ ] セキュリティスキャン合格

### パフォーマンス
- [ ] 負荷テスト合格（1000ユーザー同時接続）
- [ ] p95 応答時間 < 100ms
- [ ] メモリリークなし確認
- [ ] CPU 使用率 < 70%

### インフラ準備
- [ ] 本番サーバー構築完了
- [ ] ネットワーク設定完了
- [ ] DNS 設定完了
- [ ] SSL 証明書取得

### ドキュメント
- [ ] デプロイ手順書完成
- [ ] ロールバック手順書完成
- [ ] インシデント対応マニュアル完成
- [ ] 運用ガイド完成

---

## 本番環境仕様

### サーバー要件

```
CPU:    4コア以上（推奨: 8コア）
RAM:    8GB 以上（推奨: 16GB）
Disk:   100GB SSD（推奨: 500GB）
OS:     Ubuntu 22.04 LTS
Docker: 24.0.0以上
```

### ネットワーク要件

```
インバウンド:
  - HTTP (80): Let's Encrypt challenge
  - HTTPS (443): クライアント接続
  - SSH (22): 管理用（IP制限）

アウトバウンド:
  - HTTPS (443): 外部API/監視サービス
```

### その他

```
タイムゾーン: UTC （またはサーバーのタイムゾーン）
監視: Grafana/Prometheus（推奨）
ログ集約: ELK Stack または CloudWatch（推奨）
```

---

## デプロイ手順

### Phase 1: イメージビルド（Day 6）

```bash
# リリースバージョン設定
VERSION=v2.0.0
REGISTRY=registry.example.com
IMAGE_NAME=opencode-api

# Docker イメージビルド
docker build \
  --build-arg CARGO_NET_GIT_FETCH_WITH_CLI=true \
  -t ${REGISTRY}/${IMAGE_NAME}:${VERSION} \
  -t ${REGISTRY}/${IMAGE_NAME}:latest \
  .

# イメージ検証
docker inspect ${REGISTRY}/${IMAGE_NAME}:${VERSION}

# レジストリにプッシュ
docker login ${REGISTRY}
docker push ${REGISTRY}/${IMAGE_NAME}:${VERSION}
docker push ${REGISTRY}/${IMAGE_NAME}:latest
```

### Phase 2: カナリアリリース準備（Day 7 朝）

```bash
# デプロイ用 Docker Compose ファイル作成
mkdir -p /opt/opencode/prod
cd /opt/opencode/prod

# 環境変数ファイル作成
cat > .env.production << 'EOF'
ENVIRONMENT=production
JWT_SECRET=$(openssl rand -hex 32)
RUST_LOG=info
OPENCODE__SERVER__HOST=0.0.0.0
OPENCODE__SERVER__PORT=8080
OPENCODE__SERVER__WORKERS=8
OPENCODE__DATABASE__PATH=/data/prod.db
OPENCODE__DATABASE__MAX_CONNECTIONS=20
OPENCODE__UPLOAD__MAX_SIZE_MB=100
EOF

# パーミッション設定
chmod 600 .env.production
chown root:root .env.production
```

### Phase 3: カナリアリリース実行（Day 7 14:00）

```bash
# 本番環境向け docker-compose.yml
cat > docker-compose.prod.yml << 'EOF'
version: '3.8'

services:
  opencode-api-canary:
    image: registry.example.com/opencode-api:v2.0.0
    container_name: opencode-api-canary
    restart: always
    ports:
      - "8080:8080"
    environment:
      - ENVIRONMENT=production
      - RUST_LOG=info
    env_file:
      - .env.production
    volumes:
      - ./data/uploads:/app/uploads
      - ./data/db:/data
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
    labels:
      - "opencode.version=v2.0.0"
      - "opencode.environment=production"
      - "opencode.release=canary"

  # ロードバランサー設定（nginx）
  nginx:
    image: nginx:latest
    container_name: opencode-nginx
    restart: always
    ports:
      - "443:443"
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - opencode-api-canary
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost/health"]
      interval: 30s
      timeout: 10s
      retries: 3

networks:
  default:
    driver: bridge
EOF

# Docker Compose 起動
docker-compose -f docker-compose.prod.yml up -d

# ログ確認
docker-compose -f docker-compose.prod.yml logs -f opencode-api-canary
```

### Phase 4: カナリアリリース検証（Day 7 14:30-15:30）

```bash
# 1. ヘルスチェック
curl https://api.example.com/health
# 期待: {"status":"healthy","timestamp":"..."}

# 2. 認証フロー検証
TOKEN=$(curl -X POST https://api.example.com/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"testpassword"}' \
  | jq -r '.token')

echo "Token: $TOKEN"

# 3. ファイルアップロード検証
curl -X POST https://api.example.com/api/v1/files/upload \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@/tmp/test.pdf"

# 4. メトリクス確認
# Grafana ダッシュボード: https://grafana.example.com
# - エラーレート: < 0.1%
# - p95 応答時間: < 100ms
# - メモリ使用率: < 1GB

# 5. ロード確認
ab -c 100 -n 10000 https://api.example.com/health
```

### Phase 5: トラフィック段階的増加（Day 8-9）

```bash
# Day 8: 10% → 50%
# nginx.conf のアップストリーム設定で weight を調整
upstream opencode_api {
    server opencode-api-wave1:8080 weight=50;      # Wave 1: 50%
    server opencode-api-canary:8080 weight=50;     # Wave 2: 50%
}

nginx -s reload

# Day 9: 50% → 100%
upstream opencode_api {
    server opencode-api-canary:8080 weight=100;    # Wave 2: 100%
}

nginx -s reload

# Wave 1 コンテナ停止（24時間後確認）
docker-compose -f docker-compose.wave1.yml down
```

---

## ロールバック手順

### 問題検出時の即座対応（< 5分）

```bash
# 問題検出
# - エラーレート > 1%
# - p95 応答時間 > 500ms
# - 認証フロー失敗

# 即座にロールバック
docker-compose -f docker-compose.wave1.yml up -d

# トラフィック 100% を Wave 1 へ
upstream opencode_api {
    server opencode-api-wave1:8080 weight=100;
}
nginx -s reload

# Wave 2 コンテナ停止
docker-compose -f docker-compose.prod.yml down

# インシデント報告
# - Slack: #incident-response
# - PagerDuty: アラート発火
```

### ロールバック確認

```bash
# 1. 全エンドポイント応答確認
curl https://api.example.com/health

# 2. メトリクス正常化確認
# - エラーレート < 0.1%
# - p95 < 100ms

# 3. ユーザーレポート確認
# Slack: #support チャネル確認
```

---

## 監視・アラート設定

### Prometheus メトリクス

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'opencode-api'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
```

### Grafana ダッシュボード

ダッシュボード ID: `opencode-wave2-prod`

監視対象:
- ✓ HTTP リクエストレート
- ✓ エラーレート（5XX, 4XX）
- ✓ p50/p95/p99 レスポンス時間
- ✓ メモリ使用率
- ✓ CPU 使用率
- ✓ ディスク使用率
- ✓ DB クエリ実行時間
- ✓ アップロード/ダウンロード速度

### アラート閾値

```yaml
# alert.yml
groups:
  - name: opencode_production
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.01
        for: 5m
        annotations:
          summary: "High error rate detected"

      - alert: HighResponseTime
        expr: histogram_quantile(0.95, http_request_duration_seconds) > 0.1
        for: 5m
        annotations:
          summary: "p95 response time > 100ms"

      - alert: HighMemoryUsage
        expr: process_resident_memory_bytes > 1073741824  # 1GB
        for: 10m
        annotations:
          summary: "Memory usage > 1GB"
```

---

## デプロイ後チェック（Day 10）

### 機能検証

- [ ] ユーザー登録・ログイン動作
- [ ] ファイルアップロード（小・大）
- [ ] ファイルダウンロード
- [ ] ファイル削除
- [ ] ファイル検索
- [ ] API エラーハンドリング

### パフォーマンス検証

- [ ] p95 応答時間 < 100ms
- [ ] 1000 並行ユーザー対応
- [ ] メモリリークなし（24時間監視）
- [ ] CPU 使用率 < 70%
- [ ] ディスク I/O 正常

### セキュリティ検証

- [ ] HTTPS 正常動作
- [ ] JWT トークン検証
- [ ] ファイルアクセス制御（owner チェック）
- [ ] CORS 設定正確

---

## トラブルシューティング

### Wave 2 起動失敗

```bash
# ログ確認
docker logs opencode-api-canary

# よくある原因と対策
# 1. 環境変数不正
docker exec opencode-api-canary env | grep OPENCODE

# 2. ポート競合
netstat -tlnp | grep 8080

# 3. ディスク容量不足
df -h /data
```

### DB マイグレーション失敗

```bash
# マイグレーション状態確認
docker exec opencode-api-canary \
  sqlx migrate info --database-url sqlite:///data/prod.db

# 手動実行
docker exec opencode-api-canary \
  sqlx migrate run --database-url sqlite:///data/prod.db
```

### パフォーマンス低下

```bash
# ホットスポット特定
# Grafana → Flame graph → CPU profile

# メモリプロファイル
docker exec opencode-api-canary \
  curl http://localhost:8080/debug/pprof/heap
```

---

## ドキュメント履歴

| バージョン | 日付 | 変更内容 |
|-----------|------|---------|
| 1.0.0 | 2026-05-27 | Wave 1 用テンプレート |
| 2.0.0 | 2026-05-28 | Wave 2 本番デプロイ手順書 |

---

**作成者**: Wave 2 チーム  
**最終更新**: 2026-05-28  
**次レビュー**: 2026-06-04
