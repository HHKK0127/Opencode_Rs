# Wave 3 実装ガイド — S3/MinIO統合入門

**対象**: Wave 3 開発チーム  
**期間**: 2026-06-02 ～ 2026-06-21（3週間）  
**期待成果**: Storage Trait 実装 + S3 統合 + 自動フェイルオーバー

---

## 🚀 クイックスタート（Day 1 準備）

### 事前要件
```bash
# Rust 環境確認
rustc --version
cargo --version

# 依存関係追加準備
# Cargo.toml に以下を追加予定:
# aws-sdk-s3 = "1.0"
# aws-smithy-runtime = "1.0"
# aws-config = "1.0"

# MinIO 準備
docker-compose -f docker-compose.minio.yml up -d
# MinIO Console: http://localhost:9001 (minio/minioadmin)
```

### Day 1 目標
- [ ] Storage Trait 設計完了
- [ ] Local Backend 実装（既存ファイルシステム）
- [ ] S3 Backend スケルトン実装
- [ ] テスト基盤セットアップ
- **目標**: 8テスト全パス

---

## 📋 実装チェックリスト（日別）

### Week 3: Core Implementation (Days 1-5)

#### Day 1: S3 Backend Foundation ✅
```
□ Storage Trait 設計・実装
  - store(data, metadata) → Result<StorageUrl>
  - retrieve(id) → Result<Bytes>
  - delete(id) → Result<()>
  - exists(id) → Result<bool>
  - health_check() → Result<()>

□ LocalStorageBackend 実装
  - ファイルシステムベース（既存）
  - Trait 対応化

□ S3StorageBackend スケルトン
  - aws-sdk-s3 初期化
  - 基本メソッド構造

□ MinIO Docker セットアップ
  - docker-compose.minio.yml 作成
  - バケット自動初期化

□ テスト基盤
  - storage/mod.rs + tests
  - Mock backend（テスト用）

結果: 8テスト全パス ✅
```

#### Day 2: Upload & Download Operations ✅
```
□ S3 PUT 実装
  - PUT /api/v1/files/upload-s3
  - メタデータ送信
  - エラーハンドリング

□ S3 GET 実装
  - GET /api/v1/files/{id}/download-s3
  - Range Request 対応

□ その他エンドポイント
  - HEAD /api/v1/files/{id}/exists-s3
  - DELETE /api/v1/files/{id}/delete-s3

□ エラーハンドリング
  - 404 対応
  - 500 エラー対応

結果: 12テスト + 4 API endpoint ✅
```

#### Day 3: Multipart & Optimization ✅
```
□ Multipart Upload API
  - POST /upload-s3/init → UploadId
  - POST /upload-s3/chunk → ETag
  - POST /upload-s3/complete

□ チャンク最適化
  - 部分サイズ調整（5MB推奨）
  - 並列upload対応

□ 進捗トラッキング
  - アップロード進捗管理

□ コスト最適化
  - ストレージクラス指定（STANDARD_IA）

結果: 10テスト全パス ✅
```

#### Day 4: Migration & Failover ✅
```
□ マイグレーション機能
  - Local → S3 ファイル移行
  - バッチ処理

□ 自動フェイルオーバー
  - FailoverStorageBackend 実装
  - Primary/Secondary 切り替え
  - ヘルスチェック

□ Dual-write テスト
  - 両ストレージに書き込み
  - 一貫性確認

結果: 8テスト全パス ✅
```

#### Day 5: Monitoring & Operations ✅
```
□ メトリクス追加
  - s3_upload_duration_seconds
  - s3_download_duration_seconds
  - s3_upload_size_bytes
  - s3_request_errors_total
  - storage_failover_events_total

□ ロードテスト実施
  - 100 VU, 1MB ファイル
  - p95 < 150ms 目標

□ 運用ドキュメント
  - S3 バックアップ手順
  - 障害復旧手順

結果: メトリクス統合完了 ✅
```

---

## 📚 参照ドキュメント

