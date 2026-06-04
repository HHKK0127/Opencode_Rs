# Wave 3 Detailed Plan - S3/MinIO Integration

## 1. Overview

### Objective
ファイルストレージのS3/MinIO統合によるスケーラビリティ向上

### Timeline
- **Period**: 3週間 (2026-06-02 ～ 2026-06-21)
- **Team**: 2人
- **Week 3**: Days 1-5 (Core Implementation)
- **Week 4-5**: Days 6-10 (Integration & Production)

### Success Criteria
- [ ] Storage Trait実装完了
- [ ] S3アップロード・ダウンロード動作
- [ ] Multipart Upload実装完了
- [ ] Local ↔ S3 自動フェイルオーバー動作
- [ ] 38個テスト全パス
- [ ] メトリクス統合完了
- [ ] ドキュメント完成
- [ ] ロードテスト成功（p95 < 150ms）

---

## 2. Technical Architecture

### Storage Abstraction Layer
```
┌─────────────────────────────────────┐
│           API Layer                 │
│  (upload, download, delete)         │
└─────────────┬───────────────────────┘
              ▼
┌─────────────────────────────────────┐
│    Storage Trait (Abstract)         │
│  + store(file): Result<Url>         │
│  + retrieve(id): Result<File>       │
│  + delete(id): Result<()>           │
└─────────────┬───────────────────────┘
              ▼
    ┌─────────────┬─────────────┐
    ▼             ▼             ▼
┌────────┐  ┌────────┐  ┌────────┐
│ Local  │  │  S3    │  │ MinIO  │
│Backend │  │Backend │  │Backend │
└────────┘  └────────┘  └────────┘
```

### Component Design

#### Storage Trait (src/storage/mod.rs)
```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn store(&self, data: Bytes, metadata: FileMetadata) -> Result<StorageUrl, StorageError>;
    async fn retrieve(&self, id: &str) -> Result<Bytes, StorageError>;
    async fn delete(&self, id: &str) -> Result<(), StorageError>;
    async fn exists(&self, id: &str) -> Result<bool, StorageError>;
    async fn health_check(&self) -> Result<(), StorageError>;
}

pub struct FileMetadata {
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    pub user_id: Uuid,
}
```

#### Backend Implementations
- `LocalStorageBackend` - 既存ローカルファイルシステム
- `S3StorageBackend` - AWS S3 / MinIO 対応
- `FailoverStorageBackend` - 自動フェイルオーバー

---

## 3. Day-by-Day Implementation Plan

### Day 1: S3 Backend Foundation (8 tests)

#### Tasks
- [ ] Storage Trait設計・実装
- [ ] S3Client ラッパー実装
- [ ] MinIO Docker Composeセットアップ
- [ ] テスト基盤整備

#### New Files
```
src/
  storage/
    mod.rs              # Trait definitions
    local_backend.rs    # Local filesystem impl
    s3_backend.rs       # S3/MinIO impl
    failover.rs         # Automatic failover
    error.rs            # Error types
```

#### Dependencies
```toml
[dependencies]
aws-sdk-s3 = "1.0"
aws-smithy-runtime = "1.0"
aws-config = "1.0"
```

#### MinIO Docker Compose
```yaml
# docker-compose.minio.yml
version: '3.8'
services:
  minio:
    image: minio/minio:latest
    ports:
      - "9000:9000"
      - "9001:9001"
    environment:
      MINIO_ROOT_USER: minio
      MINIO_ROOT_PASSWORD: minioadmin
    command: server /data --console-address ":9001"
    volumes:
      - minio_data:/data

volumes:
  minio_data:
```

#### Test Cases (8)
1. Storage trait compilation
2. Local backend basic operations
3. S3 backend initialization
4. S3 health check
5. MinIO connection
6. Config parsing (local)
7. Config parsing (s3)
8. Backend factory pattern

---

### Day 2: Upload & Download (12 tests, 4 endpoints)

#### Tasks
- [ ] PUT操作実装
- [ ] GET操作実装
- [ ] 404/エラーハンドリング
- [ ] Range Request対応

#### New API Endpoints
```
POST /api/v1/files/upload-s3          ← S3直接アップロード
GET  /api/v1/files/{id}/download-s3   ← S3直接ダウンロード
HEAD /api/v1/files/{id}/exists-s3     ← 存在確認
DELETE /api/v1/files/{id}/delete-s3   ← S3削除
```

#### Implementation Details
```rust
// S3 upload with presigned URL support
pub async fn upload_to_s3(
    &self,
    data: Bytes,
    metadata: FileMetadata
) -> Result<String, StorageError> {
    let key = format!("uploads/{}/{}", metadata.user_id, metadata.filename);
    
    self.client
        .put_object()
        .bucket(&self.bucket)
        .key(&key)
        .body(data.into())
        .content_type(metadata.content_type)
        .send()
        .await
        .map_err(|e| StorageError::S3Error(e.to_string()))?;
    
    Ok(key)
}
```

