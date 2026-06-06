# Wave 5: 本番化準備詳細計画

**プロジェクト**: OpenCode Rust PoC  
**フェーズ**: Wave 5（本番化準備）  
**実装期間**: Day 16-23（8日間）  
**前提条件**: Wave 4 Day 15 パフォーマンステスト GO / Conditional GO  
**計画バージョン**: 1.0.0  
**作成日**: 2026-06-05

---

## 🎯 概要

Wave 4 で実装・検証したキャッシング・セッション管理機能を本番環境にデプロイする準備を行う。段階的アプローチでリスクを最小化し、最終的に 100% 本番移行を達成する。

### 成功基準

| # | 基準 | 測定方法 |
|---|------|---------|
| 1 | ゼロダウンタイムデプロイメント | 監視ログ |
| 2 | カナリアリリース成功（10%→50%→100%） | トラフィック分割率 |
| 3 | エラー率 < 0.1%（本番移行後） | アプリケーションログ |
| 4 | パフォーマンス基準維持（p95 < 50ms） | APM メトリクス |
| 5 | ロールバック手順検証済み | ドリル実行記録 |

---

## 📊 Wave 5 全体スケジュール

```
Day 16-17: Production Grade Optimization（2日）
Day 18-19: デプロイメント準備（2日）
Day 20-21: Canary リリース（2日）
Day 22-23: 本番完全移行（2日）
─────────────────────────────
合計: 8日間
```

---

## Phase 1: Production Grade Optimization（Day 16-17）

### Day 16: コア最適化

#### タスク 1: Redis 接続最適化（4h）
**目的**: Wave 4 での負荷テスト結果を反映し、接続プールを最適化

**実装内容**:
- [ ] 接続プールサイズ調整（デフォルト 10 → 最適値測定）
- [ ] 接続タイムアウト設定（5s → 10s）
- [ ] リトライロジック実装（指数バックオフ）
- [ ] 接続ヘルスチェック強化

**コード変更**:
```rust
// src/core/redis_client.rs
pub struct OptimizedRedisConfig {
    pub pool_size: usize,           // 負荷テスト結果に基づく
    pub connection_timeout: Duration, // 10s
    pub max_retries: u32,           // 3
    pub retry_base_delay: Duration, // 100ms
}
```

**テスト**: 3 tests
- [ ] 接続プール枯渇時の挙動
- [ ] リトライ成功パターン
- [ ] タイムアウト処理

**成功基準**: 負荷テスト時と同等以上の性能

---

#### タスク 2: 構造化ログ実装（4h）
**目的**: 本番環境での可観測性向上

**実装内容**:
- [ ] tracing crate 統合
- [ ] JSON 形式ログ出力
- [ ] 相関 ID（Correlation ID）実装
- [ ] ログレベル環境変数制御

**コード変更**:
```rust
// src/middleware/logging.rs
use tracing::{info, error, instrument};

#[instrument(skip(req), fields(correlation_id = %uuid::Uuid::new_v4()))]
pub async fn log_request(req: Request) {
    info!(
        method = %req.method(),
        path = %req.path(),
        "Request received"
    );
}
```

**テスト**: 2 tests
- [ ] JSON ログフォーマット検証
- [ ] 相関 ID 伝播確認

**成功基準**: ログが JSON 形式で出力され、相関 ID が付与される

---

### Day 17: 堅牢性向上

#### タスク 3: エラーハンドリング強化（4h）
**目的**: 本番環境での障害耐性向上

**実装内容**:
- [ ] グローバルエラーハンドラ実装
- [ ] カスタムエラーレスポンス統一
- [ ] サーキットブレーカーパターン（Redis 接続）
- [ ] フォールバック処理（キャッシュミス時）

**コード変更**:
```rust
// src/error_handling/mod.rs
pub enum AppError {
    RedisConnectionError,
    CacheMiss,
    SessionExpired,
    // ...
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        // 統一されたエラーレスポンス
    }
}
```

**テスト**: 4 tests
- [ ] Redis 接続エラー時のフォールバック
- [ ] セッション期限切れ処理
- [ ] サーキットブレーカー発動
- [ ] エラーレスポンス統一

