# DEPLOYMENT_EXECUTION_CHECKLIST.md

## Wave 2 Day 5 本番デプロイ実行チェックリスト

**実行日**: 2026-05-30  
**デプロイ時刻**: 09:45-17:00 JST（予定）  
**バージョン**: v3.0.0  
**責任者**: [DevOps Lead]

---

## ⏰ タイムライン

```
09:00-09:40  ロードテスト実施
09:45        Go/No-Go判定
10:00-11:00  Phase 1（10% Internal Testing）
11:00-15:00  Phase 2（50% Canary）
15:00-17:00  Phase 3（100% General Availability）
17:00-17:30  最終確認・完了報告
```

---

## 📋 デプロイ前準備（09:45までに完了）

### 🔍 インフラ準備

- [ ] Dockerイメージビルド完了（v3.0.0）
  ```bash
  docker build -t opencode-api:v3.0.0 .
  docker images | grep v3.0.0
  ```

- [ ] イメージをレジストリにプッシュ
  ```bash
  docker tag opencode-api:v3.0.0 registry/opencode-api:v3.0.0
  docker push registry/opencode-api:v3.0.0
  ```

- [ ] Kubernetesクラスタ リソース確認
  ```bash
  kubectl get nodes
  kubectl top nodes
  # CPU: < 50%, Memory: < 60%
  ```

- [ ] ロードバランサー設定確認
  ```bash
  # LB健全性: 緑
  # SSL証明書: 有効（有効期限30日以上）
  ```

---

### 🔐 セキュリティ・認証準備

- [ ] JWT_SECRET 確認
  ```bash
  echo $JWT_SECRET | wc -c
  # 文字数: 32文字以上
  ```

- [ ] SSL/TLS証明書 有効期限確認
  ```bash
  openssl s_client -connect api.opencode.com:443 | \
    grep -A 2 "notAfter"
  # 期限: 今後30日以上
  ```

- [ ] IAM 権限確認（S3アクセスの場合）
  ```bash
  aws sts get-caller-identity
  aws s3 ls s3://opencode-prod/
  # エラーなし
  ```

- [ ] Firewall ルール確認
  ```bash
  # Port 8080: API サーバー ✅
  # Port 9090: Prometheus ✅
  # Port 3000: Grafana ✅
  ```

---

### 💾 バックアップ準備

- [ ] データベースバックアップ取得
  ```bash
  sqlite3 ./poc_test.db ".dump" > backups/db_pre_deploy_$(date +%Y%m%d_%H%M%S).sql
  ls -lh backups/ | grep -c "db_"
  # 数: 1個以上
  ```

- [ ] ファイルストレージ バックアップ確認
  ```bash
  du -sh ./uploads
  # サイズ: 記録
  tar -czf backups/uploads_pre_deploy.tar.gz ./uploads
  ```

- [ ] v2.0.0 ロールバック イメージ準備
  ```bash
  docker images | grep v2.0.0
  # STATUS: present ✅
  ```

---

### 📊 監視・ログ準備

- [ ] Prometheus スクレイプ設定確認
  ```bash
  curl http://localhost:9090/api/v1/targets
  # State: "up"
  ```

- [ ] Grafana ダッシュボード 確認
  ```bash
  # http://localhost:3000
  # Dashboard: "OpenCode API - Overview" ✅
  # Panel: 全4個 ✅
  ```

- [ ] Slack 通知チャンネル確認
  ```bash
  # #alerts-production: 存在 ✅
  # #alerts-critical: 存在 ✅
  # Webhook URL: 動作確認
  ```

- [ ] ログ集約 設定確認
  ```bash
  # ロガー: fluentd / filebeat ✅
  # 送信先: Elasticsearch / CloudWatch ✅
  ```

- [ ] Sentry または エラー追跡設定確認
  ```bash
  echo $SENTRY_DSN | grep -q sentry
  # 結果: 0 (found) ✅
  ```