### 必読（実装前に）
1. **[docs/Planning/WAVE3_DETAILED_PLAN.md](./WAVE3_DETAILED_PLAN.md)** — 詳細計画・Day別タスク
2. **[docs/API/API_SPECIFICATION.md](../API/API_SPECIFICATION.md)** — 既存API仕様
3. **[docs/Performance/PERFORMANCE_BENCHMARKS.md](../Performance/PERFORMANCE_BENCHMARKS.md)** — SLO（p95 < 150ms）
4. **[AGENTS.md](../../AGENTS.md)** — プロジェクト全体構成

### 参考
- AWS SDK: https://github.com/awslabs/aws-sdk-rust
- MinIO: https://docs.min.io/
- async-trait: https://docs.rs/async-trait/

---

## 🔧 開発環境セットアップ

### Step 1: MinIO ローカル起動
```bash
docker-compose -f docker-compose.minio.yml up -d

# 確認
curl http://localhost:9000/minio/health/live
```

### Step 2: Storage モジュール作成
```bash
mkdir -p src/storage
touch src/storage/{mod.rs,local_backend.rs,s3_backend.rs,failover.rs,error.rs}
```

### Step 3: Cargo.toml 更新
```toml
[dependencies]
aws-sdk-s3 = "1.0"
aws-smithy-runtime = "1.0"
aws-config = "1.0"
async-trait = "0.1"
```

### Step 4: テスト実行
```bash
cargo test --lib storage

# MinIO テスト
cargo test --lib storage -- --include-ignored minio
```

---

## ⚙️ Config ファイル設定

### config/development.toml
```toml
[storage]
type = "local"  # local | s3 | failover

[s3]
bucket = "opencode-dev"
region = "us-east-1"
endpoint = "http://localhost:9000"  # MinIO
access_key = "minio"
secret_key = "minioadmin"
use_path_style = true
```

### config/production.toml
```toml
[storage]
type = "s3"  # AWS S3 本番運用

[s3]
bucket = "opencode-prod"
region = "us-west-2"
endpoint = "https://s3.amazonaws.com"
# access_key と secret_key は環境変数から
# AWS_ACCESS_KEY / AWS_SECRET_KEY
```

---

## 📊 進捗追跡方法

### Daily Standup チェックリスト
```
[ ] テスト数: X/38
[ ] ビルド: ✅ Debug / ✅ Release
[ ] パフォーマンス: p95 = XXms
[ ] ブロッカー: なし / [説明]
[ ] 次日予定: [タスク]
```

### テスト実行スクリプト
```bash
#!/bin/bash
# Wave 3 テスト統計

echo "=== Wave 3 Test Summary ==="
cargo test --lib storage -- --nocapture | grep "test result:"
echo ""
echo "Metrics:"
curl -s http://localhost:8080/api/v1/metrics | grep storage_
```

---

## 🎯 Wave 3 成功判定（Week 5 終了時）

```
✅ Storage Trait 実装完了
✅ S3 アップロード・ダウンロード動作
✅ Multipart Upload 実装完了
✅ Local ↔ S3 自動フェイルオーバー動作
✅ 38 テスト全パス
✅ メトリクス統合完了
✅ ドキュメント完成
✅ ロードテスト成功（p95 < 150ms）
```

成功 → **Wave 4 に進行可能**

---

## 💡 開発Tips

### 1. MinIO vs AWS S3
- **開発**: MinIO（ローカル）
- **ステージング**: MinIO（Docker）
- **本番**: AWS S3

### 2. エラーハンドリング
```rust
pub enum StorageError {
    NotFound,
    Unauthorized,
    InternalError(String),
    NetworkError(String),
}
```

### 3. テスト戦略
- Unit Test: 各 Backend の個別テスト
- Integration Test: MinIO + 実装
- E2E Test: Upload → Download → Delete サイクル

### 4. Failover テスト
```rust
#[test]
fn test_failover_on_primary_error() {
    // Primary失敗 → Secondary に自動切り替え
}
```

---

**Wave 3 実装ガイド完成！** 🎉

開始日: 2026-06-02  
終了予定日: 2026-06-21  
Location: docs/Planning/WAVE3_IMPLEMENTATION_GUIDE.md
