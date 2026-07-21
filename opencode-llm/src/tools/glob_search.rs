//! Glob search tool — find files by glob pattern.

use serde_json::Value;
use tokio::task::spawn_blocking;
use tracing::info;

use super::{ToolContext, ToolError, ToolSpec};
use std::path::Path;

pub static NAME: &str = "glob_search";
static DESCRIPTION: &str = r#"Find files matching a glob pattern. Uses fast file pattern matching. Supports standard glob patterns with wildcards: *, ?, **, {a,b}.

Args:
  pattern: The glob pattern to search for (e.g. "src/**/*.rs", "*.{ts,tsx}").
  root_dir: Optional root directory. Defaults to the workspace root."#;

pub fn spec() -> ToolSpec {
    ToolSpec::new(
        NAME,
        DESCRIPTION,
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "Glob pattern to match" },
                "root_dir": { "type": "string", "description": "Optional root directory" }
            },
            "required": ["pattern"],
            "additionalProperties": false
        }),
    )
}

pub struct GlobSearchTool;

#[async_trait::async_trait]
impl super::ToolRuntime for GlobSearchTool {
    fn name(&self) -> &str {
        NAME
    }
    fn spec(&self) -> ToolSpec {
        spec()
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<String, ToolError> {
        let pattern = args
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("missing 'pattern'".into()))?;
        let root = args
            .get("root_dir")
            .and_then(|v| v.as_str())
            .map(|s| ctx.cwd.join(s))
            .unwrap_or_else(|| ctx.cwd.clone());
        info!(pattern = %pattern, ?root, "glob_search");
        let pattern_owned = pattern.to_string();
        let results = spawn_blocking(move || -> Result<Vec<String>, ToolError> {
            let mut matches = Vec::new();
            let glob_pattern = glob::Pattern::new(&pattern_owned)
                .map_err(|e| ToolError::Other(format!("invalid glob pattern: {e}")))?;
            walk_dir(&root, &root, &glob_pattern, &mut matches)?;
            matches.sort();
            Ok(matches)
        })
        .await
        .map_err(|e| ToolError::Other(format!("join error: {e}")))?;
        let results = results?;
        if results.is_empty() {
            Ok("(no matches)".to_string())
        } else {
            Ok(results.join("\n"))
        }
    }
}

fn walk_dir(
    base: &Path,
    dir: &Path,
    pattern: &glob::Pattern,
    out: &mut Vec<String>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_dir(base, &path, pattern, out)?;
        } else if let Ok(rel) = path.strip_prefix(base) {
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if pattern.matches(&rel_str) || pattern.matches(&path.to_string_lossy()) {
                out.push(path.to_string_lossy().to_string());
            }
        }
    }
    Ok(())
}