---

## 🚀 Phase 1: Internal Testing（10:00-11:00）

### デプロイ実行

- [ ] Canary デプロイ スクリプト実行
  ```bash
  ./deploy/scripts/canary-deploy.sh phase1
  # 出力: "Deploying 10% traffic..."
  ```

- [ ] Deployment ロールアウト監視
  ```bash
  kubectl rollout status deployment/opencode-api --timeout=5m
  # 出力: "deployment "opencode-api" successfully rolled out"
  ```

- [ ] Pod 起動確認
  ```bash
  kubectl get pods | grep opencode-api
  # STATUS: Running（1/1）
  ```

### ヘルスチェック（10:05-10:10）

- [ ] 基本ヘルスチェック実行
  ```bash
  for i in {1..10}; do
    curl -s http://api.opencode.com/health | jq .status
    sleep 2
  done
  # 全て: "healthy" ✅
  ```

- [ ] データベース接続確認
  ```bash
  curl http://api.opencode.com/health/db
  # status: "healthy", latency_ms < 10
  ```

- [ ] メトリクスエンドポイント確認
  ```bash
  curl http://api.opencode.com/api/v1/metrics | head -20
  # 出力: Prometheus形式テキスト ✅
  ```

### 機能テスト（10:10-10:30）

- [ ] 認証テスト
  ```bash
  curl -X POST http://api.opencode.com/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d '{"username":"testuser","password":"testpassword"}'
  # status: 200, token: present ✅
  ```

- [ ] ファイルAPI テスト
  ```bash
  # 既存ファイルリスト取得
  curl -H "Authorization: Bearer $TOKEN" \
    http://api.opencode.com/api/v1/files
  # status: 200 or 401, list: present ✅
  ```

- [ ] Smoke Test スイート実行
  ```bash
  ./scripts/smoke-tests.sh
  # 出力: "All tests passed"
  ```

### メトリクス監視（10:30-11:00）

- [ ] Prometheus メトリクス監視
  ```bash
  # ダッシュボード確認（30分間）
  # http://localhost:9090 (Overview panel)
  ```

- [ ] エラー率 監視
  ```bash
  curl http://api.opencode.com/api/v1/metrics | \
    grep 'http_requests_total{status="500"}' || echo "0"
  # 値: 0 or < 5 ✅
  ```

- [ ] レイテンシ 監視
  ```bash
  curl http://api.opencode.com/api/v1/metrics | \
    grep 'http_request_duration_seconds_bucket{le="0.1"'
  # 値: 高い % ✅
  ```

### 判定（11:00）

- [ ] エラー率 確認
  ```
  ERROR_RATE < 0.5% ?
  ☑️  YES → Continue to Phase 2
  ☐ NO → Rollback & Investigate
  ```

- [ ] パフォーマンス 確認
  ```
  P95 Latency < 100ms ?
  ☑️  YES → Continue to Phase 2
  ☐ NO → Rollback & Optimize
  ```

- [ ] リソース使用率 確認
  ```
  CPU < 70%, Memory < 200MB ?
  ☑️  YES → Continue to Phase 2
  ☐ NO → Scale up & Retry
  ```

---

## 🎯 Phase 2: Canary（11:00-15:00）

### デプロイ実行

- [ ] Canary デプロイ スクリプト実行
  ```bash
  ./deploy/scripts/canary-deploy.sh phase2
  # 出力: "Deploying 50% traffic..."
  ```

- [ ] トラフィック切り替え進捗 監視
  ```bash
  watch -n 30 'curl -s http://api.opencode.com/api/v1/metrics | \
    grep http_requests_total | wc -l'
  # 増加傾向 ✅
  ```

### 拡張テスト（11:00-11:30）

- [ ] 統合テスト実行
  ```bash
  ./scripts/integration-tests.sh
  # 結果: "All tests passed"
  ```

