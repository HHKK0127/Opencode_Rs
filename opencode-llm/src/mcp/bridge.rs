//! Bridge between MCP tools and the opencode-llm `ToolSpec`/`ToolExecutor` system.
//!
//! Allows MCP-discovered tools to be used as native tools in the conversation
//! runtime alongside the built-in MVP tools.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value as JsonValue;
use tokio::sync::Mutex;

use crate::mcp::manager::{ManagedMcpTool, McpManagerError, McpServerManager};
use crate::tools::{ToolContext, ToolError, ToolExecutor, ToolRuntime, ToolSpec};

/// Wraps an MCP tool as a [`ToolRuntime`] so it can be registered in the
/// conversation runtime's executor map.
///
/// The tool name is the *qualified* MCP name
/// (e.g. `mcp__my-server__read_file`).
pub struct McpToolBridge {
    /// Qualified tool name (mcp__<server>__<tool>).
    qualified_name: String,
    /// The tool spec for the LLM provider.
    spec: ToolSpec,
    /// Shared reference to the server manager.
    manager: Arc<Mutex<McpServerManager>>,
}

impl McpToolBridge {
    /// Create a new bridge for a managed MCP tool.
    pub fn new(tool: &ManagedMcpTool, manager: Arc<Mutex<McpServerManager>>) -> Self {
        let input_schema = tool
            .tool
            .input_schema
            .clone()
            .unwrap_or(JsonValue::Object(serde_json::Map::new()));

        Self {
            qualified_name: tool.qualified_name.clone(),
            spec: ToolSpec::new(
                &tool.qualified_name,
                tool.tool
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("MCP tool from {}", tool.server_name)),
                input_schema,
            ),
            manager,
        }
    }
}

#[async_trait]
impl ToolRuntime for McpToolBridge {
    fn name(&self) -> &str {
        &self.qualified_name
    }

    fn spec(&self) -> ToolSpec {
        self.spec.clone()
    }

    async fn execute(&self, args: JsonValue, _ctx: &ToolContext) -> Result<String, ToolError> {
        let mut mgr = self.manager.lock().await;

        let outcome = mgr
            .call_tool(&self.qualified_name, Some(args))
            .await
            .map_err(|e| map_error(e, &self.qualified_name))?;

        if outcome.is_error {
            return Err(ToolError::Other(format!(
                "MCP tool `{}` returned error: {}",
                self.qualified_name, outcome.text
            )));
        }

        Ok(outcome.text)
    }
}

/// Build a [`ToolSpec`] and [`ToolExecutor`] for each managed MCP tool.
pub fn build_mcp_tools(
    tools: &[ManagedMcpTool],
    manager: Arc<Mutex<McpServerManager>>,
) -> Vec<(ToolSpec, ToolExecutor)> {
    tools
        .iter()
        .map(|tool| {
            let bridge = McpToolBridge::new(tool, Arc::clone(&manager));
            let spec = bridge.spec();
            let executor = Arc::new(bridge) as ToolExecutor;
            (spec, executor)
        })
        .collect()
}

fn map_error(e: McpManagerError, qualified_name: &str) -> ToolError {
    match e {
        McpManagerError::Timeout { .. } => ToolError::Timeout(std::time::Duration::from_secs(60)),
        McpManagerError::UnknownTool { .. } => {
            ToolError::Other(format!("MCP tool `{qualified_name}` not found in index"))
        }
        _ => ToolError::Other(format!("MCP tool `{qualified_name}` failed: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::jsonrpc::McpTool;

    fn make_managed_tool() -> ManagedMcpTool {
        ManagedMcpTool {
            server_name: "test-server".to_string(),
            qualified_name: "mcp__test-server__my_tool".to_string(),
            raw_name: "my_tool".to_string(),
            tool: McpTool {
                name: "my_tool".to_string(),
                description: Some("A test tool".to_string()),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": { "input": { "type": "string" } }
                })),
                annotations: None,
                meta: None,
            },
        }
    }

    #[test]
    fn bridge_creates_tool_spec() {
        let mgr = Arc::new(Mutex::new(McpServerManager::new(vec![])));
        let tool = make_managed_tool();
        let bridge = McpToolBridge::new(&tool, mgr);
        assert_eq!(bridge.name(), "mcp__test-server__my_tool");
        assert_eq!(bridge.spec().name, "mcp__test-server__my_tool");
    }

    #[test]
    fn build_mcp_tools_produces_vec() {
        let mgr = Arc::new(Mutex::new(McpServerManager::new(vec![])));
        let tools = vec![make_managed_tool()];
        let pairs = build_mcp_tools(&tools, mgr);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0.name, "mcp__test-server__my_tool");
    }
}