**成功基準**: 全エラーパターンで適切な HTTP ステータスとメッセージ

---

#### タスク 4: ヘルスチェック強化（4h）
**目的**: Kubernetes 等での健全性判定精度向上

**実装内容**:
- [ ] 深層ヘルスチェック（Redis 接続確認）
- [ ] 準備完了チェック（Readiness Probe）
- [ ] 生存チェック（Liveness Probe）
- [ ] メトリクスエンドポイント（/metrics）

**コード変更**:
```rust
// src/health/mod.rs
pub async fn deep_health_check() -> HealthStatus {
    let redis_ok = check_redis_connection().await;
    let db_ok = check_database_connection().await;
    
    if redis_ok && db_ok {
        HealthStatus::Healthy
    } else {
        HealthStatus::Unhealthy
    }
}
```

**テスト**: 3 tests
- [ ] Redis 接続失敗時の Unhealthy 判定
- [ ] 正常時の Healthy 判定
- [ ] メトリクスエンドポイント

**成功基準**: /health で Redis 接続状態が正しく返却される

---

### Phase 1 成功基準

| 項目 | 基準 | テスト数 |
|------|------|---------|
| パフォーマンス維持 | 負荷テスト時と同等 | - |
| ログ出力 | JSON 形式、相関 ID 付与 | 2 |
| エラーハンドリング | 全パターン網羅 | 4 |
| ヘルスチェック | Redis 連動 | 3 |
| **合計** | | **9 tests** |

---

## Phase 2: デプロイメント準備（Day 18-19）

### Day 18: コンテナ化・CI/CD

#### タスク 5: Docker イメージ最適化（4h）
**目的**: 軽量・セキュアな本番イメージ作成

**実装内容**:
- [ ] マルチステージビルド（Rust ビルド → 実行）
- [ ] 非 root ユーザー実行
- [ ] イメージサイズ最適化（< 100MB 目標）
- [ ] セキュリティスキャン（Trivy）

**ファイル**: `Dockerfile`
```dockerfile
# ビルドステージ
FROM rust:1.75-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# 実行ステージ
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/app /usr/local/bin/
USER nonroot:nonroot
EXPOSE 8080
CMD ["app"]
```

**テスト**: 2 tests
- [ ] イメージビルド成功
- [ ] セキュリティスキャン（Critical/High 0件）

**成功基準**: イメージサイズ < 100MB、セキュリティ問題なし

---

#### タスク 6: CI/CD パイプライン構築（4h）
**目的**: 自動化されたビルド・テスト・デプロイ

**実装内容**:
- [ ] GitHub Actions workflow（.github/workflows/ci.yml）
- [ ] 自動テスト実行（unit + integration）
- [ ] Docker イメージビルド・プッシュ
- [ ] セキュリティスキャン統合

**ファイル**: `.github/workflows/ci.yml`
```yaml
name: CI/CD
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run tests
        run: cargo test
  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - name: Build Docker image
        run: docker build -t app:${{ github.sha }} .
      - name: Security scan
        uses: aquasecurity/trivy-action@master
```

**テスト**: 1 test（パイプライン実行確認）

**成功基準**: Push 時に自動でテスト→ビルド→スキャンが実行される

---

### Day 19: Kubernetes 基盤

#### タスク 7: Kubernetes manifests 作成（4h）
**目的**: 本番 Kubernetes デプロイメント定義

**実装内容**:
- [ ] Deployment manifest
- [ ] Service manifest
- [ ] ConfigMap（環境変数）
- [ ] Secret（機密情報）
- [ ] HorizontalPodAutoscaler（HPA）

**ファイル**: `k8s/`
```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: opencode-app
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: app
        image: opencode-app:latest
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        envFrom:
        - configMapRef:
            name: app-config
```

**テスト**: 2 tests（dry-run、構文チェック）

**成功基準**: `kubectl apply --dry-run=client` でエラーなし

---

#### タスク 8: シークレット管理設定（4h）
**目的**: 本番機密情報の安全な管理

