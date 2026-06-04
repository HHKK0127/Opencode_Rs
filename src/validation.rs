use crate::error::{AppError, AppResult};

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
            return Err(AppError::BadRequest(
                format!("Filename exceeds max length of {}", self.rules.max_filename_length),
            ));
        }

        // Check for path traversal attempts
        if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
            return Err(AppError::BadRequest(
                "Filename contains invalid characters".to_string(),
            ));
        }

        // Validate against disallowed extensions
        let ext = filename
            .split('.')
            .last()
            .unwrap_or("")
            .to_lowercase();

        if self.rules.disallowed_extensions.contains(&ext) {
            return Err(AppError::BadRequest(
                format!("File extension .{} is not allowed", ext),
            ));
        }

        Ok(())
    }

    /// Validate MIME type
    pub fn validate_mime_type(&self, mime_type: &str) -> AppResult<()> {
        if !self.rules.allowed_mime_types.contains(&mime_type.to_string()) {
            return Err(AppError::BadRequest(
                format!("MIME type {} is not allowed", mime_type),
            ));
        }
        Ok(())
    }

    /// Comprehensive file validation
    pub fn validate_file(
        &self,
        filename: &str,
        mime_type: &str,
        size: usize,
    ) -> AppResult<()> {
        self.validate_filename(filename)?;
        self.validate_mime_type(mime_type)?;
        self.validate_size(size)?;
        Ok(())
    }
}

/// Sanitize filename - remove/replace dangerous characters
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_end_matches('.')
        .to_string()
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
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("file name.pdf"), "file_name.pdf");
        assert_eq!(sanitize_filename("../../../etc"), ".._.._.._etc");
        assert_eq!(sanitize_filename("normal-file.txt"), "normal-file.txt");
    }
}
