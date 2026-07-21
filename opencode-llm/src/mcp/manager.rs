//! MCP server manager — manages the lifecycle of one or more stdio MCP
//! server processes: spawn, initialize, discover tools, call tools, and
//! graceful shutdown with retry/recovery.

use std::collections::BTreeMap;
use std::future::Future;
use std::io;
use std::time::Duration;

use serde_json::Value as JsonValue;
use tokio::time::timeout;

use crate::mcp::jsonrpc::{
    default_initialize_params, mcp_tool_name, JsonRpcError, JsonRpcId, McpListResourcesParams,
    McpListResourcesResult, McpListToolsParams, McpReadResourceParams, McpReadResourceResult,
    McpTool, McpToolCallContent, McpToolCallParams,
};
use crate::mcp::stdio::McpStdioProcess;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const MCP_INITIALIZE_TIMEOUT_MS: u64 = 10_000;
const MCP_LIST_TOOLS_TIMEOUT_MS: u64 = 30_000;
const MCP_TOOL_CALL_TIMEOUT_MS: u64 = 60_000;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Describes a single MCP server connection.
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Human-readable server name.
    pub server_name: String,
    /// Command to spawn.
    pub command: String,
    /// Command arguments.
    pub args: Vec<String>,
    /// Environment variables.
    pub env: BTreeMap<String, String>,
    /// Whether this server is required for the application to work.
    pub required: bool,
}

/// A tool discovered from an MCP server.
#[derive(Debug, Clone)]
pub struct ManagedMcpTool {
    /// Server that owns this tool.
    pub server_name: String,
    /// Fully qualified name (e.g. `mcp__my-server__read_file`).
    pub qualified_name: String,
    /// Raw tool name as advertised by the server.
    pub raw_name: String,
    /// The tool descriptor.
    pub tool: McpTool,
}

/// Outcome of a tool call.
#[derive(Debug, Clone)]
pub struct McpToolCallOutcome {
    /// Text content extracted from the response.
    pub text: String,
    /// Whether the tool call resulted in an error.
    pub is_error: bool,
    /// Raw JSON-RPC response content.
    pub raw_content: Vec<McpToolCallContent>,
}

/// Result of a tool discovery batch.
#[derive(Debug, Clone, Default)]
pub struct McpToolDiscoveryReport {
    /// Successfully discovered tools.
    pub tools: Vec<ManagedMcpTool>,
    /// Servers that failed during discovery.
    pub failed_servers: Vec<McpDiscoveryFailure>,
}

/// Information about a failed MCP server.
#[derive(Debug, Clone)]
pub struct McpDiscoveryFailure {
    /// Server name.
    pub server_name: String,
    /// Error message.
    pub error: String,
    /// Whether the server was required.
    pub required: bool,
}

/// Errors from the MCP server manager.
#[derive(Debug)]
pub enum McpManagerError {
    /// I/O transport error.
    Io(io::Error),
    /// Transport error specific to a server.
    Transport {
        server_name: String,
        method: &'static str,
        source: io::Error,
    },
    /// JSON-RPC error response.
    JsonRpc {
        server_name: String,
        method: &'static str,
        error: JsonRpcError,
    },
    /// Invalid response from server.
    InvalidResponse {
        server_name: String,
        method: &'static str,
        details: String,
    },
    /// Request timed out.
    Timeout {
        server_name: String,
        method: &'static str,
        timeout_ms: u64,
    },
    /// Unknown tool.
    UnknownTool { qualified_name: String },
    /// Unknown server.
    UnknownServer { server_name: String },
}

impl std::fmt::Display for McpManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Transport {
                server_name,
                method,
                source,
            } => {
                write!(
                    f,
                    "MCP `{server_name}` transport failed during {method}: {source}"
                )
            }
            Self::JsonRpc {
                server_name,
                method,
                error,
            } => {
                write!(
                    f,
                    "MCP `{server_name}` JSON-RPC error for {method}: {} ({})",
                    error.message, error.code
                )
            }
            Self::InvalidResponse {
                server_name,
                method,
                details,
            } => {
                write!(
                    f,
                    "MCP `{server_name}` invalid response for {method}: {details}"
                )
            }
            Self::Timeout {
                server_name,
                method,
                timeout_ms,
            } => {
                write!(
                    f,
                    "MCP `{server_name}` timed out after {timeout_ms}ms during {method}"
                )
            }
            Self::UnknownTool { qualified_name } => {
                write!(f, "unknown MCP tool `{qualified_name}`")
            }
            Self::UnknownServer { server_name } => {
                write!(f, "unknown MCP server `{server_name}`")
            }
        }
    }
}

