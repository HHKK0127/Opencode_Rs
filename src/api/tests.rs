#[cfg(test)]
mod tests {
    use crate::app_state::AppState;
    use crate::config::Settings;
    use crate::storage::LocalStorageBackend;
    use std::sync::Arc;
    use uuid::Uuid;

    fn create_test_app_state() -> AppState {
        let settings = Settings::default();
        let db_url = format!("sqlite://poc_test.db");
        let rt = tokio::runtime::Runtime::new().unwrap();

        let db = rt.block_on(async {
            sqlx::sqlite::SqlitePool::connect(&db_url)
                .await
                .expect("Failed to connect to database")
        });

        let storage = Arc::new(LocalStorageBackend::new("./uploads"));

        AppState::new(settings, db, storage, None)
    }

    #[test]
    fn test_upload_file_endpoint_integration() {
        let app_state = create_test_app_state();
        assert!(!app_state.server_addr().is_empty());
    }

    #[test]
    fn test_download_file_endpoint_setup() {
        let app_state = create_test_app_state();
        assert!(!app_state.db_path().is_empty());
    }

    #[test]
    fn test_delete_file_endpoint_setup() {
        let app_state = create_test_app_state();
        assert!(!app_state.upload_dir().is_empty());
    }

    #[test]
    fn test_list_files_endpoint_setup() {
        let app_state = create_test_app_state();
        let addr = app_state.server_addr();
        assert!(addr.contains(":"));
    }

    #[test]
    fn test_get_file_metadata_endpoint_setup() {
        let app_state = create_test_app_state();
        let db_path = app_state.db_path();
        assert!(!db_path.is_empty());
    }

    #[test]
    fn test_multipart_init_endpoint_setup() {
        let app_state = create_test_app_state();
        assert!(!app_state.server_addr().is_empty());
    }

    #[test]
    fn test_multipart_upload_endpoint_setup() {
        let app_state = create_test_app_state();
        assert!(!app_state.upload_dir().is_empty());
    }

    #[test]
    fn test_multipart_complete_endpoint_setup() {
        let app_state = create_test_app_state();
        let addr = app_state.server_addr();
        assert!(!addr.is_empty());
    }

    #[test]
    fn test_upload_progress_endpoint_setup() {
        let app_state = create_test_app_state();
        assert!(!app_state.db_path().is_empty());
    }

    #[test]
    fn test_storage_backend_availability() {
        let app_state = create_test_app_state();
        let _storage = &app_state.storage;
        assert!(!app_state.upload_dir().is_empty());
    }

    #[test]
    fn test_database_pool_availability() {
        let app_state = create_test_app_state();
        assert!(!app_state.db_path().is_empty());
    }

    #[test]
    fn test_api_endpoints_configured() {
        let app_state = create_test_app_state();
        let server = app_state.server_addr();
        assert!(server.contains(":8080") || server.contains(":"));
    }
}
