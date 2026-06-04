⚠️ **これはAIモデル（Claude Haiku 4.5）が作成したコードです。**
人間のレビューと検証が必要です。

---

# コードレビュー依頼

## ファイル情報
- **ファイル名**: `src/api/file_search.rs`
- **言語**: Rust
- **行数**: 244行
- **目的**: Wave 4 Day 13 - File Search API with Redis Caching (Actix-web 4.5)

## コード

```rust
//! File Search API with Redis Caching - Actix-web 4.5 Implementation
//!
//! Cache-Aside pattern implementation for search results
//! TTL: 30 minutes
//! Cache Key: files:search:{query_hash}:{page}:{per_page}

use actix_web::{
    web,
    HttpResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::{
    app_state::AppState,
    error::{AppError, AppResult},
    models::FileMetadata,
    cache::REDIS_OPERATIONS_TOTAL,
};

/// 検索クエリパラメータ
#[derive(Debug, Deserialize)]
pub struct FileSearchQuery {
    pub q: Option<String>,
    pub file_type: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl FileSearchQuery {
    /// キャッシュキー用のハッシュを生成
    fn cache_key(&self, page: i64, per_page: i64) -> String {
        let mut hasher = DefaultHasher::new();

        // 検索パラメータをハッシュ
        self.q.hash(&mut hasher);
        self.file_type.hash(&mut hasher);
        self.tags.hash(&mut hasher);
        self.created_after.hash(&mut hasher);
        self.created_before.hash(&mut hasher);
        page.hash(&mut hasher);
        per_page.hash(&mut hasher);

        format!("files:search:{:016x}:{}:{}", hasher.finish(), page, per_page)
    }
}

/// 検索結果レスポンス
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileSearchResponse {
    pub files: Vec<FileMetadata>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
    pub cached: bool,
}

/// GET /api/v1/files/search - 検索エンドポイント（キャッシュ付き）
pub async fn search_files(
    query: web::Query<FileSearchQuery>,
    app_state: web::Data<AppState>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    let cache_key = query.cache_key(page, per_page);

    // Phase 1: Try cache first (Cache-Aside pattern)
    if let Some(ref cache) = app_state.cache {
        match cache.get::<FileSearchResponse>(&cache_key).await {
            Ok(Some(cached_result)) => {
                debug!("Cache hit for search: {}", cache_key);

                // Record metrics
                REDIS_OPERATIONS_TOTAL.with_label_values(&["search_cache_hit"]).inc();

                let mut response = cached_result;
                response.cached = true;
                return Ok(HttpResponse::Ok().json(response));
            }
            Ok(None) => {
                debug!("Cache miss for search: {}", cache_key);
                REDIS_OPERATIONS_TOTAL.with_label_values(&["search_cache_miss"]).inc();
            }
            Err(e) => {
                warn!("Cache error, falling back to DB: {}", e);
                REDIS_OPERATIONS_TOTAL.with_label_values(&["search_cache_error"]).inc();
            }
        }
    }

    // Phase 2: Fetch from database (stubbed)
    info!("Executing search query: {:?}", query);

    // In production, this would query the database
    // For now, return empty search results
    let total_pages = (0i64 + per_page - 1) / per_page;

    let response = FileSearchResponse {
        files: vec![],
        total: 0,
        page,
        per_page,
        total_pages,
        cached: false,
    };

    // Phase 3: Store in cache
    if let Some(ref cache) = app_state.cache {
        let cache_result = cache
            .set_with_ttl(
                &cache_key,
                &response,
                Duration::from_secs(30 * 60), // 30 minutes
            )
            .await;

        match cache_result {
            Ok(_) => {
                REDIS_OPERATIONS_TOTAL.with_label_values(&["search_cache_set"]).inc();
                debug!("Cached search results: {}", cache_key);
            }
            Err(e) => {
                warn!("Failed to cache search results: {}", e);
                REDIS_OPERATIONS_TOTAL.with_label_values(&["search_cache_set_error"]).inc();
            }
        }
    }

    Ok(HttpResponse::Ok().json(response))
}

/// キャッシュ無効化ヘルパー（ファイル変更時に呼び出し）
pub async fn invalidate_search_cache(
    app_state: &web::Data<AppState>,
) -> Result<(), AppError> {
    if let Some(ref cache) = app_state.cache {
        // 検索キャッシュのパターンを無効化
        let pattern = "files:search:*";

        match cache.delete_pattern(pattern).await {
            Ok(deleted) => {
                info!("Invalidated {} search cache entries", deleted);
                REDIS_OPERATIONS_TOTAL.with_label_values(&["search_cache_invalidate"]).inc();
                Ok(())
            }
            Err(e) => {
                warn!("Failed to invalidate search cache: {}", e);
                Err(AppError::Internal)
            }
        }
    } else {
        Ok(())
    }
}

/// 特定ユーザーの検索キャッシュ無効化
pub async fn invalidate_user_search_cache(
    app_state: &web::Data<AppState>,
    _user_id: &str,
) -> Result<(), AppError> {
    if let Some(ref cache) = app_state.cache {
        let pattern = "files:search:*";

        match cache.delete_pattern(pattern).await {
            Ok(deleted) => {
                debug!("Invalidated {} search cache entries", deleted);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to invalidate search cache: {}", e);
                Err(AppError::Internal)
            }
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let query = FileSearchQuery {
            q: Some("test".to_string()),
            file_type: Some("pdf".to_string()),
            tags: Some(vec!["important".to_string()]),
            created_after: None,
            created_before: None,
            page: Some(1),
            per_page: Some(20),
        };

        let key1 = query.cache_key(1, 20);
        let key2 = query.cache_key(1, 20);
        let key3 = query.cache_key(2, 20);

        // Same query should produce same key
        assert_eq!(key1, key2);
        // Different page should produce different key
        assert_ne!(key1, key3);

        // Verify format
        assert!(key1.starts_with("files:search:"));
        assert!(key1.contains(":1:20"));
    }

    #[test]
    fn test_cache_key_consistency() {
        let query1 = FileSearchQuery {
            q: Some("rust".to_string()),
            file_type: None,
            tags: None,
            created_after: None,
            created_before: None,
            page: Some(1),
            per_page: Some(50),
        };

        let query2 = FileSearchQuery {
            q: Some("rust".to_string()),
            file_type: None,
            tags: None,
            created_after: None,
            created_before: None,
            page: Some(1),
            per_page: Some(50),
        };

        // Identical queries should produce identical keys
        assert_eq!(query1.cache_key(1, 50), query2.cache_key(1, 50));
    }
}
```

