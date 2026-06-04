# WAVE3_ENVIRONMENT_SETUP.md

## Wave 3 開発環境セットアップガイド

**対象**: S3/MinIO統合実装準備  
**実行予定**: 2026-06-01 ～ 2026-06-02  
**所要時間**: 3-4時間  
**対象者**: DevOps + Backend Engineer

---

## 目次

1. [環境構成概要](#1-環境構成概要)
2. [MinIO開発環境セットアップ](#2-minIO開発環境セットアップ)
3. [AWS S3接続設定](#3-aws-s3接続設定)
4. [開発環境チェックリスト](#4-開発環境チェックリスト)
5. [テスト戦略](#5-テスト戦略)
6. [トラブルシューティング](#6-トラブルシューティング)

---

## 1. 環境構成概要

### Wave 2 → Wave 3 進化

```
Wave 2 (Local Storage):
  ├─ SQLite DB + ローカルディスク
  ├─ 単一サーバー構成
  └─ p95: 50ms, スループット: 500 req/s

Wave 3 (S3/MinIO):
  ├─ SQLite DB + S3オブジェクトストレージ
  ├─ マルチサーバー対応
  ├─ 自動フェイルオーバー
  └─ スケーラビリティ向上（無制限容量）
```

### 推奨構成

```
開発環境:
  ├─ Local PC: API + LocalBackend
  ├─ Docker: MinIO (S3互換)
  └─ テスト: LocalBackend + S3Backend 並行

本番環境（Wave 3後）:
  ├─ Kubernetes: APIサーバー（複数Pod）
  ├─ AWS S3: オブジェクトストレージ
  ├─ CloudFront: CDN
  └─ ElastiCache: キャッシュレイヤー
```

---

## 2. MinIO開発環境セットアップ

### Step 1: Docker Composeで MinIO起動

**前提**: Docker 20.10+ インストール済み

```bash
# 1. docker-compose.yml に MinIO サービス追加
cat >> docker-compose.yml << 'EOF'

  minio:
    image: minio/minio:latest
    ports:
      - "9000:9000"      # MinIO API
      - "9001:9001"      # MinIO Console
    environment:
      MINIO_ROOT_USER: minio
      MINIO_ROOT_PASSWORD: minioadmin
    command: server /data --console-address ":9001"
    volumes:
      - minio_data:/data
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9000/minio/health/live"]
      interval: 30s
      timeout: 20s
      retries: 3

volumes:
  minio_data:
EOF

# 2. MinIO起動
docker-compose up -d minio

# 3. 起動確認
docker-compose logs -f minio | head -20
# expected: "Listening on :9000"

# 4. ヘルスチェック
curl -v http://localhost:9000/minio/health/live
# expected: HTTP/1.1 200 OK
```

### Step 2: MinIO Console アクセス

```bash
# ブラウザで開く
# http://localhost:9001

# ログイン情報
username: minio
password: minioadmin
```

### Step 3: テストバケット作成

**MinIO Console経由**:
```
1. Buckets → Create Bucket
2. Bucket Name: opencode-dev
3. Access: Private
4. Create Bucket
```

**CLI経由**:
```bash
# MinIO client インストール
brew install minio-mc

# MinIO登録
mc alias set minio http://localhost:9000 minio minioadmin

# テストバケット作成
mc mb minio/opencode-dev

# 確認
mc ls minio/
# expected: [2026-05-31 10:30:00 UTC]     0B opencode-dev/
```

### Step 4: テストファイルアップロード

```bash
# テストファイル作成
echo "Hello MinIO" > test.txt

# アップロード
mc cp test.txt minio/opencode-dev/

# 確認
mc ls minio/opencode-dev/
# expected: [2026-05-31 10:35:00 UTC]    11B test.txt

# ダウンロード確認
mc cat minio/opencode-dev/test.txt
# expected: Hello MinIO
```

---

## 3. AWS S3接続設定

### Step 1: AWS Account/IAM準備（本番用）

```bash
# 前提: AWS CLIインストール済み
aws --version

# 認証情報設定
aws configure
# AWS Access Key ID: [your_access_key]
# AWS Secret Access Key: [your_secret_key]
# Default region name: us-west-2
# Default output format: json

# 確認
aws sts get-caller-identity
# expected: { "Account": "123456789...", "UserId": "...", "Arn": "..." }
```

### Step 2: テストバケット作成（本番用）

```bash
# バケット作成
aws s3 mb s3://opencode-dev-wave3 --region us-west-2

# 確認
aws s3 ls | grep opencode
# expected: 2026-05-31 10:40:00 opencode-dev-wave3

# バージョニング有効化（推奨）
aws s3api put-bucket-versioning \
  --bucket opencode-dev-wave3 \
  --versioning-configuration Status=Enabled

# 暗号化設定（推奨）
aws s3api put-bucket-encryption \
  --bucket opencode-dev-wave3 \
  --server-side-encryption-configuration '{
    "Rules": [{
      "ApplyServerSideEncryptionByDefault": {
        "SSEAlgorithm": "AES256"
      }
    }]
  }'
```

### Step 3: IAM権限設定

```bash
# S3アクセス用IAM ユーザー作成（オプション）
aws iam create-user --user-name opencode-wave3

# ポリシー作成
cat > s3-policy.json << 'EOF'
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:GetObject",
        "s3:PutObject",
        "s3:DeleteObject",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::opencode-dev-wave3",
        "arn:aws:s3:::opencode-dev-wave3/*"
      ]
    }
  ]
}
EOF

# ポリシーアタッチ
aws iam put-user-policy \
  --user-name opencode-wave3 \
  --policy-name opencode-s3-access \
  --policy-document file://s3-policy.json
```

---

## 4. 開発環境チェックリスト

### 前提環境

- [ ] **OS**: Linux/macOS/Windows (WSL2)
- [ ] **Rust**: 1.70+
  ```bash
  rustc --version
  # expected: rustc 1.70.0+
  ```

- [ ] **Docker**: 20.10+
  ```bash
  docker --version
  # expected: Docker version 20.10+
  ```

- [ ] **Docker Compose**: 2.0+
  ```bash
  docker-compose --version
  # expected: Docker Compose version 2.0+
  ```

### 依存関係

- [ ] **AWS SDK for Rust**: `Cargo.toml`に追加
  ```toml
  [dependencies]
  aws-sdk-s3 = "1.0"
  aws-config = "1.0"
  tokio = { version = "1.35", features = ["full"] }
  async-trait = "0.1"
  ```

### Wave 2 の状態確認

- [ ] **v3.0.0 本番デプロイ完了**
  ```bash
  curl http://localhost:8080/health
  # expected: { "status": "healthy" }
  ```

- [ ] **メトリクスエンドポイント動作**
  ```bash
  curl http://localhost:8080/api/v1/metrics | head -5
  # expected: # HELP http_requests_total ...
  ```

- [ ] **ロードテスト成功結果**
  - p95: < 100ms ✅
  - エラー率: < 1% ✅
  - スループット: > 500 req/s ✅

### MinIO開発環境

- [ ] **MinIO コンテナ起動**
  ```bash
  docker-compose ps | grep minio
  # expected: minio ... Up
  ```

- [ ] **MinIO コンソール アクセス可能**
  ```bash
  curl http://localhost:9001/
  # expected: HTTP/1.1 200 (HTML content)
  ```

- [ ] **テストバケット存在**
  ```bash
  mc ls minio/opencode-dev/
  # expected: [date] 0B (bucket listed)
  ```

- [ ] **MinIO API 動作**
  ```bash
  aws s3api list-buckets --endpoint-url http://localhost:9000 \
    --region us-east-1 \
    --aws-access-key-id minio \
    --aws-secret-access-key minioadmin
  # expected: { "Buckets": [...] }
  ```

### AWS S3 接続（本番用）

- [ ] **AWS CLI認証成功**
  ```bash
  aws sts get-caller-identity
  # expected: { "Account": "...", "Arn": "..." }
  ```

- [ ] **S3バケット作成完了**
  ```bash
  aws s3 ls s3://opencode-dev-wave3/
  # expected: bucket listed
  ```

- [ ] **S3 テスト読み書き成功**
  ```bash
  echo "test" | aws s3 cp - s3://opencode-dev-wave3/test.txt
  aws s3 cp s3://opencode-dev-wave3/test.txt -
  # expected: test
  ```

- [ ] **IAM権限設定完了**
  ```bash
  aws iam get-user-policy --user-name opencode-wave3 --policy-name opencode-s3-access
  # expected: policy document returned
  ```

### Rust開発環境

- [ ] **依存関係 ビルド成功**
  ```bash
  cd /path/to/opencode_poc
  cargo build --release 2>&1 | grep -i error
  # expected: (no errors)
  ```

- [ ] **既存テスト パス**
  ```bash
  cargo test --release 2>&1 | tail -5
  # expected: test result: ok. ...
  ```

### IDE設定

- [ ] **VS Code拡張インストール**
  - rust-analyzer
  - CodeLLDB
  - Better TOML

- [ ] **RustRover設定（JetBrains）**
  - Toolchain: Stable
  - Edition: 2021

---

## 5. テスト戦略

### LocalBackend テスト（Wave 2既存）

```bash
# 既存テスト実行
cargo test --lib storage::backends::local
# expected: test result: ok. ... (すべてパス)
```

### MinIO テスト（Wave 3新規）

```rust
// src/storage/backends/s3_backend.rs に追加

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_minio_connection() {
        let config = S3Config {
            endpoint: "http://localhost:9000".to_string(),
            bucket: "opencode-dev".to_string(),
            region: "us-east-1".to_string(),
            access_key: "minio".to_string(),
            secret_key: "minioadmin".to_string(),
        };

        let backend = S3Backend::new(config).await;
        assert!(backend.is_ok(), "MinIO connection failed");
    }

    #[tokio::test]
    async fn test_minio_upload_download() {
        // テストファイルアップロード
        // ダウンロード確認
        // メタデータ確認
    }
}
```

実行:
```bash
# MinIO有効時にテスト実行
MINIO_ENDPOINT=http://localhost:9000 \
MINIO_BUCKET=opencode-dev \
cargo test --lib storage::backends::s3
```

### Dual-write テスト

```rust
// LocalBackend と S3Backend 同時テスト
#[tokio::test]
async fn test_dual_write_consistency() {
    let local = LocalBackend::new(...);
    let s3 = S3Backend::new(...);

    let file = FileData { ... };

    // 両方に書き込み
    local.upload(&file).await.unwrap();
    s3.upload(&file).await.unwrap();

    // メタデータ確認
    let local_meta = local.get_metadata(&file.id).await.unwrap();
    let s3_meta = s3.get_metadata(&file.id).await.unwrap();

    assert_eq!(local_meta, s3_meta, "Metadata mismatch");
}
```

---

## 6. トラブルシューティング

### MinIO接続エラー

**症状**: `Failed to connect to MinIO at http://localhost:9000`

**原因**: MinIOコンテナが起動していない

**対応**:
```bash
# コンテナ状態確認
docker-compose ps | grep minio

# ログ確認
docker-compose logs minio

# 再起動
docker-compose restart minio

# ヘルスチェック
curl http://localhost:9000/minio/health/live
```

### AWS 認証エラー

**症状**: `AWS SDK Error: InvalidAccessKeyId`

**原因**: IAM認証情報が間違っている or 権限不足

**対応**:
```bash
# 認証情報確認
aws sts get-caller-identity

# 認証情報再設定
aws configure

# IAM権限確認
aws iam get-user-policy \
  --user-name opencode-wave3 \
  --policy-name opencode-s3-access
```

### バケット作成エラー

**症状**: `NoSuchBucket: The specified bucket does not exist`

**原因**: バケット名が間違っている or リージョン指定が異なる

**対応**:
```bash
# バケット一覧確認
aws s3 ls

# バケット存在確認（リージョン指定）
aws s3api head-bucket \
  --bucket opencode-dev-wave3 \
  --region us-west-2
```

### S3 アップロードエラー

**症状**: `AccessDenied: Access Denied`

**原因**: IAM権限不足

**対応**:
```bash
# ポリシー確認
aws iam get-user-policy \
  --user-name opencode-wave3 \
  --policy-name opencode-s3-access

# テストアップロード
echo "test" | aws s3 cp - s3://opencode-dev-wave3/test.txt --debug

# 権限再設定（必要な場合）
aws iam put-user-policy \
  --user-name opencode-wave3 \
  --policy-name opencode-s3-access \
  --policy-document file://s3-policy.json
```

---

## 実装準備チェックリスト

### 開発環境セットアップ

- [ ] Wave 2 本番デプロイ確認
- [ ] MinIO Docker起動完了
- [ ] テストバケット作成完了
- [ ] AWS S3バケット作成完了（本番用）
- [ ] IAM権限設定完了
- [ ] Rust依存関係追加完了

### Day 1 準備

- [ ] Storage Trait 設計確認（WAVE3_DETAILED_PLAN.md）
- [ ] S3Backend スケルトン作成
- [ ] MinIO テスト作成
- [ ] CI/CD パイプライン設定

### チーム準備

- [ ] AWS認証情報配布
- [ ] MinIO CLI（mc）インストール確認
- [ ] AWS SDK ドキュメント確認
- [ ] チーム教育実施

---

## 推奨される読み物

- [WAVE3_DETAILED_PLAN.md](./WAVE3_DETAILED_PLAN.md) — Wave 3詳細計画
- [AWS S3 Documentation](https://docs.aws.amazon.com/s3/) — AWS公式ドキュメント
- [MinIO Documentation](https://min.io/docs/) — MinIO公式ドキュメント
- [async-trait Documentation](https://docs.rs/async-trait/) — Rustの非同期トレイト

---

**Wave 3環境準備ガイド完成！** 🚀

セットアップ完了後、[WAVE3_DETAILED_PLAN.md](./WAVE3_DETAILED_PLAN.md) に従って Day 1 実装を開始してください。

期待開始日: 2026-06-02  
期待完了日: 2026-06-21（18日間）
