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
    cache::metrics::REDIS_OPERATIONS_TOTAL,
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
    pub files: Vec<serde_json::Value>, // Generic JSON response for search results
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
            .set(
                &cache_key,
                &response,
                Some(Duration::from_secs(30 * 60)), // 30 minutes
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
        // 検索キャッシュを無効化
        // Note: Pattern matching would require SCAN/KEYS command implementation
        // For now, we just log that invalidation is needed
        info!("Invalidated search cache (pattern-based deletion not yet implemented)");
        REDIS_OPERATIONS_TOTAL.with_label_values(&["search_cache_invalidate"]).inc();
        Ok(())
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
        // Pattern-based invalidation not yet implemented
        // This would require SCAN command support
        debug!("Invalidated search cache for user (pattern-based deletion not yet implemented)");
        Ok(())
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

/// ルーティング設定
pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::resource("/files/search")
            .route(actix_web::web::get().to(search_files))
    );
}
