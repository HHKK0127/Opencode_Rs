//! Bash command execution tool.
//!
//! Runs shell commands on the local machine. Execution is guarded by the
//! [`ToolContext::can_exec_shell`] flag and a configurable timeout.

use serde_json::Value;
use tokio::process::Command;
use tracing::info;
use super::{ToolContext, ToolError, ToolSpec};
pub static NAME: &str = "bash";
static DESCRIPTION: &str = r#"Execute a shell command on the local machine. The command runs in a sub-process with a configurable timeout. Use this to run code, install packages, compile, test, or perform any shell operation within the workspace directory.

**Security**: Only available when the tool policy grants shell execution permission.

Args:
  command: The shell command to execute (use && to chain commands, ; to run regardless of exit code)."#;

pub fn spec() -> ToolSpec {
    ToolSpec::new(
        NAME,
        DESCRIPTION,
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute."
                }
            },
            "required": ["command"],
            "additionalProperties": false
        }),
    )
}

pub struct BashTool;

#[async_trait::async_trait]
impl super::ToolRuntime for BashTool {
    fn name(&self) -> &str {
        NAME
    }

    fn spec(&self) -> ToolSpec {
        spec()
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<String, ToolError> {
        if !ctx.can_exec_shell {
            return Err(ToolError::PermissionDenied(
                "shell execution is disabled".into(),
            ));
        }

        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("missing 'command' field".into()))?;

        info!(command, "bash tool executing");

        let child = Command::new(if cfg!(target_os = "windows") {
            "cmd.exe"
        } else {
            "sh"
        })
        .arg(if cfg!(target_os = "windows") {
            "/C"
        } else {
            "-c"
        })
        .arg(command)
        .current_dir(&ctx.cwd)
        .kill_on_drop(true)
        .output();

        let output = if let Some(timeout) = ctx.timeout {
            tokio::time::timeout(timeout, child)
                .await
                .map_err(|_| ToolError::Timeout(timeout))?
        } else {
            child.await
        }
        .map_err(ToolError::Io)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let mut result = String::new();
        if output.status.success() {
            if !stdout.is_empty() {
                result.push_str(&stdout);
            }
            if !stderr.is_empty() {
                result.push_str("\n--- stderr ---\n");
                result.push_str(&stderr);
            }
        } else {
            if !stdout.is_empty() {
                result.push_str(&stdout);
                result.push('\n');
            }
            result.push_str(&format!("exit code: {}", output.status.code().unwrap_or(-1)));
            if !stderr.is_empty() {
                result.push_str("\n--- stderr ---\n");
                result.push_str(&stderr);
            }
        }

        if result.is_empty() {
            result = "(command produced no output)".to_string();
        }

        Ok(result)
    }
}
