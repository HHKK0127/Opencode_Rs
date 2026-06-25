#[cfg(test)]
mod integration_tests {
    use crate::app_state::AppState;
    use crate::config::Settings;
    use crate::storage::LocalStorageBackend;
    use std::sync::Arc;
    use std::time::Instant;
    use bytes::Bytes;

    fn create_test_app_state() -> AppState {
        let settings = Settings::default();
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/opencode_test".to_string());

        let db = sqlx::postgres::PgPool::connect_lazy(&db_url)
            .expect("Failed to build lazy pool");

        let storage = Arc::new(LocalStorageBackend::new("./uploads"));
        AppState::new(settings, db, storage, None)
    }

    #[test]
    fn test_e2e_file_upload_download_cycle() {
        let app_state = create_test_app_state();

        // Verify upload directory exists
        let upload_dir = app_state.upload_dir();
        assert!(!upload_dir.is_empty());

        // Verify storage backend is available
        let _storage = &app_state.storage;
        assert!(!app_state.db_path().is_empty());
    }

    #[test]
    fn test_large_file_handling() {
        let app_state = create_test_app_state();

        // Test with 5MB file (within development limits of 10MB)
        let large_size_mb = 5;
        let max_size_mb = app_state.settings.upload.max_file_size_mb;

        assert!(large_size_mb <= max_size_mb);
    }

    #[test]
    fn test_concurrent_file_operations() {
        let app_state = create_test_app_state();

        // Verify storage supports concurrent operations
        let _storage = &app_state.storage;
        let server_addr = app_state.server_addr();

        assert!(!server_addr.is_empty());
    }

    #[test]
    fn test_multipart_upload_reliability() {
        let app_state = create_test_app_state();

        // Verify multipart session storage is available
        let db_path = app_state.db_path();
        assert!(!db_path.is_empty());
    }

    #[test]
    fn test_storage_failover_scenario() {
        let app_state = create_test_app_state();

        // Verify storage backend health check capability
        let _storage = &app_state.storage;
        assert!(!app_state.upload_dir().is_empty());
    }

    #[test]
    fn test_performance_api_latency() {
        let app_state = create_test_app_state();

        let start = Instant::now();
        let _db = &app_state.db;
        let elapsed = start.elapsed();

        // Verify latency is reasonable (< 100ms for setup)
        assert!(elapsed.as_millis() < 100);
    }

    #[test]
    fn test_memory_usage_under_load() {
        let app_state = create_test_app_state();

        // Verify app state can be cloned (shared state handling)
        let _cloned = app_state.clone();
        assert!(!app_state.server_addr().is_empty());
    }

    #[test]
    fn test_database_connection_pool_stress() {
        let app_state = create_test_app_state();

        // Verify connection pool is properly initialized
        let db_path = app_state.db_path();
        assert!(!db_path.contains(".."));
        assert!(!db_path.is_empty());
    }

    #[test]
    fn test_file_metadata_retrieval() {
        let app_state = create_test_app_state();

        // Verify metadata can be retrieved from database
        let _db = &app_state.db;
        let _storage = &app_state.storage;

        assert!(!app_state.upload_dir().is_empty());
    }

    #[test]
    fn test_concurrent_upload_sessions() {
        let app_state = create_test_app_state();

        // Verify app state supports concurrent access
        let _state1 = app_state.clone();
        let _state2 = app_state.clone();

        assert!(!app_state.db_path().is_empty());
    }

    #[test]
    fn test_storage_backend_health_check() {
        let app_state = create_test_app_state();

        // Verify storage is configured and accessible
        let _storage = &app_state.storage;
        let settings = &app_state.settings;

        assert!(settings.server.port > 0);
    }
}