#### Test Cases (12)
1. Simple upload to S3
2. Upload with metadata
3. Download from S3
4. Download 404 handling
5. Delete from S3
6. Range request (partial)
7. Large file upload (>10MB)
8. Content-type preservation
9. Concurrent uploads
10. S3 failure fallback
11. Presigned URL generation
12. URL expiration handling

---

### Day 3: Multipart Upload & Optimization (10 tests)

#### Tasks
- [ ] Multipart Upload API設計
- [ ] チャンク最適化
- [ ] 進捗トラッキング（S3用）
- [ ] コスト最適化（ストレージクラス）

#### Multipart Flow
```
1. POST /upload-s3/init     → UploadId返却
2. POST /upload-s3/chunk    → ETag返却（繰り返し）
3. POST /upload-s3/complete → 完了確認
```

#### Test Cases (10)
1. Multipart init
2. Single part upload
3. Multiple parts
4. Complete multipart
5. Abort multipart
6. Resume upload
7. Parallel part upload
8. Large file (1GB) multipart
9. Invalid part number
10. Incomplete upload cleanup

---

### Day 4: Migration & Failover (8 tests)

#### Tasks
- [ ] Local → S3 マイグレーション関数
- [ ] 自動フェイルオーバー戦略
- [ ] Dual-write テスト
- [ ] パフォーマンス検証

#### Failover Strategy
```rust
pub struct FailoverStorage {
    primary: Box<dyn StorageBackend>,
    secondary: Box<dyn StorageBackend>,
    health_checker: HealthChecker,
}
```

#### Test Cases (8)
1. Primary success
2. Primary failure → secondary success
3. Both failure
4. Health check recovery
5. Dual-write consistency
6. Migration dry-run
7. Migration actual
8. Rollback migration

---

### Day 5: Monitoring & Operations

#### Tasks
- [ ] S3メトリクス追加
- [ ] Cost tracking メトリクス
- [ ] ロードテスト（S3対応）
- [ ] オペレーション手順書

#### New Metrics
```
s3_upload_duration_seconds
s3_download_duration_seconds
s3_upload_size_bytes
s3_request_errors_total
storage_failover_events_total
storage_cost_estimate_dollars
```

---

## 4. Configuration System

### Development Config
```toml
# config/development.toml
[storage]
type = "local"  # local | s3 | failover
local_dir = "./uploads"

[s3]
bucket = "opencode-dev"
region = "us-east-1"
endpoint = "http://localhost:9000"  # MinIO
access_key = "minio"
secret_key = "minioadmin"
use_path_style = true
```

### Production Config
```toml
# config/production.toml
[storage]
type = "s3"

[s3]
bucket = "opencode-prod"
region = "us-west-2"
endpoint = "https://s3.amazonaws.com"
access_key = "${AWS_ACCESS_KEY}"
secret_key = "${AWS_SECRET_KEY}"
storage_class = "STANDARD_IA"
```

---

## 5. API Changes

### New Endpoints
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /api/v1/files/upload-s3 | Direct S3 upload |
| POST | /api/v1/files/upload-s3/init | Multipart init |
| POST | /api/v1/files/upload-s3/chunk | Upload part |
| POST | /api/v1/files/upload-s3/complete | Complete multipart |
| GET | /api/v1/files/{id}/download-s3 | S3 redirect/download |
| DELETE | /api/v1/files/{id}/delete-s3 | S3 delete |
| GET | /api/v1/storage/health | Storage health |

### Backward Compatibility
Existing endpoints automatically route based on `storage.type` config

---

## 6. Testing Strategy

### Unit Tests (15)
- Trait implementation
- Error handling
- Config parsing
- URL generation

### Integration Tests (18)
- MinIO operations
- Failover scenarios
- Migration
- Multipart upload

### E2E Tests (5)
- Full upload-download cycle
- Large file handling
- Concurrent access
- Failover simulation
- Migration validation

**Total: 38 tests**

---

## 7. Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| MinIO互換性問題 | 中 | 高 | Day 1で十分テスト |
| AWS SDK統合複雑性 | 高 | 中 | 段階的実装、十分なUT |
| マイグレーション失敗 | 低 | 高 | Dual-write、ロールバック手順 |
| パフォーマンス低下 | 中 | 中 | インデックス継続、メトリクス監視 |

---

## 8. Success Criteria

```
□ Storage Trait実装完了
□ S3アップロード・ダウンロード動作
□ Multipart Upload実装完了
□ Local ↔ S3 自動フェイルオーバー動作
□ 38個テスト全パス
□ メトリクス統合完了
□ ドキュメント完成
□ ロードテスト成功（p95 < 150ms）
```

---

**Wave 3 Detailed Plan - S3/MinIO Integration**  
**Implementation Start**: 2026-06-02  
**Expected Completion**: 2026-06-21  
**Location**: docs/Planning/WAVE3_DETAILED_PLAN.md