## レビュー観点

以下の観点でレビューをお願いします：

### 1. バグ・エラー
- 明らかな構文エラー
- ランタイムエラーの可能性
- エッジケースの処理漏れ

### 2. セキュリティ
- 入力検証の不足
- インジェクション攻撃の可能性
- 機密情報の露出

### 3. パフォーマンス
- 無駄な計算やループ
- メモリ効率
- 非効率なアルゴリズム

### 4. 可読性・保守性
- 命名の適切さ
- コメントの不足
- 複雑すぎるロジック

### 5. ベストプラクティス
- Rust / Actix-web イディオム
- エラーハンドリング
- キャッシングパターン

## 回答形式

以下の形式で回答してください：

```
【総合評価】
LGTM / 軽微な修正 / 要修正

【問題点】
1. [重大度] [問題の説明]
   - 場所: [行番号または関数名]
   - 修正案: [具体的な修正コード]

【改善提案】
1. [提案]: [理由]

【修正後のコード（該当部分）】
\`\`\`rust
[修正後のコード]
\`\`\`
```

---

**作成日時**: 2026-06-04 23:08:26  
**作成者**: Claude Code (Haiku 4.5)  
**プロジェクト**: OpenCode Rust PoC - Wave 4 Day 13  
**ブランチ**: feature/wave4-day13-api-caching

---

## 重要な確認事項

このコードは以下を前提としています：

1. **AppState** が `cache: Option<Arc<dyn CacheOperations>>` フィールドを持つ
2. **CacheOperations** トレイトに以下のメソッドが実装されている：
   - `get::<T>(&self, key: &str) -> Result<Option<T>, CacheError>`
   - `set_with_ttl::<T>(&self, key: &str, value: &T, ttl: Duration) -> Result<(), CacheError>`
   - `delete_pattern(&self, pattern: &str) -> Result<usize, CacheError>`
3. **REDIS_OPERATIONS_TOTAL** メトリクスが Prometheus 経由で利用可能
4. **FileMetadata** が Serialize/Deserialize を実装している

---

**次のステップ**: 
- このコードをレビュー
- フィードバック内容を基に修正
- 次セッションで files.rs, routes.rs, テストを実装

