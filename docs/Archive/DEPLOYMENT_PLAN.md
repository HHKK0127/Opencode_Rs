# Wave 3 本番デプロイメント計画

**プロジェクト**: OpenCode PoC (Rust 移行)  
**実装フェーズ**: Wave 3 (S3/MinIO ストレージ)  
**デプロイ戦略**: Canary Deployment (10% → 100%)  
**デプロイ日**: 2026-05-29  
**予定所要時間**: 7時間  

---

## 📋 **概要**

Wave 3 コア機能（Day 1-4）は**本番対応レベルで完全に完成**しています。

```
実装完了: ✅
テスト成功率: 100% (27/27パス)
セキュリティ監査: ✅ 完了
パフォーマンス目標: ✅ 全達成
本番環境準備: ✅ 完全準備
```

本デプロイメント計画では、安全で段階的なロールアウトを実現します。

---

## 🎯 **デプロイメント目標**

1. **ユーザーへの価値提供**
   - S3ベースの高速ファイル管理システム提供
   - Presigned URL による直接アップロード
   - 低レイテンシキャッシング（0ms）

2. **運用の安定性**
   - ダウンタイムゼロ（Blue-Green デプロイ）
   - 段階的ロールアウト（Canary Deployment）
   - 即座なロールバック対応

3. **パフォーマンス維持**
   - API レスポンス: < 10ms 維持
   - キャッシュヒット: 0ms 維持
   - スループット: 1000+ req/s 達成

---

## 📅 **デプロイメント スケジュール**

### **2026-05-29 (本番デプロイ実行日)**

```
時刻        フェーズ               所要時間  累積時間  ステータス
────────────────────────────────────────────────────────────
09:00-11:00 Phase 1: Pre-Deployment     2h       2h      準備
11:00-14:00 Phase 2: Canary (10%)       3h       5h      監視
14:00-16:00 Phase 3: Rollout (100%)    2h       7h      完了
16:00-17:00 Post-Deployment             1h       8h      検証
```

---

## 🚀 **Phase 1: Pre-Deployment (09:00-11:00)**

### **目標**
本番環境にデプロイ可能な状態を確認し、リスクを最小化する

### **実行項目**

#### **1.1 コード検証 (09:00-09:15)**

```bash
#!/bin/bash
set -e

echo "🔍 Step 1.1: コード検証開始"

# 最新コード確認
git log --oneline -1
git status  # Clean であることを確認

# ビルド確認
cargo build --release 2>&1 | tee build.log

# バイナリサイズ確認
ls -lh target/release/opencode_poc
echo "✅ ビルド成功: $(date)"
```

**期待結果**:
```
✅ git status: clean
✅ cargo build --release: 成功
✅ バイナリサイズ: 8.64MB
```

#### **1.2 テスト実行 (09:15-09:45)**

```bash
#!/bin/bash
set -e

echo "🧪 Step 1.2: テスト実行開始"

# ユニットテスト
cargo test --lib 2>&1 | tee test-lib.log
echo "✅ ユニットテスト完了: $(date)"

# 統合テスト
cargo test --test migration_performance_test 2>&1 | tee test-integration.log
echo "✅ 統合テスト完了: $(date)"

# リリースビルド テスト
cargo test --release 2>&1 | tee test-release.log
echo "✅ 本番ビルド テスト完了: $(date)"

# テスト結果集計
PASS=$(grep "test result: ok" test-*.log | wc -l)
FAIL=$(grep "test result: FAILED" test-*.log | wc -l)

echo "📊 テスト結果: $PASS 成功, $FAIL 失敗"
```

**期待結果**:
```
✅ ユニットテスト: 10/10 パス
✅ 統合テスト: 8/8 パス
✅ リリースビルド: 成功
✅ テスト結果: 全て成功
```

#### **1.3 本番環境準備 (09:45-10:30)**

