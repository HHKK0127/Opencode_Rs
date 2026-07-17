//! Edit file tool — applies surgical text replacements.

use serde_json::Value;
use tokio::fs;
use tracing::info;

use super::{ToolContext, ToolError, ToolSpec};

pub static NAME: &str = "edit_file";
static DESCRIPTION: &str = r#"Apply a surgical text replacement to a file. Finds the exact `old_str` in the file and replaces it with `new_str`. Use this for precise, targeted edits instead of rewriting the whole file.

Args:
  file_path: Path to the file to edit.
  old_str: The exact existing text to replace (must match exactly).
  new_str: The replacement text.
  explanation: Brief explanation of what is being changed and why."#;

pub fn spec() -> ToolSpec {
    ToolSpec::new(
        NAME,
        DESCRIPTION,
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string" },
                "old_str": { "type": "string", "description": "The exact text to replace" },
                "new_str": { "type": "string", "description": "The replacement text" },
                "explanation": { "type": "string" }
            },
            "required": ["file_path", "old_str", "new_str"],
            "additionalProperties": false
        }),
    )
}

pub struct EditFileTool;

#[async_trait::async_trait]
impl super::ToolRuntime for EditFileTool {
    fn name(&self) -> &str { NAME }
    fn spec(&self) -> ToolSpec { spec() }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<String, ToolError> {
        if !ctx.can_write {
            return Err(ToolError::PermissionDenied("write is disabled".into()));
        }
        let file_path = args
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("missing 'file_path'".into()))?;
        let old_str = args
            .get("old_str")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("missing 'old_str'".into()))?;
        let new_str = args
            .get("new_str")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("missing 'new_str'".into()))?;
        let path = ctx.cwd.join(file_path);
        info!(?path, "edit_file");
        let content = fs::read_to_string(&path).await.map_err(ToolError::Io)?;
        if !content.contains(old_str) {
            return Err(ToolError::Other(format!(
                "old_str not found in {}",
                path.display()
            )));
        }
        let new_content = content.replace(old_str, new_str);
        fs::write(&path, &new_content).await.map_err(ToolError::Io)?;
        Ok(format!("replaced 1 occurrence in {}", path.display()))
    }
}
