# Wave 4 Day 15 パフォーマンステスト環境セットアップ

**作成日**: 2026-06-05  
**実施予定日**: 2026-06-06  
**所要時間**: 約 30 分

---

## 1. システム要件

### ハードウェア
| コンポーネント | 要件 |
|-------------|------|
| CPU | 4 cores 以上 |
| メモリ | 8GB 以上 |
| ストレージ | 10GB 以上の空き容量 |
| ネットワーク | 1Gbps 以上 |

### ソフトウェア
| ツール | バージョン | 用途 |
|--------|-----------|------|
| **k6** | v0.45.0+ | 負荷テストツール |
| **Redis** | 7.0+ | キャッシングサーバー |
| **Rust** | 1.75+ | アプリケーションビルド |
| **Docker** (オプション) | 20.10+ | コンテナ実行 |

---

## 2. Redis セットアップ

### 2.1 Docker での起動（推奨）

```bash
# Redis コンテナ起動
docker run -d \
  --name opencode-redis \
  --port 6379:6379 \
  -e REDIS_PASSWORD=test_password \
  redis:7-alpine \
  redis-server --requirepass test_password

# 確認
docker ps | grep opencode-redis
# expected: opencode-redis が起動中

# 接続確認
redis-cli -h localhost -p 6379 -a test_password ping
# expected: PONG
```

### 2.2 ローカル Redis インストール（macOS）

```bash
# Homebrew でインストール
brew install redis

# バージョン確認
redis-server --version
# expected: Redis server v=7.0.x

# 起動
redis-server --port 6379 --requirepass test_password &

# 接続確認
redis-cli -p 6379 -a test_password ping
# expected: PONG
```

### 2.3 ローカル Redis インストール（Linux）

```bash
# Ubuntu/Debian
sudo apt-get install redis-server

# 設定ファイル編集
sudo nano /etc/redis/redis.conf
# requirepass test_password を設定

# 起動
sudo systemctl start redis-server

# 確認
redis-cli -a test_password ping
# expected: PONG
```

### 2.4 Redis メモリ設定

```bash
# Redis CLI で設定確認
redis-cli -a test_password CONFIG GET maxmemory
# expected: (integer) 536870912 (512MB)

# 必要に応じて メモリ制限を設定
redis-cli -a test_password CONFIG SET maxmemory 512mb
redis-cli -a test_password CONFIG SET maxmemory-policy allkeys-lru
```

---

## 3. アプリケーション ビルド・起動

### 3.1 環境変数設定

```bash
# .env ファイル作成（または .env.local）
cat > .env.local << 'EOF'
# Redis 接続
REDIS_URL=redis://:test_password@localhost:6379
REDIS_POOL_SIZE=50

# キャッシング
CACHE_TTL=300              # 5分
SESSION_TTL=86400          # 24時間

# アプリケーション
ENVIRONMENT=production     # ロードテスト環境
JWT_SECRET=test_secret_key_for_testing
DATABASE_URL=sqlite:./test_load.db
RUST_LOG=warn              # ログレベル（本番相当）
EOF

# 確認
cat .env.local
```

### 3.2 リリースビルド

```bash
# 依存関係の更新（初回のみ）
cargo update

# リリースビルド（最適化版）
cargo build --release
# 所要時間: 約 60-90秒
# 出力: ./target/release/opencode_poc

# ビルド確認
ls -lh target/release/opencode_poc
# expected: ファイルサイズ 8-10MB
```

### 3.3 テスト用データベース準備

```bash
# テスト用の新しいデータベースを作成
rm -f test_load.db  # 既存ファイルをクリア

# アプリケーション起動時に自動初期化されます
# ユーザー表、ファイル表が自動作成されます
```

### 3.4 サーバー起動

```bash
# 別のターミナルで実行
source .env.local
./target/release/opencode_poc
# expected output:
# Server listening on 127.0.0.1:8080
# Connected to Redis at localhost:6379
# Database initialized
```