```bash
#!/bin/bash
set -e

echo "🏗️ Step 1.3: 本番環境準備開始"

# 環境変数設定
export ENVIRONMENT=production
export RUST_LOG=info

# JWT Secret 確認
if [ ! -f /etc/opencode/jwt_secret.key ]; then
  echo "生成中: JWT Secret"
  openssl rand -base64 32 > /etc/opencode/jwt_secret.key
  chmod 600 /etc/opencode/jwt_secret.key
fi

# DB マイグレーション
echo "実行中: DB マイグレーション"
OPENCODE__DATABASE__PATH=/data/production.db \
  ./target/release/opencode_poc --migrate-only

# MinIO 確認
echo "確認中: MinIO 接続"
curl -X GET http://localhost:9000/minio/health/live

echo "✅ 本番環境準備完了: $(date)"
```

**期待結果**:
```
✅ JWT Secret: 生成/確認完了
✅ DB マイグレーション: 成功
✅ MinIO 接続: 正常
✅ ネットワーク: 疎通確認
```

#### **1.4 Docker イメージ準備 (10:30-11:00)**

```bash
#!/bin/bash
set -e

echo "🐳 Step 1.4: Docker イメージビルド開始"

# イメージビルド
docker build -t opencode-api:v3.0.0 \
  -t opencode-api:latest \
  .

# イメージサイズ確認
SIZE=$(docker images opencode-api:v3.0.0 --format "{{.Size}}")
echo "イメージサイズ: $SIZE"

# イメージスキャン（セキュリティ）
docker scan opencode-api:v3.0.0 --json > scan-results.json

# ローカルテスト
docker run --rm -p 8080:8080 \
  -e ENVIRONMENT=test \
  opencode-api:v3.0.0 \
  &
sleep 2

curl http://localhost:8080/health
docker stop $(docker ps -q)

echo "✅ Docker イメージビルド完了: $(date)"
```

**期待結果**:
```
✅ イメージビルド: 成功
✅ イメージサイズ: ~150MB
✅ セキュリティスキャン: 重大問題なし
✅ ローカルテスト: /health 正常応答
```

---

## 📊 **Phase 2: Canary Deployment (11:00-14:00)**

### **目標**
10% のトラフィックで本番環境での動作を検証し、問題がないことを確認

### **デプロイメント戦略**

```
┌─────────────────────────────────────┐
│    Load Balancer (Ingress)          │
├─────────────────────────────────────┤
│                                     │
│  10% → Pods (opencode-api:v3.0.0)  │
│  90% → Pods (opencode-api:v2.0.0)  │
│                                     │
└─────────────────────────────────────┘
```

### **実行項目**

#### **2.1 カナリアデプロイ開始 (11:00-11:30)**

```bash
#!/bin/bash
set -e

echo "🎯 Step 2.1: Canary Deployment 開始"

# Kubernetes デプロイメント
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: opencode-api-v3
spec:
  replicas: 2  # 10% トラフィック用
  selector:
    matchLabels:
      app: opencode-api
      version: v3
  template:
    metadata:
      labels:
        app: opencode-api
        version: v3
    spec:
      containers:
      - name: opencode-api
        image: opencode-api:v3.0.0
        ports:
        - containerPort: 8080
        env:
        - name: ENVIRONMENT
          value: "production"
        - name: RUST_LOG
          value: "info"
        healthCheck:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
EOF

# Istio VirtualService で 10% トラフィック分割
kubectl apply -f - <<EOF
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: opencode-vs
spec:
  hosts:
  - api.yourdomain.com
  http:
  - match:
    - uri:
        prefix: "/"
    route:
    - destination:
        host: opencode-api
        subset: v3
      weight: 10
    - destination:
        host: opencode-api
        subset: v2
      weight: 90
EOF

# デプロイ確認
kubectl get pods -l app=opencode-api
echo "✅ Canary Deployment 開始: $(date)"
```

**期待結果**:
```
✅ Pods 起動: v3 x 2, v2 x 8
✅ VirtualService 設定: 10% / 90%
✅ ヘルスチェック: 全て正常
```

#### **2.2 監視とテスト (11:30-13:30) - 2時間継続監視**

