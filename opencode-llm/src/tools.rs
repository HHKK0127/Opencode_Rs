//! Tool execution framework.
//!
//! The 6 built-in tools exposed by [`mvp_tool_specs`] cover the most common
//! actions an AI coding agent needs: shell, file I/O, search, and a couple of
//! web tools. More tools can be added by constructing new [`ToolSpec`] values
//! and passing them to the runtime builder.

pub mod bash;
pub mod edit_file;
pub mod glob_search;
pub mod grep_search;
pub mod read_file;
pub mod web_fetch;
pub mod web_search;
pub mod write_file;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::error::LlmError;
use crate::types::ToolDefinition;

/// A boxed tool executor (cheap to clone, shareable across threads).
pub type ToolExecutor = Arc<dyn ToolRuntime>;

/// Tool error type. Maps to a tool_result block in the conversation.
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    /// Invalid arguments supplied to the tool.
    #[error("invalid arguments: {0}")]
    InvalidArgs(String),
    /// I/O failure during tool execution.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// Execution timed out.
    #[error("execution timed out after {0:?}")]
    Timeout(std::time::Duration),
    /// Permission denied by the active policy.
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    /// Generic execution error.
    #[error("tool error: {0}")]
    Other(String),
}

impl From<ToolError> for LlmError {
    fn from(e: ToolError) -> Self {
        LlmError::Tool(e.to_string())
    }
}

/// Execution context passed to a tool. Carries the working directory and any
/// approved allow-list.
#[derive(Debug, Clone, Default)]
pub struct ToolContext {
    /// Working directory for the tool. All relative paths resolve against this.
    pub cwd: PathBuf,
    /// Whether the tool may write files.
    pub can_write: bool,
    /// Whether the tool may execute arbitrary shell commands.
    pub can_exec_shell: bool,
    /// Maximum duration before a tool execution is cancelled.
    pub timeout: Option<std::time::Duration>,
}

impl ToolContext {
    /// Construct a context rooted at `cwd` with full permissions.
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self {
            cwd: cwd.into(),
            can_write: true,
            can_exec_shell: true,
            timeout: Some(std::time::Duration::from_secs(60)),
        }
    }

    /// Construct a read-only context.
    pub fn read_only(cwd: impl Into<PathBuf>) -> Self {
        Self {
            cwd: cwd.into(),
            can_write: false,
            can_exec_shell: false,
            timeout: Some(std::time::Duration::from_secs(30)),
        }
    }
}

/// A tool spec — the metadata sent to the provider so the model knows about it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    /// Tool name (e.g. `bash`).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// JSON schema for the tool's input.
    pub input_schema: Value,
}

impl ToolSpec {
    /// Construct a tool spec.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }

    /// Convert to the provider-facing [`ToolDefinition`].
    pub fn to_definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema: self.input_schema.clone(),
        }
    }
}

/// Tool runtime trait. Implementors execute the tool's logic.
#[async_trait]
pub trait ToolRuntime: Send + Sync {
    /// The tool's name.
    fn name(&self) -> &str;
    /// The tool's spec (sent to the provider).
    fn spec(&self) -> ToolSpec;
    /// Execute the tool with the given arguments.
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<String, ToolError>;
}

/// Build the default set of tool specs. This is the list advertised to the
/// model on every turn.
pub fn mvp_tool_specs() -> Vec<ToolSpec> {
    vec![
        bash::spec(),
        read_file::spec(),
        write_file::spec(),
        edit_file::spec(),
        glob_search::spec(),
        grep_search::spec(),
        web_fetch::spec(),
        web_search::spec(),
    ]
}

/// Build the default tool executors keyed by name.
pub fn mvp_tool_executors() -> BTreeMap<String, ToolExecutor> {
    let mut m: BTreeMap<String, ToolExecutor> = BTreeMap::new();
    m.insert(bash::NAME.into(), Arc::new(bash::BashTool));
    m.insert(read_file::NAME.into(), Arc::new(read_file::ReadFileTool));
    m.insert(write_file::NAME.into(), Arc::new(write_file::WriteFileTool));
    m.insert(edit_file::NAME.into(), Arc::new(edit_file::EditFileTool));
    m.insert(glob_search::NAME.into(), Arc::new(glob_search::GlobSearchTool));
    m.insert(grep_search::NAME.into(), Arc::new(grep_search::GrepSearchTool));
    m.insert(web_fetch::NAME.into(), Arc::new(web_fetch::WebFetchTool));
    m.insert(web_search::NAME.into(), Arc::new(web_search::WebSearchTool));
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mvp_tool_specs_includes_core_tools() {
        let specs = mvp_tool_specs();
        let names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"bash"));
        assert!(names.contains(&"read_file"));
        assert!(names.contains(&"write_file"));
        assert!(names.contains(&"edit_file"));
        assert!(names.contains(&"glob_search"));
        assert!(names.contains(&"grep_search"));
    }
}