---

## 4. k6 セットアップ

### 4.1 k6 インストール

#### macOS
```bash
brew install k6
k6 version
# expected: k6 v0.45.0+ (go1.20.x, linux/amd64)
```

#### Linux (Ubuntu/Debian)
```bash
sudo apt-get install k6
# または
curl https://dl.k6.io/stable/release/deb/repo/llvm-snapshot.gpg.key | sudo apt-key add -
echo "deb https://dl.k6.io/stable/release/deb any main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6
```

#### Windows
```powershell
# Chocolatey でインストール
choco install k6

# または直接ダウンロード
# https://github.com/grafana/k6/releases
```

### 4.2 k6 バージョン確認

```bash
k6 version
# expected: k6 v0.45.0+
```

---

## 5. テスト環境確認

### 5.1 ヘルスチェック スクリプト

```bash
# health-check.sh を作成
cat > deploy/scripts/health-check-load.sh << 'EOF'
#!/bin/bash

set -e

BASE_URL="http://127.0.0.1:8080"
REDIS_HOST="localhost"
REDIS_PORT="6379"
REDIS_PASS="test_password"

echo "🔍 Checking API Health..."

# 1. API ヘルスチェック
HEALTH=$(curl -s $BASE_URL/health)
if echo "$HEALTH" | grep -q "healthy"; then
    echo "✅ API health: OK"
else
    echo "❌ API health: FAILED"
    exit 1
fi

# 2. データベースチェック
DB_HEALTH=$(curl -s $BASE_URL/health/db)
if echo "$DB_HEALTH" | grep -q "healthy"; then
    echo "✅ Database: OK"
else
    echo "❌ Database: FAILED"
    exit 1
fi

# 3. Redis 接続確認
if redis-cli -h $REDIS_HOST -p $REDIS_PORT -a $REDIS_PASS ping | grep -q "PONG"; then
    echo "✅ Redis: OK"
else
    echo "❌ Redis: FAILED"
    exit 1
fi

# 4. メトリクス エンドポイント確認
METRICS=$(curl -s $BASE_URL/api/v1/metrics | head -1)
if [ ! -z "$METRICS" ]; then
    echo "✅ Metrics endpoint: OK"
else
    echo "❌ Metrics endpoint: FAILED"
    exit 1
fi

echo ""
echo "✅ All checks passed! Ready for load testing."
echo ""
echo "Test endpoints:"
echo "  API: $BASE_URL"
echo "  Redis: $REDIS_HOST:$REDIS_PORT"
echo ""
EOF

chmod +x deploy/scripts/health-check-load.sh

# 実行
./deploy/scripts/health-check-load.sh
```

### 5.2 手動確認

```bash
# 1. API ヘルスチェック
curl -X GET http://127.0.0.1:8080/health
# expected: {"status":"healthy","timestamp":"..."}

# 2. データベースチェック
curl -X GET http://127.0.0.1:8080/health/db
# expected: {"status":"healthy","database":"sqlite","latency_ms":2}

# 3. メトリクス確認
curl -X GET http://127.0.0.1:8080/api/v1/metrics | head -20
# expected: Prometheus フォーマットのメトリクス出力

# 4. Redis 接続確認
redis-cli -a test_password INFO server | grep version
# expected: redis_version:7.x.x
```

---

## 6. テストユーザー・データ準備

### 6.1 テストユーザー作成

```bash
# テストデータを作成するスクリプト
cat > scripts/setup_test_data.sh << 'EOF'
#!/bin/bash

BASE_URL="http://127.0.0.1:8080"

echo "Creating test users..."

for i in {1..10}; do
    curl -X POST $BASE_URL/api/v1/auth/register \
      -H "Content-Type: application/json" \
      -d '{
        "username": "loadtest_user_'$i'",
        "password": "valid_password_'$i'"
      }' \
      -s > /dev/null
    
    echo "✅ Created loadtest_user_$i"
done

echo ""
echo "✅ Test users ready for load testing"
EOF

chmod +x scripts/setup_test_data.sh
./scripts/setup_test_data.sh
```