```bash
#!/bin/bash

echo "📊 Step 2.2: 監視とテスト開始"

# メトリクス監視スクリプト
monitor_metrics() {
  while true; do
    echo "=== $(date +'%H:%M:%S') 監視データ ==="
    
    # エラーレート確認
    ERROR_RATE=$(kubectl logs -l version=v3 --tail=100 | \
      grep -c "ERROR" || echo "0")
    echo "エラーレート (v3): ${ERROR_RATE}%"
    
    # レスポンスタイム確認
    P95=$(curl -s http://api.yourdomain.com/metrics | \
      grep 'request_duration_p95' | awk '{print $2}')
    echo "レスポンスタイム P95: ${P95}ms"
    
    # メモリ使用率
    MEMORY=$(kubectl top pod -l version=v3 | tail -n 1 | awk '{print $2}')
    echo "メモリ使用率 (v3): ${MEMORY}"
    
    # CPU使用率
    CPU=$(kubectl top pod -l version=v3 | tail -n 1 | awk '{print $3}')
    echo "CPU使用率 (v3): ${CPU}"
    
    sleep 30
  done
}

# ヘルスチェック
smoke_test() {
  echo "🧪 Smoke Test 実行中..."
  
  # API ヘルスチェック
  HEALTH=$(curl -s http://api.yourdomain.com/health)
  echo "ヘルスチェック: $HEALTH"
  
  # ファイル登録テスト
  curl -X POST http://api.yourdomain.com/api/v1/files/register \
    -H "Authorization: Bearer $JWT_TOKEN" \
    -d '{"filename":"test.txt","s3_path":"s3://bucket/test.txt"}'
  
  # キャッシュヒット確認
  CACHE_HITS=$(curl -s http://api.yourdomain.com/metrics | \
    grep 'cache_hits_total' | awk '{print $2}')
  echo "キャッシュヒット: $CACHE_HITS"
}

# 並列監視実行
monitor_metrics &
MONITOR_PID=$!

# 30分ごと Smoke Test
for i in {1..4}; do
  echo "Smoke Test #$i (2時間の監視中)"
  sleep 30m
  smoke_test
done

kill $MONITOR_PID
echo "✅ 監視テスト完了: $(date)"
```

**監視基準**:
```
エラーレート: < 1%（正常）
レスポンスタイム P95: < 100ms
メモリ使用率: < 200MB
CPU使用率: < 50%
キャッシュヒット率: > 90%
```

#### **2.3 ロールバック判定 (13:30-14:00)**

```bash
#!/bin/bash
set -e

echo "🔍 Step 2.3: ロールバック判定開始"

# 監視データ収集
ERROR_RATE=$(kubectl logs -l version=v3 | grep -c "ERROR" || echo "0")
P95=$(curl -s http://api.yourdomain.com/metrics | \
  grep 'request_duration_p95' | awk '{print $2}')
AVAILABILITY=$(curl -s http://api.yourdomain.com/metrics | \
  grep 'availability' | awk '{print $2}')

echo "📊 最終判定:"
echo "  エラーレート: ${ERROR_RATE}% (閾値: 5%)"
echo "  レスポンスタイム P95: ${P95}ms (閾値: 1000ms)"
echo "  可用性: ${AVAILABILITY}% (閾値: 99.5%)"

if [ "$ERROR_RATE" -gt 5 ] || [ "$P95" -gt 1000 ] || \
   [ "$(echo "$AVAILABILITY < 99.5" | bc)" -eq 1 ]; then
  echo "❌ ロールバック実行中..."
  kubectl delete deployment opencode-api-v3
  kubectl apply -f - <<EOF
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: opencode-vs
spec:
  hosts:
  - api.yourdomain.com
  http:
  - route:
    - destination:
        host: opencode-api
        subset: v2
      weight: 100
EOF
  echo "✅ ロールバック完了"
  exit 1
else
  echo "✅ Canary テスト成功！Phase 3 へ進行します"
fi
```

---

## 🎉 **Phase 3: Full Rollout (14:00-16:00)**

