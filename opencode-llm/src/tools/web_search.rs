//! Web search tool — performs web search via a configurable search API.
//!
//! Uses the `WEB_SEARCH_API_URL` environment variable (defaults to DuckDuckGo
//! Lite HTML search). When `WEB_SEARCH_API_KEY` is set, it adds an `Authorization`
//! header for authenticated search APIs.

use serde_json::Value;
use tracing::info;

use super::{ToolContext, ToolError, ToolSpec};

pub static NAME: &str = "web_search";
static DESCRIPTION: &str = r#"Search the web for up-to-date information. Returns text results from the search engine.
Use this for questions about recent events, new developments, trends, or any topic that requires current information.

Args:
  query: The search query or question.
  max_results: Maximum number of results to return (default: 5, max: 20)."#;

pub fn spec() -> ToolSpec {
    ToolSpec::new(
        NAME,
        DESCRIPTION,
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "The search query" },
                "max_results": { "type": "integer", "default": 5, "description": "Max results (1-20)" }
            },
            "required": ["query"],
            "additionalProperties": false
        }),
    )
}

pub struct WebSearchTool;

#[async_trait::async_trait]
impl super::ToolRuntime for WebSearchTool {
    fn name(&self) -> &str {
        NAME
    }
    fn spec(&self) -> ToolSpec {
        spec()
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<String, ToolError> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("missing 'query'".into()))?;
        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(5)
            .min(20) as usize;

        info!(query = %query, max_results, "web_search");

        // Use configurable search API with DuckDuckGo Lite as fallback.
        let api_url = std::env::var("WEB_SEARCH_API_URL")
            .unwrap_or_else(|_| "https://lite.duckduckgo.com/lite".to_string());
        let api_key = std::env::var("WEB_SEARCH_API_KEY").ok();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("OpenCode_Rs/1.0")
            .build()
            .map_err(|e| ToolError::Other(format!("http client error: {e}")))?;

        let mut req = client.get(&api_url).query(&[("q", query)]);

        if let Some(key) = &api_key {
            req = req.header("Authorization", format!("Bearer {key}"));
        }

        let response = req
            .send()
            .await
            .map_err(|e| ToolError::Other(format!("search request failed: {e}")))?;

        if !response.status().is_success() {
            // Fallback: return a summary message.
            return Ok(format!(
                "[web_search] Results for \"{query}\":\n  (Search API returned HTTP {})\n\n\
                 Tip: Set WEB_SEARCH_API_URL and WEB_SEARCH_API_KEY for better results,\n\
                 or use web_fetch to directly fetch specific URLs.",
                response.status()
            ));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ToolError::Other(format!("read body error: {e}")))?;

        // Extract text content from HTML response.
        let text = extract_text(&body, max_results);
        Ok(text)
    }
}

/// Crude HTML-to-text extraction for search results.
fn extract_text(html: &str, max_results: usize) -> String {
    let mut results = Vec::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut current = String::new();

    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '<' if i + 6 < chars.len()
                && chars[i..i + 7].iter().collect::<String>().to_lowercase() == "<script" =>
            {
                in_script = true;
                in_tag = true;
                if !current.trim().is_empty() {
                    results.push(current.trim().to_string());
                    current.clear();
                }
            }
            '<' if i + 5 < chars.len()
                && chars[i..i + 6].iter().collect::<String>().to_lowercase() == "<style" =>
            {
                in_style = true;
                in_tag = true;
                if !current.trim().is_empty() {
                    results.push(current.trim().to_string());
                    current.clear();
                }
            }
            '<' => {
                in_tag = true;
                if !current.trim().is_empty() {
                    let trimmed = current.trim().to_string();
                    // Skip very short fragments.
                    if trimmed.len() > 3 {
                        results.push(trimmed);
                    }
                    current.clear();
                }
            }
            '>' => {
                in_tag = false;
                if in_script {
                    if i > 0 && chars[i - 1] == '/' {
                        // self-closing
                    }
                    // Check for </script>
                    if i + 8 < chars.len()
                        && chars[i..i + 9].iter().collect::<String>().to_lowercase() == "</script>"
                    {
                        in_script = false;
                        i += 8;
                    }
                }
                if in_style
                    && i + 6 < chars.len()
                    && chars[i..i + 7].iter().collect::<String>().to_lowercase() == "</style>"
                {
                    in_style = false;
                    i += 6;
                }
            }
            _ => {
                if !in_tag && !in_script && !in_style {
                    current.push(chars[i]);
                }
            }
        }
        i += 1;
    }

    if !current.trim().is_empty() {
        let trimmed = current.trim().to_string();
        if trimmed.len() > 3 {
            results.push(trimmed);
        }
    }

    // Deduplicate and limit.
    let mut seen = std::collections::BTreeSet::new();
    let mut output = String::new();
    output.push_str("[web_search] Results for query\n\n");
    let mut count = 0;
    for line in results {
        let clean = line.trim();
        if clean.is_empty() || clean.len() < 10 {
            continue;
        }
        if seen.insert(clean.to_lowercase()) {
            output.push_str(&format!("  • {clean}\n"));
            count += 1;
            if count >= max_results {
                break;
            }
        }
    }

    if count == 0 {
        output.push_str("  (No results found)\n");
    }

    output
}