- [ ] API エンドポイント 検証（複数リクエスト）
  ```bash
  for endpoint in /health /api/v1/metrics /api/v1/files; do
    curl http://api.opencode.com$endpoint > /dev/null 2>&1 && echo "✓ $endpoint" || echo "✗ $endpoint"
  done
  # 全て: ✓
  ```

### パフォーマンス監視（11:30-14:30）

継続的に以下をモニタリング（3時間）:

- [ ] エラー率 監視
  ```
  目標: < 1%
  監視方法: Dashboard / Prometheus
  確認間隔: 15分ごと
  ```

- [ ] レイテンシ 監視
  ```
  目標: p95 < 150ms
  監視方法: Dashboard
  確認間隔: 15分ごと
  ```

- [ ] スループット 監視
  ```
  目標: > 300 req/s
  監視方法: Metrics endpoint
  確認間隔: 15分ごと
  ```

- [ ] ユーザーフィードバック 収集
  ```bash
  # Slack: #opencode-deploy で報告受付
  # 問題報告: 即座に対応
  ```

### 監視ログ（サンプル）

```
11:30 - エラー率: 0.2%, P95: 45ms, スループット: 320 req/s ✅
11:45 - エラー率: 0.15%, P95: 48ms, スループット: 330 req/s ✅
12:00 - エラー率: 0.18%, P95: 52ms, スループット: 340 req/s ✅
...
14:30 - エラー率: 0.22%, P95: 50ms, スループット: 350 req/s ✅
```

### 判定（15:00）

- [ ] エラー率 確認
  ```
  ERROR_RATE < 1% ?
  ☑️  YES → Continue to Phase 3
  ☐ NO → Investigate & Wait
  ```

- [ ] ユーザー報告 確認
  ```
  Critical Issues: None ?
  ☑️  YES → Continue to Phase 3
  ☐ NO → Address issues before Phase 3
  ```

---

## 🌟 Phase 3: General Availability（15:00-17:00）

### デプロイ実行

- [ ] 本番デプロイ スクリプト実行
  ```bash
  ./deploy/scripts/canary-deploy.sh phase3
  # 出力: "Deploying 100% traffic..."
  ```

- [ ] トラフィック 100% 確認
  ```bash
  curl http://api.opencode.com/api/v1/metrics | \
    grep 'http_requests_total' | tail -5
  # 全トラフィック新バージョン処理 ✅
  ```

### 最終監視（15:00-15:30）

- [ ] エラー率 最終確認
  ```
  ERROR_RATE < 0.1% ?
  ☑️  YES → Success
  ☐ NO → Investigate
  ```

- [ ] パフォーマンス 最終確認
  ```
  P95 < 100ms, スループット > 500 req/s?
  ☑️  YES → Success
  ☐ NO → Investigate
  ```

- [ ] メモリ安定性 確認
  ```
  メモリ増加なし、スワップなし?
  ☑️  YES → Success
  ☐ NO → Monitor closely
  ```

### ドキュメント更新（15:30-16:00）

- [ ] DEPLOYMENT_HISTORY.md 更新
  ```bash
  echo "2026-05-30 15:00 - v3.0.0 deployed (100% traffic)" >> DEPLOYMENT_HISTORY.md
  ```

- [ ] VERSION ファイル更新
  ```bash
  echo "3.0.0" > VERSION
  git add VERSION DEPLOYMENT_HISTORY.md
  ```

- [ ] Runbook 最新版 確認
  ```bash
  # OPERATIONS_GUIDE.md: 最新 ✅
  # MONITORING.md: 最新 ✅
  # CANARY_RELEASE_PLAN.md: 実行完了記録 ✅
  ```

---

## 📊 デプロイ完了確認（16:00-17:00）

### 最終チェック

- [ ] 本番環境 接続確認
  ```bash
  curl -I https://api.opencode.com/health
  # HTTP/2 200 ✅
  ```