impl std::error::Error for McpManagerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Transport { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for McpManagerError {
    fn from(v: io::Error) -> Self {
        Self::Io(v)
    }
}

// ---------------------------------------------------------------------------
// Internal server state
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct ManagedMcpServer {
    config: McpServerConfig,
    process: Option<McpStdioProcess>,
    initialized: bool,
}

impl ManagedMcpServer {
    fn new(config: McpServerConfig) -> Self {
        Self {
            config,
            process: None,
            initialized: false,
        }
    }
}

#[derive(Debug, Clone)]
struct ToolRoute {
    server_name: String,
    raw_name: String,
}

// ---------------------------------------------------------------------------
// McpServerManager
// ---------------------------------------------------------------------------

/// Manages a collection of MCP stdio server processes.
///
/// Provides methods to discover tools, call tools, list/read resources, and
/// gracefully shut down servers. Includes automatic retry and recovery for
/// transient transport errors.
#[derive(Debug)]
pub struct McpServerManager {
    servers: BTreeMap<String, ManagedMcpServer>,
    tool_index: BTreeMap<String, ToolRoute>,
    next_request_id: u64,
}

impl McpServerManager {
    /// Create a new manager from a list of server configs.
    pub fn new(configs: Vec<McpServerConfig>) -> Self {
        let mut servers = BTreeMap::new();
        for cfg in configs {
            servers.insert(cfg.server_name.clone(), ManagedMcpServer::new(cfg));
        }
        Self {
            servers,
            tool_index: BTreeMap::new(),
            next_request_id: 1,
        }
    }

    /// Return the list of configured server names.
    pub fn server_names(&self) -> Vec<String> {
        self.servers.keys().cloned().collect()
    }

    // -----------------------------------------------------------------------
    // Tool discovery
    // -----------------------------------------------------------------------

    /// Discover tools from all configured servers. Fails fast on first error.
    pub async fn discover_tools(&mut self) -> Result<Vec<ManagedMcpTool>, McpManagerError> {
        let names = self.server_names();
        let mut all = Vec::new();
        for name in names {
            let tools = self.discover_tools_for_server(&name).await?;
            self.clear_routes(&name);
            for tool in tools {
                self.tool_index.insert(
                    tool.qualified_name.clone(),
                    ToolRoute {
                        server_name: tool.server_name.clone(),
                        raw_name: tool.raw_name.clone(),
                    },
                );
                all.push(tool);
            }
        }
        Ok(all)
    }

    /// Discover tools from all servers, collecting per-server errors instead
    /// of failing fast.
    pub async fn discover_tools_best_effort(&mut self) -> McpToolDiscoveryReport {
        let names = self.server_names();
        let mut tools = Vec::new();
        let mut failed = Vec::new();

        for name in names {
            match self.discover_tools_for_server(&name).await {
                Ok(server_tools) => {
                    self.clear_routes(&name);
                    for tool in server_tools {
                        self.tool_index.insert(
                            tool.qualified_name.clone(),
                            ToolRoute {
                                server_name: tool.server_name.clone(),
                                raw_name: tool.raw_name.clone(),
                            },
                        );
                        tools.push(tool);
                    }
                }
                Err(e) => {
                    self.clear_routes(&name);
                    let required = self
                        .servers
                        .get(&name)
                        .map(|s| s.config.required)
                        .unwrap_or(false);
                    failed.push(McpDiscoveryFailure {
                        server_name: name,
                        error: e.to_string(),
                        required,
                    });
                }
            }
        }

        McpToolDiscoveryReport {
            tools,
            failed_servers: failed,
        }
    }

    /// Discover tools from a single server.
    async fn discover_tools_for_server(
        &mut self,
        server_name: &str,
    ) -> Result<Vec<ManagedMcpTool>, McpManagerError> {
        let mut attempts = 0usize;
        loop {
            match self.discover_tools_once(server_name).await {
                Ok(tools) => return Ok(tools),
                Err(e) if attempts == 0 && is_retryable(&e) => {
                    self.reset_server(server_name).await?;
                    attempts += 1;
                }
                Err(e) => {
                    if should_reset(&e) {
                        let _ = self.reset_server(server_name).await;
                    }
                    return Err(e);
                }
            }
        }
    }

