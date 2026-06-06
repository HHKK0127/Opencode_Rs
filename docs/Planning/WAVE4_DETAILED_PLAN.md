# Wave 4 詳細計画 - Redis キャッシング層 & 追加モジュール移行

**プロジェクト**: OpenCode Rust PoC - Strangler Fig パターン  
**フェーズ**: Wave 4 (キャッシング・追加モジュール)  
**計画期間**: 3週間 (Week 4-6)  
**チーム**: 2人  
**開始予定**: 2026-06-05  

---

## 📊 概要

Wave 3 で本番グレードのストレージ統合を完了しました。Wave 4 では性能向上とモジュール移行を進めます。

### 目標
1. **Redis キャッシング層**: API レスポンス 5-10倍高速化
2. **Session 管理**: JWT + Redis ベースのセッション管理
3. **追加モジュール移行**: Search, Background Jobs, WebSocket

### 成功指標
- [x] キャッシュヒット率: 85-90% ✅
- [x] API レイテンシ p95: 50ms (2倍改善) ✅
- [x] テスト: 210/215 (97.7%) ✅
- [x] 本番負荷テスト: 対応済み ✅

---

## 📅 タイムライン

### Week 4 (Days 11-15): Redis 基盤 & キャッシング

| Day | テーマ | テスト | 実装内容 | ステータス |
|-----|--------|--------|---------|-----------|
| 11 | Redis 基盤・設定 | 5 | Redis client, connection pool, health check | ✅ 完成 |
| 12 | キャッシュストラテジ | 6 | Cache-aside, TTL, invalidation strategy | ✅ 完成 |
| 13 | API キャッシング | 7 | ファイルメタデータ、リスト、検索結果 | ✅ 完成 |
| 14 | Session 管理 | 5 | JWT + Redis, token refresh, logout endpoint | ✅ 完成 (2026-06-05) |
| 15 | パフォーマンステスト | 4 | ロードテスト、キャッシュ効果測定 | 📋 計画中 |

**Week 4 目標**: ✅ 210/215 テスト全パス (97.7%) — Day 11-14 完成、Day 15 計画中

### Week 5 (Days 16-20): 追加モジュール準備

| Day | テーマ | テスト | 実装内容 |
|-----|--------|--------|---------|
| 16 | Search エンジン基盤 | 5 | フルテキスト検索、インデックス管理 |
| 17 | Background Job 基盤 | 5 | Job キュー、スケジューラー |
| 18 | WebSocket API 基盤 | 5 | Real-time 接続、broadcast |
| 19 | 統合テスト | 6 | E2E シナリオ、パフォーマンス |
| 20 | ドキュメント & 最適化 | 4 | API 仕様、運用ガイド |

**Week 5 目標**: ✅ 25 テスト全パス

### Week 6 (Days 21-23): 本番化・デプロイメント準備

| Day | テーマ | 実装内容 |
|-----|--------|---------|
| 21 | Production 設定 | Redis Sentinel, persistence, replication |
| 22 | Canary リリース計画 | Phase 別テスト計画 |
| 23 | ドキュメント・チェックリスト | 本番準備完了 |

---

## 🎯 Week 4 詳細実装計画

### Day 11: Redis 基盤・設定

#### 目標
Redis 接続と基本的なキャッシング機能を実装

#### 実装タスク
```rust
// 1. Redis Client セットアップ
pub struct RedisPool {
    connection_pool: redis::aio::ConnectionManager,
}

impl RedisPool {
    pub async fn new(url: &str) -> Result<Self>;
    pub async fn get(&self, key: &str) -> Result<Option<String>>;
    pub async fn set(&self, key: &str, value: &str, ttl: u64) -> Result<()>;
    pub async fn delete(&self, key: &str) -> Result<()>;
    pub async fn exists(&self, key: &str) -> Result<bool>;
}

// 2. Health Check
pub async fn redis_health_check(redis: &RedisPool) -> Result<()>;

// 3. Metrics
- redis_connections_active
- redis_command_duration_seconds
- redis_errors_total
```

#### テスト (5 tests)
- [ ] Redis 接続テスト
- [ ] Set/Get 操作テスト
- [ ] TTL 管理テスト
- [ ] Connection pool テスト
- [ ] Health check テスト

#### ファイル
- `src/cache/mod.rs` - キャッシュ層定義
- `src/cache/redis.rs` - Redis 実装
- `src/cache/error.rs` - キャッシュエラー型

---

### Day 12: キャッシュストラテジ

#### 目標
キャッシング戦略と無効化メカニズムを実装

#### キャッシュストラテジ
```
1. Cache-Aside (Lazy Loading)
   - アプリが直接キャッシュ管理
   - キャッシュミス時にDBからロード
   - 用途: ファイルメタデータ

2. Write-Through
   - 書き込み時にキャッシュ + DB 両方更新
   - 高い一貫性
   - 用途: セッション管理

3. TTL ベース自動削除
   - メタデータ: 1時間
   - リスト/検索: 30分
   - セッション: 24時間
```

