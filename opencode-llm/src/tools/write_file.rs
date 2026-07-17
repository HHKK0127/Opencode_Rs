//! Write file contents tool.

use serde_json::Value;
use tokio::fs;
use tracing::info;

use super::{ToolContext, ToolError, ToolSpec};

pub static NAME: &str = "write_file";
static DESCRIPTION: &str = r#"Write content to a file. Creates parent directories automatically. Overwrites the file if it already exists.

Args:
  file_path: Path to the file to write (relative to workspace or absolute).
  content: The content to write.
  explanation: Brief explanation of what is being changed and why (shown to the user)."#;

pub fn spec() -> ToolSpec {
    ToolSpec::new(
        NAME,
        DESCRIPTION,
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string" },
                "content": { "type": "string", "description": "The file content to write" },
                "explanation": { "type": "string" }
            },
            "required": ["file_path", "content"],
            "additionalProperties": false
        }),
    )
}

pub struct WriteFileTool;

#[async_trait::async_trait]
impl super::ToolRuntime for WriteFileTool {
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
        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("missing 'content'".into()))?;
        let path = ctx.cwd.join(file_path);
        info!(?path, "write_file");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(ToolError::Io)?;
        }
        fs::write(&path, content).await.map_err(ToolError::Io)?;
        Ok(format!("wrote {} bytes to {}", content.len(), path.display()))
    }
}