    async fn discover_tools_once(
        &mut self,
        server_name: &str,
    ) -> Result<Vec<ManagedMcpTool>, McpManagerError> {
        self.ensure_server_ready(server_name).await?;

        let mut cursor = None;
        let mut discovered = Vec::new();

        loop {
            let id = self.take_id();
            let response =
                {
                    let server = self.server_mut(server_name)?;
                    let process = server.process.as_mut().ok_or_else(|| {
                        McpManagerError::InvalidResponse {
                            server_name: server_name.to_string(),
                            method: "tools/list",
                            details: "process missing".to_string(),
                        }
                    })?;
                    Self::run_with_timeout(
                        server_name,
                        "tools/list",
                        MCP_LIST_TOOLS_TIMEOUT_MS,
                        process.list_tools(
                            id,
                            Some(McpListToolsParams {
                                cursor: cursor.clone(),
                            }),
                        ),
                    )
                    .await?
                };

            if let Some(error) = response.error {
                return Err(McpManagerError::JsonRpc {
                    server_name: server_name.to_string(),
                    method: "tools/list",
                    error,
                });
            }

            let result = response
                .result
                .ok_or_else(|| McpManagerError::InvalidResponse {
                    server_name: server_name.to_string(),
                    method: "tools/list",
                    details: "missing result".to_string(),
                })?;

            for tool in result.tools {
                let qualified = mcp_tool_name(server_name, &tool.name);
                discovered.push(ManagedMcpTool {
                    server_name: server_name.to_string(),
                    qualified_name: qualified,
                    raw_name: tool.name.clone(),
                    tool,
                });
            }

            match result.next_cursor {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }

        Ok(discovered)
    }

    // -----------------------------------------------------------------------
    // Tool calling
    // -----------------------------------------------------------------------

    /// Call a tool by its qualified name.
    pub async fn call_tool(
        &mut self,
        qualified_name: &str,
        arguments: Option<JsonValue>,
    ) -> Result<McpToolCallOutcome, McpManagerError> {
        let route = self
            .tool_index
            .get(qualified_name)
            .cloned()
            .ok_or_else(|| McpManagerError::UnknownTool {
                qualified_name: qualified_name.to_string(),
            })?;

        let timeout_ms = MCP_TOOL_CALL_TIMEOUT_MS;
        self.ensure_server_ready(&route.server_name).await?;
        let id = self.take_id();

        let response = {
            let server = self.server_mut(&route.server_name)?;
            let process =
                server
                    .process
                    .as_mut()
                    .ok_or_else(|| McpManagerError::InvalidResponse {
                        server_name: route.server_name.clone(),
                        method: "tools/call",
                        details: "process missing".to_string(),
                    })?;
            Self::run_with_timeout(
                &route.server_name,
                "tools/call",
                timeout_ms,
                process.call_tool(
                    id,
                    McpToolCallParams {
                        name: route.raw_name,
                        arguments,
                        meta: None,
                    },
                ),
            )
            .await
        };

        // Recover on transport/timeout errors.
        if let Err(e) = &response {
            if should_reset(e) {
                let _ = self.reset_server(&route.server_name).await;
            }
        }

        let response = response?;

        if let Some(error) = response.error {
            return Err(McpManagerError::JsonRpc {
                server_name: route.server_name,
                method: "tools/call",
                error,
            });
        }

        let result = response
            .result
            .ok_or_else(|| McpManagerError::InvalidResponse {
                server_name: route.server_name,
                method: "tools/call",
                details: "missing result".to_string(),
            })?;

        let text = result
            .content
            .iter()
            .filter(|c| c.kind == "text")
            .map(|c| c.data.get("text").and_then(|v| v.as_str()).unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(McpToolCallOutcome {
            text,
            is_error: result.is_error.unwrap_or(false),
            raw_content: result.content,
        })
    }

    // -----------------------------------------------------------------------
    // Resources
    // -----------------------------------------------------------------------

    /// List resources from a server.
    pub async fn list_resources(
        &mut self,
        server_name: &str,
    ) -> Result<McpListResourcesResult, McpManagerError> {
        let mut attempts = 0usize;
        loop {
            match self.list_resources_once(server_name).await {
                Ok(r) => return Ok(r),
                Err(e) if attempts == 0 && is_retryable(&e) => {
                    self.reset_server(server_name).await?;
                    attempts += 1;
                }
                Err(e) => {
                    if should_reset(&e) {
                        let _ = self.reset_server(server_name).await;
                    }
                    return Err(e);
                }
            }
        }
    }

    async fn list_resources_once(
        &mut self,
        server_name: &str,
    ) -> Result<McpListResourcesResult, McpManagerError> {
        self.ensure_server_ready(server_name).await?;

        let mut resources = Vec::new();
        let mut cursor = None;

        loop {
            let id = self.take_id();
            let response =
                {
                    let server = self.server_mut(server_name)?;
                    let process = server.process.as_mut().ok_or_else(|| {
                        McpManagerError::InvalidResponse {
                            server_name: server_name.to_string(),
                            method: "resources/list",
                            details: "process missing".to_string(),
                        }
                    })?;
                    Self::run_with_timeout(
                        server_name,
                        "resources/list",
                        MCP_LIST_TOOLS_TIMEOUT_MS,
                        process.list_resources(
                            id,
                            Some(McpListResourcesParams {
                                cursor: cursor.clone(),
                            }),
                        ),
                    )
                    .await?
                };

            if let Some(error) = response.error {
                return Err(McpManagerError::JsonRpc {
                    server_name: server_name.to_string(),
                    method: "resources/list",
                    error,
                });
            }

            let result = response
                .result
                .ok_or_else(|| McpManagerError::InvalidResponse {
                    server_name: server_name.to_string(),
                    method: "resources/list",
                    details: "missing result".to_string(),
                })?;

            resources.extend(result.resources);

            match result.next_cursor {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }

        Ok(McpListResourcesResult {
            resources,
            next_cursor: None,
        })
    }

    /// Read a resource.
    pub async fn read_resource(
        &mut self,
        server_name: &str,
        uri: &str,
    ) -> Result<McpReadResourceResult, McpManagerError> {
        let mut attempts = 0usize;
        loop {
            match self.read_resource_once(server_name, uri).await {
                Ok(r) => return Ok(r),
                Err(e) if attempts == 0 && is_retryable(&e) => {
                    self.reset_server(server_name).await?;
                    attempts += 1;
                }
                Err(e) => {
                    if should_reset(&e) {
                        let _ = self.reset_server(server_name).await;
                    }
                    return Err(e);
                }
            }
        }
    }

    async fn read_resource_once(
        &mut self,
        server_name: &str,
        uri: &str,
    ) -> Result<McpReadResourceResult, McpManagerError> {
        self.ensure_server_ready(server_name).await?;

        let id = self.take_id();
        let response = {
            let server = self.server_mut(server_name)?;
            let process =
                server
                    .process
                    .as_mut()
                    .ok_or_else(|| McpManagerError::InvalidResponse {
                        server_name: server_name.to_string(),
                        method: "resources/read",
                        details: "process missing".to_string(),
                    })?;
            Self::run_with_timeout(
                server_name,
                "resources/read",
                MCP_LIST_TOOLS_TIMEOUT_MS,
                process.read_resource(
                    id,
                    McpReadResourceParams {
                        uri: uri.to_string(),
                    },
                ),
            )
            .await?
        };

        if let Some(error) = response.error {
            return Err(McpManagerError::JsonRpc {
                server_name: server_name.to_string(),
                method: "resources/read",
                error,
            });
        }

        response
            .result
            .ok_or_else(|| McpManagerError::InvalidResponse {
                server_name: server_name.to_string(),
                method: "resources/read",
                details: "missing result".to_string(),
            })
    }

    // -----------------------------------------------------------------------
    // Shutdown
    // -----------------------------------------------------------------------

    /// Gracefully shut down all managed servers.
    pub async fn shutdown(&mut self) {
        let names: Vec<_> = self.servers.keys().cloned().collect();
        for name in names {
            if let Some(server) = self.servers.get_mut(&name) {
                if let Some(process) = server.process.as_mut() {
                    let _ = process.shutdown().await;
                }
                server.process = None;
                server.initialized = false;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn clear_routes(&mut self, server_name: &str) {
        self.tool_index.retain(|_, r| r.server_name != server_name);
    }

    fn server_mut(&mut self, name: &str) -> Result<&mut ManagedMcpServer, McpManagerError> {
        self.servers
            .get_mut(name)
            .ok_or_else(|| McpManagerError::UnknownServer {
                server_name: name.to_string(),
            })
    }

    fn take_id(&mut self) -> JsonRpcId {
        let id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);
        JsonRpcId::Number(id)
    }

    async fn reset_server(&mut self, server_name: &str) -> Result<(), McpManagerError> {
        let mut process = {
            let server = self.server_mut(server_name)?;
            server.initialized = false;
            server.process.take()
        };
        if let Some(ref mut p) = process {
            let _ = p.shutdown().await;
        }
        Ok(())
    }

    async fn ensure_server_ready(&mut self, server_name: &str) -> Result<(), McpManagerError> {
        // Check if process exited.
        let exited = {
            let server = self.servers.get_mut(server_name).ok_or_else(|| {
                McpManagerError::UnknownServer {
                    server_name: server_name.to_string(),
                }
            })?;
            match server.process.as_mut() {
                Some(p) => p.has_exited()?,
                None => false,
            }
        };
        if exited {
            self.reset_server(server_name).await?;
        }

        let mut attempts = 0usize;
        loop {
            let needs_spawn = self
                .servers
                .get(server_name)
                .map(|s| s.process.is_none())
                .ok_or_else(|| McpManagerError::UnknownServer {
                    server_name: server_name.to_string(),
                })?;

            if needs_spawn {
                let cfg = {
                    let s = self.server_mut(server_name)?;
                    s.config.clone()
                };
                let process =
                    McpStdioProcess::spawn(&cfg.command, &cfg.args, &cfg.env).map_err(|e| {
                        McpManagerError::Transport {
                            server_name: server_name.to_string(),
                            method: "spawn",
                            source: e,
                        }
                    })?;
                let s = self.server_mut(server_name)?;
                s.process = Some(process);
                s.initialized = false;
            }

            let needs_init = self
                .servers
                .get(server_name)
                .map(|s| !s.initialized)
                .unwrap_or(false);

            if !needs_init {
                return Ok(());
            }

            let id = self.take_id();
            let response =
                {
                    let server = self.server_mut(server_name)?;
                    let process = server.process.as_mut().ok_or_else(|| {
                        McpManagerError::InvalidResponse {
                            server_name: server_name.to_string(),
                            method: "initialize",
                            details: "process missing before init".to_string(),
                        }
                    })?;
                    Self::run_with_timeout(
                        server_name,
                        "initialize",
                        MCP_INITIALIZE_TIMEOUT_MS,
                        process.initialize(id, default_initialize_params()),
                    )
                    .await
                };

            let response = match response {
                Ok(r) => r,
                Err(e) if attempts == 0 && is_retryable(&e) => {
                    self.reset_server(server_name).await?;
                    attempts += 1;
                    continue;
                }
                Err(e) => {
                    if should_reset(&e) {
                        let _ = self.reset_server(server_name).await;
                    }
                    return Err(e);
                }
            };

            if let Some(error) = response.error {
                return Err(McpManagerError::JsonRpc {
                    server_name: server_name.to_string(),
                    method: "initialize",
                    error,
                });
            }

            if response.result.is_none() {
                self.reset_server(server_name).await?;
                return Err(McpManagerError::InvalidResponse {
                    server_name: server_name.to_string(),
                    method: "initialize",
                    details: "missing result".to_string(),
                });
            }

            let s = self.server_mut(server_name)?;
            s.initialized = true;
            return Ok(());
        }
    }

    async fn run_with_timeout<T, F>(
        server_name: &str,
        method: &'static str,
        timeout_ms: u64,
        future: F,
    ) -> Result<T, McpManagerError>
    where
        F: Future<Output = io::Result<T>>,
    {
        match timeout(Duration::from_millis(timeout_ms), future).await {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(e)) if e.kind() == io::ErrorKind::InvalidData => {
                Err(McpManagerError::InvalidResponse {
                    server_name: server_name.to_string(),
                    method,
                    details: e.to_string(),
                })
            }
            Ok(Err(source)) => Err(McpManagerError::Transport {
                server_name: server_name.to_string(),
                method,
                source,
            }),
            Err(_) => Err(McpManagerError::Timeout {
                server_name: server_name.to_string(),
                method,
                timeout_ms,
            }),
        }
    }
}

fn is_retryable(e: &McpManagerError) -> bool {
    matches!(
        e,
        McpManagerError::Transport { .. } | McpManagerError::Timeout { .. }
    )
}

fn should_reset(e: &McpManagerError) -> bool {
    matches!(
        e,
        McpManagerError::Transport { .. }
            | McpManagerError::Timeout { .. }
            | McpManagerError::InvalidResponse { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_manager_has_no_servers_by_default() {
        let mgr = McpServerManager::new(vec![]);
        assert!(mgr.server_names().is_empty());
    }

    #[test]
    fn new_manager_with_config() {
        let cfg = McpServerConfig {
            server_name: "test".to_string(),
            command: "echo".to_string(),
            args: vec![],
            env: BTreeMap::new(),
            required: false,
        };
        let mgr = McpServerManager::new(vec![cfg]);
        assert_eq!(mgr.server_names(), vec!["test"]);
    }
}