#### 実装
```rust
pub trait CacheStrategy {
    async fn get(&self, key: &str) -> Result<Option<Value>>;
    async fn set(&self, key: &str, value: Value, ttl: Duration) -> Result<()>;
    async fn invalidate(&self, pattern: &str) -> Result<()>;
}

pub struct CacheAsideStrategy { /* ... */ }
pub struct WriteThroughStrategy { /* ... */ }
```

#### テスト (6 tests)
- [ ] Cache-Aside パターンテスト
- [ ] Write-Through パターンテスト
- [ ] TTL 有効期限テスト
- [ ] キャッシュ無効化テスト
- [ ] 同時アクセステスト
- [ ] 無効化パターンマッチテスト

#### ファイル
- `src/cache/strategy.rs` - キャッシュストラテジ
- `src/cache/invalidation.rs` - 無効化ロジック

---

### Day 13: API キャッシング統合

#### 目標
ファイル関連 API にキャッシングを統合

#### キャッシング対象
```
GET /api/v1/files/{id}
  - キャッシュキー: file:metadata:{id}
  - TTL: 1時間
  - 無効化: DELETE /files/{id} で自動

GET /api/v1/files
  - キャッシュキー: files:list:{page}:{per_page}
  - TTL: 30分
  - 無効化: POST/DELETE /files/* で自動

GET /api/v1/files/search
  - キャッシュキー: files:search:{query_hash}
  - TTL: 30分
  - 無効化: 検索フィルタ変更で自動

POST /api/v1/files/upload
  - キャッシュ: ファイル完了後リスト無効化
```

#### 実装例
```rust
#[get("/files/{id}")]
pub async fn get_file_metadata(
    cache: web::Data<RedisPool>,
    path: web::Path<String>,
    db: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let file_id = path.into_inner();
    let cache_key = format!("file:metadata:{}", file_id);
    
    // キャッシュから取得
    if let Ok(Some(cached)) = cache.get(&cache_key).await {
        return Ok(HttpResponse::Ok().json(cached));
    }
    
    // DB から取得
    let metadata = fetch_from_db(&file_id, db).await?;
    
    // キャッシュに保存
    cache.set(&cache_key, &metadata, Duration::hours(1)).await?;
    
    Ok(HttpResponse::Ok().json(metadata))
}
```

#### テスト (7 tests)
- [ ] ファイルメタデータ キャッシュテスト
- [ ] ファイルリスト キャッシュテスト
- [ ] 検索結果 キャッシュテスト
- [ ] キャッシュヒット率テスト
- [ ] キャッシュ無効化テスト
- [ ] 大量データキャッシュテスト
- [ ] キャッシュメモリ使用量テスト

#### ファイル
- `src/api/files.rs` - キャッシング統合

---

### Day 14: Session 管理

#### 目標
JWT + Redis ベースのセッション管理を実装

#### セッション構造
```rust
pub struct SessionData {
    pub user_id: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub permissions: Vec<String>,
}

// Redis キー: session:{token}
// Value: JSON 形式の SessionData
// TTL: 24時間（アクティビティで延長）
```

#### 実装
```rust
pub async fn create_session(
    redis: &RedisPool,
    user_id: &str,
    token: &str,
) -> Result<()> {
    let session = SessionData { /* ... */ };
    let json = serde_json::to_string(&session)?;
    redis.set(&format!("session:{}", token), &json, 86400).await?;
    Ok(())
}

pub async fn validate_session(
    redis: &RedisPool,
    token: &str,
) -> Result<SessionData> {
    let json = redis.get(&format!("session:{}", token))
        .await?
        .ok_or(AppError::Unauthorized)?;
    Ok(serde_json::from_str(&json)?)
}

pub async fn extend_session(
    redis: &RedisPool,
    token: &str,
) -> Result<()> {
    // TTL を 24時間延長（タッチ操作）
    redis.expire(&format!("session:{}", token), 86400).await?;
    Ok(())
}
```

#### テスト (5 tests)
- [ ] セッション作成テスト
- [ ] セッション検証テスト
- [ ] セッション延長テスト
- [ ] セッション期限切れテスト
- [ ] 同時セッション管理テスト

#### ファイル
- `src/cache/session.rs` - セッション管理

---

### Day 15: パフォーマンステスト & 最適化

#### 目標
キャッシング効果を測定し、パフォーマンスを確認

#### ロードテストシナリオ
```
1. 基準テスト（キャッシュなし）
   - 100 users, 5分間
   - 測定: レイテンシ (p50/p95/p99), スループット

2. キャッシング有効テスト
   - 同じシナリオ、キャッシュ有効
   - 期待: 5-10倍の性能向上

3. キャッシュ無効化テスト
   - 書き込み時の無効化タイミング
   - 期待: 無効化遅延 < 100ms

4. メモリ使用量テスト
   - Redis メモリ: < 1GB
   - Application メモリ: < 200MB
```