- [ ] Grafana ダッシュボード 全緑確認
  ```
  - Request Rate: 正常
  - Error Rate: < 0.1%
  - Latency: p95 < 100ms
  - Resource: 正常
  ```

- [ ] ログ集約 動作確認
  ```bash
  # CloudWatch / Elasticsearch で logs 確認
  # 最新エントリ: 5分以内 ✅
  ```

### 完了報告

- [ ] Slack通知送信
  ```
  #alerts-production に投稿:
  🎉 v3.0.0 本番デプロイ完了
  時刻: 2026-05-30 15:30 JST
  ステータス: ✅ 成功
  p95 Latency: XXms
  Error Rate: X.X%
  ```

- [ ] ステークホルダーに報告
  ```
  To: 開発チーム, PM, 経営層
  Subject: [報告] OpenCode API v3.0.0 本番稼働開始

  本文:
  - デプロイ完了日時
  - パフォーマンス指標
  - 今後の監視体制
  - 問い合わせ先
  ```

- [ ] チーム通知（全員確認）
  ```bash
  # Slack #general:
  @channel v3.0.0本番稼働開始。
  監視ダッシュボード: http://localhost:3000
  問題報告: #opencode-issues
  ```

---

## 🚨 ロールバック手順（緊急時）

### 即座のロールバック

```bash
# Step 1: トラフィック停止（30秒以内）
kubectl patch service opencode-api -p '{"spec":{"selector":{"version":"v2.0.0"}}}'

# Step 2: v2.0.0 起動確認
kubectl set image deployment/opencode-api \
  opencode-api=registry/opencode-api:v2.0.0

# Step 3: ヘルスチェック
curl http://api.opencode.com/health
# status: healthy ✅

# Step 4: インシデント記録
echo "$(date): Rollback from v3.0.0 to v2.0.0 due to [REASON]" >> incidents/$(date +%Y%m%d)_rollback.log
```

### ロールバック判定基準

```
即座ロールバック対象:
  ❌ エラー率 > 5% (5分継続)
  ❌ p95 > 500ms (5分継続)
  ❌ メモリリーク検出
  ❌ データベース接続エラー (大量)
  ❌ 認証システム停止

24時間監視後判定:
  ⚠️  エラー率 > 2%
  ⚠️  パフォーマンス低下傾向
```

---

## 📋 デプロイ記録フォーム

```
【本番デプロイ実行記録】

実行日: 2026-05-30
実行者: [名前]
責任者: [DevOps Lead]

【ロードテスト結果】
p95 Latency: _____ ms
Error Rate: _____%
Peak Throughput: _____ req/s
判定: ☐ GO ☐ NO-GO

【Phase 1 (10%)】
開始時刻: _____ JST
終了時刻: _____ JST
エラー率: _____%
判定: ☐ PASS ☐ FAIL

【Phase 2 (50%)】
開始時刻: _____ JST
終了時刻: _____ JST
エラー率: _____%
ユーザー報告: ☐ なし ☐ あり（内容: _______）
判定: ☐ PASS ☐ FAIL

【Phase 3 (100%)】
開始時刻: _____ JST
終了時刻: _____ JST
最終確認: ☐ 完了
メモリ: ______ MB
CPU: _____%

【デプロイ完了】
時刻: _____ JST
ステータス: ☐ 成功 ☐ ロールバック
所要時間: _____ 分
```

---

## 参考資料

- [CANARY_RELEASE_PLAN.md](./CANARY_RELEASE_PLAN.md) — 詳細手順書
- [LOAD_TEST_PLAN.md](./LOAD_TEST_PLAN.md) — ロードテスト計画
- [OPERATIONS_GUIDE.md](./OPERATIONS_GUIDE.md) — 運用ガイド
- [MONITORING.md](./docs/MONITORING.md) — 監視設定

---

**本番デプロイチェックリスト完成！** ✅

実行日: 2026-05-30  
Go/No-Go判定: 09:45 JST  
デプロイ完了: 17:00 JST（予定）
