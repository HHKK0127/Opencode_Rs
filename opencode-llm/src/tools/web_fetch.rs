//! Web fetch tool — fetches a URL and returns markdown content.

use serde_json::Value;
use tracing::info;

use super::{ToolContext, ToolError, ToolSpec};

pub static NAME: &str = "web_fetch";
static DESCRIPTION: &str = r#"Fetch a URL from the internet and return the page content. Use this to retrieve up-to-date information from HTML web pages.

Args:
  url: The URL to fetch (must be http or https)."#;

pub fn spec() -> ToolSpec {
    ToolSpec::new(
        NAME,
        DESCRIPTION,
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "URL to fetch" },
                "max_length": { "type": "integer", "default": 8000, "description": "Maximum characters" }
            },
            "required": ["url"],
            "additionalProperties": false
        }),
    )
}

pub struct WebFetchTool;

#[async_trait::async_trait]
impl super::ToolRuntime for WebFetchTool {
    fn name(&self) -> &str {
        NAME
    }
    fn spec(&self) -> ToolSpec {
        spec()
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<String, ToolError> {
        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("missing 'url'".into()))?;
        let max_length = args
            .get("max_length")
            .and_then(|v| v.as_u64())
            .unwrap_or(8000);
        info!(url = %url, "web_fetch");
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| ToolError::Other(format!("http client error: {e}")))?;
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| ToolError::Other(format!("http request failed: {e}")))?;
        if !response.status().is_success() {
            return Ok(format!("HTTP {}", response.status()));
        }
        let body = response
            .text()
            .await
            .map_err(|e| ToolError::Other(format!("read body error: {e}")))?;
        let truncated = if body.len() > max_length as usize {
            let mut s: String = body.chars().take(max_length as usize).collect();
            s.push_str("\n...(truncated)");
            s
        } else {
            body
        };
        Ok(truncated)
    }
}
