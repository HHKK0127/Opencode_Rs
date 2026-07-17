//! Read file contents tool.

use serde_json::Value;
use tokio::fs;
use tracing::info;

use super::{ToolContext, ToolError, ToolSpec};

pub static NAME: &str = "read_file";
static DESCRIPTION: &str = r#"Read the contents of a file. Returns the full file content as a string.

Args:
  file_path: Absolute path or path relative to the working directory of the file to read."#;

pub fn spec() -> ToolSpec {
    ToolSpec::new(
        NAME,
        DESCRIPTION,
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to read."
                }
            },
            "required": ["file_path"],
            "additionalProperties": false
        }),
    )
}

pub struct ReadFileTool;

#[async_trait::async_trait]
impl super::ToolRuntime for ReadFileTool {
    fn name(&self) -> &str { NAME }
    fn spec(&self) -> ToolSpec { spec() }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<String, ToolError> {
        let file_path = args
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("missing 'file_path'".into()))?;
        let path = ctx.cwd.join(file_path);
        info!(?path, "read_file");
        let content = fs::read_to_string(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ToolError::Other(format!("file not found: {}", path.display()))
            } else {
                ToolError::Io(e)
            }
        })?;
        Ok(content)
    }
}
