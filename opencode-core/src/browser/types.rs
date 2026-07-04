use serde::{Deserialize, Serialize};
use validator::Validate;

use super::BrowserError;

// === リクエスト型 ===

#[derive(Debug, Deserialize, Validate)]
pub struct NavigateArgs {
    #[validate(length(min = 1))]
    pub url: String,
    pub new_tab: Option<bool>,
    pub group_title: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct FindTabArgs {
    #[validate(length(min = 1))]
    pub url: String,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ClickArgs {
    #[validate(length(min = 1, max = 500))]
    pub selector: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct FillArgs {
    #[validate(length(min = 1, max = 500))]
    pub selector: String,
    #[validate(length(max = 100_000))]
    pub value: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct EvaluateArgs {
    #[validate(length(max = 1_000_000))]
    pub code: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ScreenshotArgs {
    pub format: Option<String>,
    #[validate(range(min = 0, max = 100))]
    pub quality: Option<u8>,
    pub selector: Option<String>,
    pub path: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SaveAsPdfArgs {
    pub paper_format: Option<String>,
    pub landscape: Option<bool>,
    #[validate(range(min = 0.1, max = 2.0))]
    pub scale: Option<f64>,
    pub print_background: Option<bool>,
    pub path: String,
}

// === レスポンス型 ===

#[derive(Debug, Serialize)]
pub struct NavigateResponse {
    pub success: bool,
    pub url: String,
    pub tab_id: String,
}

#[derive(Debug, Serialize)]
pub struct FindTabResponse {
    pub success: bool,
    pub url: String,
    pub tab_id: String,
}

#[derive(Debug, Serialize)]
pub struct SnapshotResponse {
    pub url: String,
    pub title: String,
    pub tree: Vec<AccessibilityNode>,
}

#[derive(Debug, Serialize)]
pub struct AccessibilityNode {
    #[serde(rename = "@ref")]
    pub e_ref: String,
    pub role: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<AccessibilityNode>,
}

#[derive(Debug, Serialize)]
pub struct ClickResponse {
    pub success: bool,
    pub tag: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct FillResponse {
    pub success: bool,
    pub tag: String,
    pub mode: String,
}

#[derive(Debug, Serialize)]
pub struct EvaluateResponse {
    #[serde(rename = "type")]
    pub result_type: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ScreenshotResponse {
    pub format: String,
    pub path: String,
    pub size_bytes: u64,
    pub mime_type: String,
}

#[derive(Debug, Serialize)]
pub struct PdfResponse {
    pub path: String,
    pub size_bytes: u64,
    pub mime_type: String,
    pub page_title: String,
}

#[derive(Debug, Serialize)]
pub struct TabsResponse {
    pub success: bool,
    pub tabs: Vec<TabInfo>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TabInfo {
    pub tab_id: String,
    pub url: String,
    pub title: String,
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_title: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CloseTabResponse {
    pub success: bool,
    pub closed: bool,
}

#[derive(Debug, Serialize)]
pub struct CloseSessionResponse {
    pub success: bool,
    pub closed: usize,
}

// === パス検証 ===

fn validate_path(path: &str, base_dir: &str) -> Result<std::path::PathBuf, BrowserError> {
    use std::path::Path;
    let base = Path::new(base_dir).canonicalize()
        .map_err(|_| BrowserError::MissingPath)?;
    let target = base.join(path);
    let canonical = target.canonicalize().unwrap_or(target.clone());

    if !canonical.starts_with(&base) {
        return Err(BrowserError::PathTraversal(path.to_string()));
    }
    Ok(canonical)
}

impl ScreenshotArgs {
    pub fn validate_path(&self, base_dir: &str) -> Result<std::path::PathBuf, BrowserError> {
        validate_path(&self.path, base_dir)
    }
}

impl SaveAsPdfArgs {
    pub fn validate_path(&self, base_dir: &str) -> Result<std::path::PathBuf, BrowserError> {
        validate_path(&self.path, base_dir)
    }
}
