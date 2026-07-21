use actix_web::{get, web, HttpResponse};
use serde::Serialize;

use crate::app_state::AppState;
use crate::error::AppResult;

#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub created_at: String,
}

#[get("/users")]
pub async fn list_users(app_state: web::Data<AppState>) -> AppResult<HttpResponse> {
    let users = sqlx::query_as::<_, UserResponse>(
        r#"
        SELECT id, username, created_at::text
        FROM users
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&app_state.db)
    .await
    .map_err(|e| crate::error::AppError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(users))
}

#[get("/users/{id}")]
pub async fn get_user(
    app_state: web::Data<AppState>,
    id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let user = sqlx::query_as::<_, UserResponse>(
        r#"
        SELECT id, username, created_at::text
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id.as_str())
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| crate::error::AppError::Database(e.to_string()))?
    .ok_or(crate::error::AppError::NotFound)?;

    Ok(HttpResponse::Ok().json(user))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_users).service(get_user);
}
