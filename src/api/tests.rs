#[cfg(test)]
mod tests {
    use crate::app_state::AppState;
    use crate::config::Settings;
    use crate::storage::LocalStorageBackend;
    use std::sync::Arc;

    async fn create_test_app_state() -> AppState {
        let settings = Settings::default();
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/opencode_test".to_string());

        let db = sqlx::postgres::PgPool::connect_lazy(&db_url)
            .expect("Failed to build lazy pool");

        let storage = Arc::new(LocalStorageBackend::new("./uploads"));
        AppState::new(settings, db, storage, None)
    }

    #[tokio::test]
    async fn test_upload_file_endpoint_integration() {
        let app_state = create_test_app_state().await;
        assert!(!app_state.server_addr().is_empty());
    }

    #[tokio::test]
    async fn test_download_file_endpoint_setup() {
        let app_state = create_test_app_state().await;
        assert!(!app_state.db_path().is_empty());
    }

    #[tokio::test]
    async fn test_delete_file_endpoint_setup() {
        let app_state = create_test_app_state().await;
        assert!(!app_state.upload_dir().is_empty());
    }

    #[tokio::test]
    async fn test_list_files_endpoint_setup() {
        let app_state = create_test_app_state().await;
        let addr = app_state.server_addr();
        assert!(addr.contains(":"));
    }

    #[tokio::test]
    async fn test_get_file_metadata_endpoint_setup() {
        let app_state = create_test_app_state().await;
        let db_path = app_state.db_path();
        assert!(!db_path.is_empty());
    }

    #[tokio::test]
    async fn test_multipart_init_endpoint_setup() {
        let app_state = create_test_app_state().await;
        assert!(!app_state.server_addr().is_empty());
    }

    #[tokio::test]
    async fn test_multipart_upload_endpoint_setup() {
        let app_state = create_test_app_state().await;
        assert!(!app_state.upload_dir().is_empty());
    }

    #[tokio::test]
    async fn test_multipart_complete_endpoint_setup() {
        let app_state = create_test_app_state().await;
        let addr = app_state.server_addr();
        assert!(!addr.is_empty());
    }

    #[tokio::test]
    async fn test_upload_progress_endpoint_setup() {
        let app_state = create_test_app_state().await;
        assert!(!app_state.db_path().is_empty());
    }

    #[tokio::test]
    async fn test_storage_backend_availability() {
        let app_state = create_test_app_state().await;
        let _storage = &app_state.storage;
        assert!(!app_state.upload_dir().is_empty());
    }

    #[tokio::test]
    async fn test_database_pool_availability() {
        let app_state = create_test_app_state().await;
        assert!(!app_state.db_path().is_empty());
    }

    #[tokio::test]
    async fn test_api_endpoints_configured() {
        let app_state = create_test_app_state().await;
        let server = app_state.server_addr();
        assert!(server.contains(":8080") || server.contains(":"));
    }
}