**実装内容**:
- [ ] Kubernetes Secrets 作成手順
- [ ] 環境変数マッピング
- [ ] シークレットローテーション方針
- [ ] ローカル開発用 .env.example

**ファイル**: `k8s/secrets.yaml`（テンプレート）
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
type: Opaque
stringData:
  REDIS_URL: "redis://..."
  JWT_SECRET: "..."
```

**テスト**: 1 test（シークレット適用確認）

**成功基準**: シークレットが Pod に正しくマウントされる

---

### Phase 2 成功基準

| 項目 | 基準 | テスト数 |
|------|------|---------|
| Docker イメージ | < 100MB、セキュア | 2 |
| CI/CD パイプライン | 自動実行 | 1 |
| Kubernetes manifests | dry-run 成功 | 2 |
| シークレット管理 | 正しくマウント | 1 |
| **合計** | | **6 tests** |

---

## Phase 3: Canary リリース（Day 20-21）

### Day 20: カナリア基盤構築

#### タスク 9: トラフィック分割設定（6h）
**目的**: 段階的な本番移行（10% → 50% → 100%）

**実装内容**:
- [ ] Istio / NGINX Ingress Controller 設定
- [ ] VirtualService 定義（重み付きルーティング）
- [ ] ヘッダーベースルーティング（テスト用）
- [ ] 自動ロールバック条件設定

**ファイル**: `k8s/canary/`
```yaml
# virtualservice.yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: app-canary
spec:
  http:
  - match:
    - headers:
        canary:
          exact: "true"
    route:
    - destination:
        host: app-canary
      weight: 100
  - route:
    - destination:
        host: app-stable
      weight: 90
    - destination:
        host: app-canary
      weight: 10
```

**テスト**: 2 tests
- [ ] 10% トラフィックが Canary に到達
- [ ] ヘッダー指定で強制 Canary ルーティング

**成功基準**: 重み付きルーティングが機能する

---

#### タスク 10: ロールバック手順確立（2h）
**目的**: 問題発生時の迅速な復旧

**実装内容**:
- [ ] ロールバックスクリプト作成
- [ ] ワンクリックロールバック手順
- [ ] データ整合性確認手順
- [ ] 緊急連絡フロー整備

**ファイル**: `scripts/rollback.sh`
```bash
#!/bin/bash
# 即座に Stable バージョンに 100% 切り替え
kubectl patch virtualservice app-canary --type=merge -p '{"spec":{"http":[{"route":[{"destination":{"host":"app-stable"},"weight":100}]}]}}'
```

**テスト**: 1 test（ロールバックドリル）

**成功基準**: 5分以内にロールバック完了

---

### Day 21: 監視・Alerting

#### タスク 11: 監視ダッシュボード構築（4h）
**目的**: カナリア環境の可観測性確保

**実装内容**:
- [ ] Grafana ダッシュボード作成
- [ ] 主要メトリクス可視化（レイテンシ、エラー率、スループット）
- [ ] Redis メトリクス統合
- [ ] セッションメトリクス統合

**ファイル**: `monitoring/grafana-dashboard.json`

**テスト**: 1 test（ダッシュボードインポート）

**成功基準**: 主要メトリクスがリアルタイムで確認できる

---

#### タスク 12: Alerting 設定（4h）
**目的**: 異常検知・自動通知

**実装内容**:
- [ ] Prometheus Alertmanager 設定
- [ ] 重要アラート定義（エラー率 > 1%、レイテンシ > 100ms）
- [ ] Slack/PagerDuty 連携
- [ ] アラート抑制（重複防止）

**ファイル**: `monitoring/alerts.yml`
```yaml
groups:
- name: app-alerts
  rules:
  - alert: HighErrorRate
    expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.01
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "Error rate is above 1%"
```

**テスト**: 2 tests
- [ ] アラート発火テスト
- [ ] 通知到達テスト

**成功基準**: 閾値超過時にアラートが発火・通知される

---

### Phase 3 成功基準

| 項目 | 基準 | テスト数 |
|------|------|---------|
| トラフィック分割 | 10%→50%→100% 制御可能 | 2 |
| ロールバック | 5分以内完了 | 1 |
| 監視ダッシュボード | 主要メトリクス可視化 | 1 |
| Alerting | 閾値超過で通知 | 2 |
| **合計** | | **6 tests** |

---

## Phase 4: 本番完全移行（Day 22-23）

### Day 22: 最終検証・移行

#### タスク 13: 最終検証（4h）
**目的**: 本番移行前の最終確認

**実装内容**:
- [ ] 機能回帰テスト
- [ ] パフォーマンスベースライン確認
- [ ] セキュリティスキャン最終実施
- [ ] バックアップ確認

**テスト**: 全 210 tests 再実行（スモークテストとして）

**成功基準**: 全テストパス、パフォーマンス基準満たす

---

#### タスク 14: 100% トラフィック移行（4h）
**目的**: 本番環境への完全移行

**実装内容**:
- [ ] Canary → Stable 昇格
- [ ] 100% トラフィック移行
- [ ] 旧バージョン停止（バックアップ保持）
- [ ] DNS 切り替え（必要に応じて）

**手順**:
```bash
# 1. Canary を Stable に昇格
kubectl set image deployment/app-stable app=app:canary-version

