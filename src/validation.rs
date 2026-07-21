#![allow(dead_code)]
use crate::error::{AppError, AppResult};
use uuid::Uuid;

/// File validation constraints for production
pub struct FileValidationRules {
    pub max_size_bytes: usize,
    pub allowed_mime_types: Vec<String>,
    pub disallowed_extensions: Vec<String>,
    pub max_filename_length: usize,
}

impl Default for FileValidationRules {
    fn default() -> Self {
        Self {
            max_size_bytes: 100 * 1024 * 1024, // 100MB
            allowed_mime_types: vec![
                "application/pdf".to_string(),
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "text/plain".to_string(),
                "text/csv".to_string(),
                "application/json".to_string(),
                "application/zip".to_string(),
                "application/octet-stream".to_string(),
            ],
            disallowed_extensions: vec![
                "exe".to_string(),
                "bat".to_string(),
                "cmd".to_string(),
                "com".to_string(),
                "msi".to_string(),
                "scr".to_string(),
                "vbs".to_string(),
                "js".to_string(),
            ],
            max_filename_length: 255,
        }
    }
}

pub struct FileValidator {
    rules: FileValidationRules,
}

impl FileValidator {
    pub fn new(rules: FileValidationRules) -> Self {
        Self { rules }
    }

    pub fn new_default() -> Self {
        Self::new(FileValidationRules::default())
    }

    /// Validate file size
    pub fn validate_size(&self, size: usize) -> AppResult<()> {
        if size == 0 {
            return Err(AppError::BadRequest("File size cannot be zero".to_string()));
        }
        if size > self.rules.max_size_bytes {
            return Err(AppError::PayloadTooLarge);
        }
        Ok(())
    }

    /// Validate filename
    pub fn validate_filename(&self, filename: &str) -> AppResult<()> {
        if filename.is_empty() {
            return Err(AppError::BadRequest("Filename cannot be empty".to_string()));
        }

        if filename.len() > self.rules.max_filename_length {
            return Err(AppError::BadRequest(format!(
                "Filename exceeds max length of {}",
                self.rules.max_filename_length
            )));
        }

        // Check for path traversal attempts
        if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
            return Err(AppError::BadRequest(
                "Filename contains invalid characters".to_string(),
            ));
        }

        // Validate against disallowed extensions
        let ext = filename.split('.').next_back().unwrap_or("").to_lowercase();

        if self.rules.disallowed_extensions.contains(&ext) {
            return Err(AppError::BadRequest(format!(
                "File extension .{} is not allowed",
                ext
            )));
        }

        Ok(())
    }

    /// Validate MIME type
    pub fn validate_mime_type(&self, mime_type: &str) -> AppResult<()> {
        if !self
            .rules
            .allowed_mime_types
            .contains(&mime_type.to_string())
        {
            return Err(AppError::BadRequest(format!(
                "MIME type {} is not allowed",
                mime_type
            )));
        }
        Ok(())
    }

    /// Comprehensive file validation
    pub fn validate_file(&self, filename: &str, mime_type: &str, size: usize) -> AppResult<()> {
        self.validate_filename(filename)?;
        self.validate_mime_type(mime_type)?;
        self.validate_size(size)?;
        Ok(())
    }
}

/// UUID形式の検証
pub fn is_valid_uuid(s: &str) -> bool {
    Uuid::parse_str(s).is_ok()
}

/// ファイル名のサニタイズ
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .replace("..", "")
        .replace("/", "")
        .replace("\\", "")
        .trim()
        .to_string()
}

/// ページ番号の検証（正の整数）
pub fn validate_page(page: Option<u32>) -> u32 {
    page.unwrap_or(1).max(1)
}

/// per_pageの検証（1-100）
pub fn validate_per_page(per_page: Option<u32>) -> u32 {
    per_page.unwrap_or(20).clamp(1, 100)
}

/// Validate request headers for security
pub fn validate_content_length(content_length: Option<u64>, max_bytes: usize) -> AppResult<()> {
    if let Some(length) = content_length {
        if length as usize > max_bytes {
            return Err(AppError::PayloadTooLarge);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_filename_empty() {
        let validator = FileValidator::new_default();
        assert!(validator.validate_filename("").is_err());
    }

    #[test]
    fn test_validate_filename_path_traversal() {
        let validator = FileValidator::new_default();
        assert!(validator.validate_filename("../etc/passwd").is_err());
        assert!(validator.validate_filename("../../config.toml").is_err());
    }

    #[test]
    fn test_validate_filename_disallowed_extension() {
        let validator = FileValidator::new_default();
        assert!(validator.validate_filename("malware.exe").is_err());
        assert!(validator.validate_filename("script.bat").is_err());
    }

    #[test]
    fn test_validate_filename_valid() {
        let validator = FileValidator::new_default();
        assert!(validator.validate_filename("document.pdf").is_ok());
        assert!(validator.validate_filename("image-file.png").is_ok());
        assert!(validator.validate_filename("data_2026.csv").is_ok());
    }

    #[test]
    fn test_validate_size_zero() {
        let validator = FileValidator::new_default();
        assert!(validator.validate_size(0).is_err());
    }

    #[test]
    fn test_validate_size_exceeds_limit() {
        let validator = FileValidator::new_default();
        let oversized = 101 * 1024 * 1024; // 101MB
        assert!(validator.validate_size(oversized).is_err());
    }

    #[test]
    fn test_valid_uuid() {
        assert!(is_valid_uuid("550e8400-e29b-41d4-a716-446655440000"));
        assert!(!is_valid_uuid("invalid-uuid"));
        assert!(!is_valid_uuid(""));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("../../../etc/passwd"), "etcpasswd");
        assert_eq!(sanitize_filename("file/name"), "filename");
        assert_eq!(sanitize_filename("normal-file.txt"), "normal-file.txt");
    }

    #[test]
    fn test_validate_page() {
        assert_eq!(validate_page(Some(5)), 5);
        assert_eq!(validate_page(Some(0)), 1); // 0 は 1 に正規化
        assert_eq!(validate_page(None), 1); // デフォルト
    }

    #[test]
    fn test_validate_per_page() {
        assert_eq!(validate_per_page(Some(20)), 20);
        assert_eq!(validate_per_page(Some(150)), 100); // 上限 100
        assert_eq!(validate_per_page(Some(0)), 1); // 下限 1
        assert_eq!(validate_per_page(None), 20); // デフォルト
    }
}
