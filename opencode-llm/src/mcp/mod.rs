//! # MCP (Model Context Protocol) stdio client
//!
//! Implements the MCP stdio transport layer, enabling OpenCode to connect
//! to external MCP servers (e.g. filesystem, database, API) and expose
//! their tools as first-class tools in the conversation runtime.
//!
//! Architecture (inspired by `claw-code`'s MCP implementation):
//!
//! ```text
//! ┌──────────────────────────────────────────┐
//! │  McpServerManager                       │  ← lifecycle + discovery + invocation
//! │  - discover_tools / discover_tools_best_effort │
//! │  - call_tool                            │
//! │  - list_resources / read_resource       │
//! └──────────────┬──────────────────────────┘
//!                │ McpStdioProcess
//! ┌──────────────▼──────────────────────────┐
//! │  stdio::McpStdioProcess                │  ← stdin/stdout frame transport
//! │  - write_frame / read_frame            │
//! │  - request / initialize / call_tool    │
//! └──────────────┬──────────────────────────┘
//!                │ JSON-RPC 2.0
//! ┌──────────────▼──────────────────────────┐
//! │  jsonrpc                                │  ← wire format types
//! │  - JsonRpcRequest / JsonRpcResponse     │
//! │  - McpTool / McpToolCallResult / ...    │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Quick start
//!
//! ```ignore
//! use opencode_llm::mcp::manager::{McpServerManager, McpServerConfig};
//! use std::collections::BTreeMap;
//!
//! let cfg = McpServerConfig {
//!     server_name: "fs".to_string(),
//!     command: "npx".to_string(),
//!     args: vec!["-y".into(), "@modelcontextprotocol/server-filesystem".into(), "/tmp".into()],
//!     env: BTreeMap::new(),
//!     required: false,
//! };
//! let mut mgr = McpServerManager::new(vec![cfg]);
//! let report = mgr.discover_tools_best_effort().await;
//! println!("Discovered {} tools", report.tools.len());
//! ```

pub mod bridge;
pub mod jsonrpc;
pub mod manager;
pub mod stdio;

pub use bridge::{build_mcp_tools, McpToolBridge};
pub use jsonrpc::{
    default_initialize_params, mcp_tool_name, mcp_tool_prefix, normalize_name,
    JsonRpcError, JsonRpcId, JsonRpcRequest, JsonRpcResponse, McpInitializeClientInfo,
    McpInitializeParams, McpInitializeResult, McpInitializeServerInfo, McpListResourcesParams,
    McpListResourcesResult, McpReadResourceParams, McpReadResourceResult, McpResource,
    McpResourceContents, McpListToolsParams, McpListToolsResult, McpTool, McpToolCallContent,
    McpToolCallParams, McpToolCallResult,
};
pub use manager::{
    McpDiscoveryFailure, McpManagerError, McpServerConfig, McpServerManager,
    McpToolCallOutcome, McpToolDiscoveryReport, ManagedMcpTool,
};
pub use stdio::McpStdioProcess;
