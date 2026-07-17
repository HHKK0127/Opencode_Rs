//! Grep search tool — search file contents with regex.

use serde_json::Value;
use tokio::task::spawn_blocking;
use tracing::info;

use super::{ToolContext, ToolError, ToolSpec};

pub static NAME: &str = "grep_search";
static DESCRIPTION: &str = r#"Search file contents using a regular expression pattern. Fast pre-filtering uses billions-of-lines-proven ripgrep engine.

Args:
  pattern: The regular expression to search for.
  paths: Optional comma-separated list of directories to search. Defaults to workspace root.
  glob: Optional glob filter for file types (e.g. "*.rs", "*.{ts,tsx}").
  case_sensitive: Whether the search is case-sensitive. Defaults to true."#;

pub fn spec() -> ToolSpec {
    ToolSpec::new(
        NAME,
        DESCRIPTION,
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string" },
                "paths": { "type": "string", "description": "Comma-separated directories" },
                "glob": { "type": "string", "description": "Glob filter for file types" },
                "case_sensitive": { "type": "boolean", "default": true }
            },
            "required": ["pattern"],
            "additionalProperties": false
        }),
    )
}

pub struct GrepSearchTool;

#[async_trait::async_trait]
impl super::ToolRuntime for GrepSearchTool {
    fn name(&self) -> &str { NAME }
    fn spec(&self) -> ToolSpec { spec() }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<String, ToolError> {
        let pattern = args
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("missing 'pattern'".into()))?;
        let glob = args.get("glob").and_then(|v| v.as_str());
        let case_sensitive = args
            .get("case_sensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let root = ctx.cwd.clone();
        let pat_owned = pattern.to_string();
        let glob_owned = glob.map(|s| s.to_string());

        info!(pattern = %pattern, ?root, "grep_search");
        let results = spawn_blocking(move || -> Result<String, ToolError> {
            let mut cmd = std::process::Command::new("rg");
            cmd.arg("--no-heading")
                .arg("--with-filename")
                .arg("--line-number")
                .arg("-n")
                .arg("-C").arg("2");
            if let Some(ref g) = glob_owned {
                cmd.arg("--glob").arg(g);
            }
            if !case_sensitive {
                cmd.arg("-i");
            }
            cmd.arg(&pat_owned).arg(&root);
            let output = cmd.output().map_err(ToolError::Io)?;
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                if stdout.is_empty() { Ok("(no matches)".to_string()) } else { Ok(stdout) }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if stderr.contains("not found") || stderr.contains("No such file") {
                    Ok("(no matches)".to_string())
                } else {
                    Ok(format!("rg exited with {}: {}", output.status, stderr))
                }
            }
        })
        .await
        .map_err(|e| ToolError::Other(format!("join error: {e}")))?;
        results
    }
}
