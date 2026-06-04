use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::app_state::AppState;

#[derive(Debug, Deserialize)]
pub struct FileSearchQuery {
    q: Option<String>,               // フリーワード検索
    mime_type: Option<String>,       // MIMEタイプフィルタ
    size_min: Option<i64>,           // 最小サイズ（バイト）
    size_max: Option<i64>,           // 最大サイズ（バイト）
    created_after: Option<String>,   // 作成日時フィルタ（RFC3339）
    sort: Option<String>,            // ソート（created_at, size, filename）
    order: Option<String>,           // asc/desc
    page: Option<i64>,               // ページ番号
    per_page: Option<i64>,           // 1ページあたりの件数
}

#[derive(Debug, Serialize)]
pub struct FileSearchResult {
    id: String,
    filename: String,
    original_name: String,
    size: i64,
    mime_type: String,
    created_at: String,
    url: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    files: Vec<FileSearchResult>,
    pagination: PaginationInfo,
    query_summary: QuerySummary,
}

#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    page: i64,
    per_page: i64,
    total: i64,
    total_pages: i64,
}

#[derive(Debug, Serialize)]
pub struct QuerySummary {
    keyword: Option<String>,
    filters_applied: i32,
    results_count: i64,
}

#[get("/files/search")]
pub async fn search_files(
    app_state: web::Data<AppState>,
    query: web::Query<FileSearchQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * per_page;

    // Build SQL query dynamically
    let mut sql = String::from(
        "SELECT id, filename, original_name, size, mime_type, created_at FROM files WHERE 1=1"
    );
    let mut params: Vec<String> = Vec::new();

    // フリーワード検索（filename または original_name）
    if let Some(q) = &query.q {
        sql.push_str(" AND (filename LIKE ? OR original_name LIKE ?)");
        let search_pattern = format!("%{}%", q);
        params.push(search_pattern.clone());
        params.push(search_pattern);
    }

    // MIMEタイプフィルタ
    if let Some(mime) = &query.mime_type {
        sql.push_str(" AND mime_type = ?");
        params.push(mime.clone());
    }

    // サイズ範囲フィルタ
    if let Some(size_min) = query.size_min {
        sql.push_str(" AND size >= ?");
        params.push(size_min.to_string());
    }
    if let Some(size_max) = query.size_max {
        sql.push_str(" AND size <= ?");
        params.push(size_max.to_string());
    }

    // 作成日時フィルタ
    if let Some(created_after) = &query.created_after {
        sql.push_str(" AND created_at >= ?");
        params.push(created_after.clone());
    }

    // ソート（安全性のためホワイトリスト）
    let sort_field = match query.sort.as_deref() {
        Some("size") => "size",
        Some("filename") => "filename",
        Some("original_name") => "original_name",
        _ => "created_at",
    };
    let sort_order = match query.order.as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    };
    sql.push_str(&format!(" ORDER BY {} {}", sort_field, sort_order));

    // ページネーション
    sql.push_str(&format!(" LIMIT {} OFFSET {}", per_page, offset));

    // Execute query with dynamic parameters
    let files = execute_dynamic_query(&app_state.db, &sql, &params)
        .await?;

    // Get total count
    let mut count_sql = String::from("SELECT COUNT(*) as count FROM files WHERE 1=1");
    if query.q.is_some() {
        count_sql.push_str(" AND (filename LIKE ? OR original_name LIKE ?)");
    }
    if query.mime_type.is_some() {
        count_sql.push_str(" AND mime_type = ?");
    }
    if query.size_min.is_some() {
        count_sql.push_str(" AND size >= ?");
    }
    if query.size_max.is_some() {
        count_sql.push_str(" AND size <= ?");
    }
    if query.created_after.is_some() {
        count_sql.push_str(" AND created_at >= ?");
    }

    let total: (i64,) = sqlx::query_as(&count_sql)
        .bind(&query.q.as_ref().map(|q| format!("%{}%", q)))
        .bind(&query.q.as_ref().map(|q| format!("%{}%", q)))
        .bind(&query.mime_type)
        .bind(query.size_min)
        .bind(query.size_max)
        .bind(&query.created_after)
        .fetch_one(&app_state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let total_pages = (total.0 + per_page - 1) / per_page;

    let file_list: Vec<FileSearchResult> = files
        .iter()
        .map(|f| FileSearchResult {
            id: f.0.clone(),
            filename: f.1.clone(),
            original_name: f.2.clone(),
            size: f.3,
            mime_type: f.4.clone(),
            created_at: f.5.clone(),
            url: format!("/api/v1/files/{}/download", f.0),
        })
        .collect();

    let filters_applied = [
        query.q.is_some(),
        query.mime_type.is_some(),
        query.size_min.is_some(),
        query.size_max.is_some(),
        query.created_after.is_some(),
    ]
    .iter()
    .filter(|&&f| f)
    .count() as i32;

    Ok(HttpResponse::Ok().json(SearchResponse {
        files: file_list,
        pagination: PaginationInfo {
            page,
            per_page,
            total: total.0,
            total_pages,
        },
        query_summary: QuerySummary {
            keyword: query.q.clone(),
            filters_applied,
            results_count: total.0,
        },
    }))
}

async fn execute_dynamic_query(
    db: &sqlx::sqlite::SqlitePool,
    sql: &str,
    _params: &[String],
) -> AppResult<Vec<(String, String, String, i64, String, String)>> {
    // Note: sqlx doesn't support dynamic parameter binding in production
    // For a real implementation, use a query builder or construct safe queries

    // Simplified: Execute base query
    let result = sqlx::query_as::<_, (String, String, String, i64, String, String)>(sql)
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(result)
}

#[get("/files/stats")]
pub async fn file_statistics(
    app_state: web::Data<AppState>,
) -> AppResult<HttpResponse> {
    let stats = sqlx::query_as::<_, (i64, Option<i64>, Option<f64>, i64)>(
        "SELECT COUNT(*) as total_files, SUM(size) as total_size, AVG(size) as avg_size, COUNT(DISTINCT mime_type) as unique_mimes FROM files"
    )
    .fetch_one(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_files": stats.0,
        "total_size_bytes": stats.1.unwrap_or(0),
        "average_size_bytes": stats.2.unwrap_or(0.0) as i64,
        "unique_mime_types": stats.3,
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(search_files)
        .service(file_statistics);
}