# 2. 100% トラフィック確認
kubectl patch virtualservice app-canary --type=merge -p '{"spec":{"http":[{"route":[{"destination":{"host":"app-stable"},"weight":100}]}]}}'

# 3. 旧バージョンスケールダウン（保持期間 24h）
kubectl scale deployment/app-old --replicas=0
```

**成功基準**: 全トラフィックが新バージョンに流れる

---

### Day 23: 完了・報告

#### タスク 15: 旧システム停止（2h）
**目的**: リソース解放・運用コスト削減

**実装内容**:
- [ ] 旧バージョン Deployment 削除（保持期間経過後）
- [ ] 不要リソース削除
- [ ] 最終バックアップ

**成功基準**: 旧システム完全停止、データ保持確認

---

#### タスク 16: 完了報告（6h）
**目的**: プロジェクト完了の記録・知見共有

**実装内容**:
- [ ] 完了報告書作成
- [ ] 振り返り（レトロスペクティブ）実施
- [ ] 知見ドキュメント化
- [ ] 次期改善項目整理

**ファイル**: `docs/Project/COMPLETION_REPORT.md`

**内容**:
- Wave 1-5 全体振り返り
- 成功要因・課題
- パフォーマンス成果
- 次期ロードマップ提案

---

### Phase 4 成功基準

| 項目 | 基準 | テスト数 |
|------|------|---------|
| 最終検証 | 全テストパス | 210 |
| トラフィック移行 | 100% 完了 | - |
| 旧システム停止 | 完全停止 | - |
| 完了報告 | ドキュメント化 | - |
| **合計** | | **210 tests** |

---

## 📊 Wave 5 全体サマリー

| Phase | 期間 | 主要タスク | テスト数 |
|-------|------|-----------|---------|
| Phase 1 | Day 16-17 | 最適化 | 9 |
| Phase 2 | Day 18-19 | デプロイメント準備 | 6 |
| Phase 3 | Day 20-21 | Canary リリース | 6 |
| Phase 4 | Day 22-23 | 本番移行 | 210 |
| **合計** | **8日間** | **16タスク** | **231 tests** |

---

## 🎯 リスク評価

| リスク | 確率 | 影響 | 対策 |
|--------|------|------|------|
| カナリア検出遅れ | 中 | 高 | 監視強化、自動ロールバック |
| Redis 接続問題 | 低 | 高 | Phase 1 で最適化済み |
| セキュリティ脆弱性 | 低 | 高 | CI/CD で自動スキャン |
| パフォーマンス劣化 | 中 | 中 | 段階的移行で検出 |

---

## 🚀 次のステップ

1. **Wave 4 Day 15 テスト実行完了待ち**
   - GO / Conditional GO / NO-GO 判定

2. **Wave 5 Phase 1 開始**（Day 16）
   - Redis 最適化から着手

---

**Wave 5 詳細計画書作成完了** ✅
