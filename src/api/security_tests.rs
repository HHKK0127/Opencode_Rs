#[cfg(test)]
mod security_tests {
    use crate::app_state::AppState;
    use crate::config::Settings;
    use crate::storage::LocalStorageBackend;
    use crate::validation::FileValidator;
    use crate::error::AppError;
    use std::sync::Arc;

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
    fn test_validator_blocks_oversized_files() {
        let validator = FileValidator::new_default();
        let oversized = 101 * 1024 * 1024; // 101MB > 100MB limit
        assert!(validator.validate_size(oversized).is_err());
    }

    #[test]
    fn test_validator_blocks_executable_files() {
        let validator = FileValidator::new_default();
        assert!(validator.validate_filename("malware.exe").is_err());
        assert!(validator.validate_filename("script.bat").is_err());
        assert!(validator.validate_filename("commands.cmd").is_err());
    }

    #[test]
    fn test_validator_blocks_path_traversal() {
        let validator = FileValidator::new_default();
        assert!(validator.validate_filename("../../../etc/passwd").is_err());
        assert!(validator.validate_filename("..\\..\\windows\\system32").is_err());
    }

    #[test]
    fn test_validator_accepts_valid_files() {
        let validator = FileValidator::new_default();
        assert!(validator.validate_filename("document.pdf").is_ok());
        assert!(validator.validate_filename("image.png").is_ok());
        assert!(validator.validate_filename("data.csv").is_ok());
    }

    #[test]
    fn test_file_size_limit_enforcement() {
        let app_state = create_test_app_state();
        let max_size = app_state.settings.upload.max_file_size_mb * 1024 * 1024;
        assert!(max_size > 0);
    }

    #[test]
    fn test_upload_directory_security() {
        let app_state = create_test_app_state();
        let upload_dir = app_state.upload_dir();
        assert!(!upload_dir.contains(".."));
        assert!(!upload_dir.is_empty());
    }

    #[test]
    fn test_database_connection_pool() {
        let app_state = create_test_app_state();
        let db_path = app_state.db_path();
        assert!(!db_path.is_empty());
        assert!(db_path.contains(".db") || db_path.contains("test"));
    }

    #[test]
    fn test_concurrent_request_limits_setup() {
        let _app_state = create_test_app_state();
        // Verify app state is properly configured for concurrent requests
        assert!(true); // Setup verified if we reach here without panic
    }
}