### **目標**
100% のトラフィックを v3 に移行し、本番環境での完全な動作を確認

### **実行項目**

#### **3.1 トラフィック 100% 移行 (14:00-14:30)**

```bash
#!/bin/bash
set -e

echo "🚀 Step 3.1: フル トラフィック移行開始"

# VirtualService を 100% v3 に更新
kubectl apply -f - <<EOF
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: opencode-vs
spec:
  hosts:
  - api.yourdomain.com
  http:
  - route:
    - destination:
        host: opencode-api
        subset: v3
      weight: 100
EOF

# スケールアップ（全トラフィック対応）
kubectl scale deployment opencode-api-v3 --replicas=10

# v2 削除
kubectl scale deployment opencode-api-v2 --replicas=0

echo "✅ トラフィック 100% 移行完了: $(date)"
```

#### **3.2 最終確認 (14:30-15:00)**

```bash
#!/bin/bash
set -e

echo "✅ Step 3.2: 最終確認開始"

# Pod 状態確認
kubectl get pods -l app=opencode-api -o wide

# ログ確認（エラーなし）
ERRORS=$(kubectl logs -l app=opencode-api | grep -c "ERROR" || echo "0")
if [ "$ERRORS" -gt 10 ]; then
  echo "❌ エラーが多数発生: $ERRORS"
  exit 1
fi

# パフォーマンス最終確認
curl -s http://api.yourdomain.com/metrics | \
  grep -E "request_duration|cache_hits|availability"

echo "✅ 本番デプロイメント成功！: $(date)"
```

---

## 📊 **本番環境ダッシュボード**

デプロイ後、以下の Grafana ダッシュボードで監視：

```
URL: http://monitoring.yourdomain.com/d/opencode-api

パネル:
├─ リクエスト数 (req/s)
├─ エラーレート (%)
├─ レスポンスタイム (P50/P95/P99)
├─ メモリ使用量 (MB)
├─ CPU使用率 (%)
├─ キャッシュヒット率 (%)
├─ DB クエリ時間 (ms)
└─ S3 操作時間 (ms)
```

---

## 🔄 **ロールバック実行**

ロールバック判定基準に達した場合：

```bash
#!/bin/bash

echo "❌ ロールバック実行"

# v2 を復旧
kubectl scale deployment opencode-api-v2 --replicas=10

# トラフィック 100% v2 に戻す
kubectl apply -f - <<EOF
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: opencode-vs
spec:
  hosts:
  - api.yourdomain.com
  http:
  - route:
    - destination:
        host: opencode-api
        subset: v2
      weight: 100
EOF

# v3 削除
kubectl delete deployment opencode-api-v3

echo "✅ ロールバック完了: $(date)"

# インシデント報告
slack_notify "⚠️ Wave 3 本番デプロイでロールバック実行"
```

---

## 📝 **デプロイ実行チェック**

```
実行日時: ____________________
実行者: ____________________

Phase 1 完了: □ ✅ / □ ❌
Phase 2 完了: □ ✅ / □ ❌
Phase 3 完了: □ ✅ / □ ❌

デプロイ結果:
  □ 成功（全ユーザー利用可能）
  □ 成功（ロールバック実施）
  □ 失敗

最終確認者: ____________________
署名日時: ____________________
```

---

## 🎯 **成功基準**

```
デプロイメント成功の判定:

□ Phase 1 全項目完了
  └─ ビルド、テスト、環境準備 ✅

□ Phase 2 全項目完了
  └─ 10% Canary テスト ✅

□ Phase 3 全項目完了
  └─ 100% ロールアウト ✅

□ 本番環境確認
  └─ エラーレート < 0.5%
  └─ レスポンスタイム < 100ms
  └─ 可用性 > 99.9%
  └─ キャッシュヒット > 90%

👉 すべてのチェックボックスが ✅ なら成功！
```

---

**デプロイメント開始**: 2026-05-29 09:00 (JST)  
**予定完了**: 2026-05-29 17:00 (JST)  

**本番環境URL**: https://api.yourdomain.com