### 6.2 テスト商品データ作成（オプション）

```bash
# テスト用商品データを挿入（SQLite 直接操作）
sqlite3 test_load.db << 'EOF'
INSERT INTO products (id, name, price, created_at) VALUES 
  ('1', 'Product 1', 100.00, datetime('now')),
  ('2', 'Product 2', 200.00, datetime('now')),
  ('3', 'Product 3', 300.00, datetime('now'));

SELECT COUNT(*) FROM products;
EOF
```

---

## 7. モニタリング セットアップ（オプション）

### 7.1 Prometheus インストール（オプション）

```bash
# Docker でインストール
docker run -d \
  --name prometheus \
  -p 9090:9090 \
  -v $(pwd)/prometheus.yml:/etc/prometheus/prometheus.yml \
  prom/prometheus:latest

# prometheus.yml 設定
cat > prometheus.yml << 'EOF'
global:
  scrape_interval: 15s
  scrape_timeout: 10s

scrape_configs:
  - job_name: 'opencode-api'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/api/v1/metrics'
    scrape_interval: 5s
EOF
```

### 7.2 リアルタイム監視（オプション）

```bash
# メトリクス監視スクリプト
watch -n 5 'curl -s http://127.0.0.1:8080/api/v1/metrics | \
  grep -E "http_requests_total|redis_cache_hits|active_connections"'
```

---

## 8. 環境準備チェックリスト

```bash
# 以下のコマンドをすべて実行して確認
[ ] Redis が起動中か確認
    redis-cli -a test_password ping

[ ] アプリケーションがビルドされているか確認
    ls -l target/release/opencode_poc

[ ] アプリケーションが起動中か確認
    curl http://127.0.0.1:8080/health

[ ] k6 がインストールされているか確認
    k6 version

[ ] テストスクリプトが存在するか確認
    ls -l tests/load/wave4_*.js

[ ] テストユーザーが作成されているか確認
    redis-cli -a test_password INFO stats | grep total_connections_received

[ ] ヘルスチェックがすべて OK か確認
    ./deploy/scripts/health-check-load.sh
```

---

## 9. トラブルシューティング

### Redis 接続エラー

```bash
# エラー: "Connection refused"
# 解決策:
redis-cli -a test_password ping
# Redis が起動していない場合は以下で起動:
docker restart opencode-redis
# または
redis-server --port 6379 --requirepass test_password &
```

### アプリケーション起動エラー

```bash
# エラー: "Address already in use"
# 解決策:
lsof -i :8080
# ポート 8080 を使用しているプロセスを終了:
kill -9 <PID>
```

### k6 スクリプト実行エラー

```bash
# エラー: "Failed to establish connection"
# 解決策:
1. API がヘルスチェック に応答しているか確認
   curl http://127.0.0.1:8080/health
2. Redis が接続可能か確認
   redis-cli -a test_password ping
3. 環境変数 BASE_URL を明示的に指定
   k6 run --env BASE_URL=http://127.0.0.1:8080 tests/load/wave4_cache_efficiency.js
```

---

## 10. 環境準備完了

すべてのチェックが OK の場合、環境準備は完了です。

次のステップ：**テスト実行（A）** に進みます。

```bash
# テスト実行コマンド（参考）
k6 run tests/load/wave4_cache_efficiency.js
k6 run tests/load/wave4_session_concurrent.js
k6 run tests/load/wave4_redis_integration.js
k6 run tests/load/wave4_e2e_flow.js
```

---

**作成日**: 2026-06-05  
**対象**: Wave 4 Day 15 パフォーマンステスト  
**次**: ステップ A（テスト実行）