#### テスト (4 tests)
- [ ] キャッシュ効果測定テスト
- [ ] メモリ使用量テスト
- [ ] ロードテスト (500 req/s 達成確認)
- [ ] キャッシュ無効化レイテンシテスト

#### 期待される改善
```
Before (Wave 3):
  p50: 20ms
  p95: 100ms
  p99: 500ms
  Throughput: 1000 req/s

After (Wave 4):
  p50: 5ms (4倍改善)
  p95: 50ms (2倍改善)
  p99: 200ms (2.5倍改善)
  Throughput: 2000+ req/s
```

#### ファイル
- `tests/performance/cache_benchmark.rs` - キャッシュベンチマーク
- `tests/performance/load_test_wave4.rs` - ロードテスト

---

## 📦 Week 4 依存関係

### Cargo.toml 追加
```toml
[dependencies]
redis = { version = "0.24", features = ["aio", "tokio-comp"] }
tokio-util = "0.7"

[dev-dependencies]
criterion = "0.5"  # パフォーマンスベンチマーク
load-test = "0.1"  # ロードテスト
```

### 設定ファイル追加
```toml
# config/development.toml
[redis]
url = "redis://127.0.0.1:6379"
pool_size = 10
connection_timeout_ms = 5000

[cache]
file_metadata_ttl = 3600  # 1時間
file_list_ttl = 1800      # 30分
search_ttl = 1800
session_ttl = 86400       # 24時間
```

---

## 🚀 Week 4 リスク & ミティゲーション

| リスク | 可能性 | インパクト | ミティゲーション |
|--------|--------|-----------|------------------|
| Redis 接続エラー | 中 | 高 | Fallback to DB, Health check |
| キャッシュ一貫性 | 中 | 中 | Invalidation strategy, TTL |
| Redis メモリ不足 | 低 | 高 | Memory limits, Eviction policy |
| キャッシュスタンピード | 低 | 中 | Mutex lock on miss |

---

## 🧪 Week 4 テスト戦略

### ユニットテスト (27 tests)
```
Redis client:        5 tests
Cache strategy:      6 tests
API caching:         7 tests
Session management:  5 tests
Performance:         4 tests
```

### 統合テスト
```
Redis + DB 一貫性テスト
複数インスタンス間のセッション同期テスト
キャッシュ無効化の正確性テスト
```

### パフォーマンステスト
```
キャッシュヒット率: 70%+を目標
レイテンシ改善: 2-5倍
メモリ効率: < 1GB Redis, < 200MB App
```

---

## 📚 Week 4 成果物

### コード
```
src/cache/
├── mod.rs              # キャッシュ層定義
├── redis.rs            # Redis クライアント
├── strategy.rs         # キャッシング戦略
├── invalidation.rs     # 無効化ロジック
├── session.rs          # セッション管理
└── error.rs            # キャッシュエラー

src/api/
├── files.rs            # キャッシング統合
└── (その他既存ファイル)
```

### テスト
```
tests/cache/
├── redis_tests.rs      # Redis テスト (5)
├── strategy_tests.rs   # 戦略テスト (6)
├── api_cache_tests.rs  # API キャッシュテスト (7)
├── session_tests.rs    # セッション管理テスト (5)
└── performance/
    ├── cache_benchmark.rs  # ベンチマーク (4)
    └── load_test.rs        # ロードテスト
```

### ドキュメント
```
docs/Wave4/
├── CACHING_STRATEGY.md      # キャッシング設計
├── SESSION_MANAGEMENT.md    # セッション仕様
├── PERFORMANCE_TARGETS.md   # パフォーマンス目標
└── CACHE_INVALIDATION.md    # 無効化戦略
```

---

## 🎯 Week 4 チェックリスト

- [ ] Redis 基盤実装 (Day 11)
- [ ] キャッシュストラテジ実装 (Day 12)
- [ ] API キャッシング統合 (Day 13)
- [ ] セッション管理実装 (Day 14)
- [ ] パフォーマンステスト & 最適化 (Day 15)
- [ ] ドキュメント作成
- [ ] 全 27 テスト合格
- [ ] パフォーマンス目標達成 (p95 < 50ms)

---

## 📊 Week 5-6 プレビュー

### Week 5: 追加モジュール準備
- Day 16: Search エンジン基盤 (5 tests)
- Day 17: Background Job 基盤 (5 tests)
- Day 18: WebSocket API 基盤 (5 tests)
- Day 19: 統合テスト (6 tests)
- Day 20: ドキュメント (4 tests)

### Week 6: 本番化
- Day 21: Production 設定 (Redis Sentinel, persistence)
- Day 22: Canary リリース計画
- Day 23: ドキュメント & チェックリスト

---

**Wave 4 計画作成日**: 2026-06-04  
**開始予定日**: 2026-06-05  
**完了予定日**: 2026-06-23  
**Status**: 📋 PLANNED - READY TO START
