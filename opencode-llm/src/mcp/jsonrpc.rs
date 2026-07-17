//! JSON-RPC 2.0 types and MCP protocol message types.
//!
//! Defines the wire format shared between the client and server processes.
//! Protocol version: "2025-03-26" (MCP specification).

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 base types
// ---------------------------------------------------------------------------

/// A JSON-RPC request identifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum JsonRpcId {
    /// Numeric id.
    Number(u64),
    /// String id.
    String(String),
    /// Null id (notification).
    Null,
}

impl From<u64> for JsonRpcId {
    fn from(v: u64) -> Self {
        Self::Number(v)
    }
}

/// A JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcRequest<T = JsonValue> {
    /// Always "2.0".
    pub jsonrpc: String,
    /// Request id.
    pub id: JsonRpcId,
    /// Method name.
    pub method: String,
    /// Optional parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<T>,
}

impl<T> JsonRpcRequest<T> {
    /// Build a new request.
    pub fn new(id: impl Into<JsonRpcId>, method: impl Into<String>, params: Option<T>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            method: method.into(),
            params,
        }
    }
}

/// A JSON-RPC error object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i64,
    /// Error message.
    pub message: String,
    /// Optional extra data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
}

/// A JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse<T = JsonValue> {
    /// Always "2.0".
    pub jsonrpc: String,
    /// Echo of the request id.
    pub id: JsonRpcId,
    /// Success payload (absent on error).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    /// Error payload (absent on success).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

// ---------------------------------------------------------------------------
// MCP handshake — initialize
// ---------------------------------------------------------------------------

/// Parameters for `initialize`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpInitializeParams {
    /// Protocol version string.
    pub protocol_version: String,
    /// Client capabilities.
    pub capabilities: JsonValue,
    /// Client identification.
    pub client_info: McpInitializeClientInfo,
}

/// Client info block inside `McpInitializeParams`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct McpInitializeClientInfo {
    /// Client name.
    pub name: String,
    /// Client version.
    pub version: String,
}

/// Result of `initialize`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpInitializeResult {
    /// Negotiated protocol version.
    pub protocol_version: String,
    /// Server capabilities.
    pub capabilities: JsonValue,
    /// Server identification.
    pub server_info: McpInitializeServerInfo,
}

/// Server info block inside `McpInitializeResult`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct McpInitializeServerInfo {
    /// Server name.
    pub name: String,
    /// Server version.
    pub version: String,
}

// ---------------------------------------------------------------------------
// MCP tools/lifecycle
// ---------------------------------------------------------------------------

/// Parameters for `tools/list`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpListToolsParams {
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// A tool advertised by an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpTool {
    /// Tool name.
    pub name: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema for the tool's input.
    #[serde(rename = "inputSchema", skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<JsonValue>,
    /// Extra annotations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<JsonValue>,
    /// Server-side metadata.
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<JsonValue>,
}

/// Result of `tools/list`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpListToolsResult {
    /// Available tools.
    pub tools: Vec<McpTool>,
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Parameters for `tools/call`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpToolCallParams {
    /// Tool name to invoke.
    pub name: String,
    /// Arguments for the tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<JsonValue>,
    /// Metadata.
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<JsonValue>,
}

/// A single content item in a tool-call result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpToolCallContent {
    /// Content type (e.g. "text", "image", "resource").
    #[serde(rename = "type")]
    pub kind: String,
    /// Extra fields (text, mimeType, etc.).
    #[serde(flatten)]
    pub data: BTreeMap<String, JsonValue>,
}

/// Result of `tools/call`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpToolCallResult {
    /// Content items.
    #[serde(default)]
    pub content: Vec<McpToolCallContent>,
    /// Structured content (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<JsonValue>,
    /// Whether the tool call resulted in an error.
    #[serde(default)]
    pub is_error: Option<bool>,
    /// Metadata.
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<JsonValue>,
}

// ---------------------------------------------------------------------------
// MCP resources
// ---------------------------------------------------------------------------

/// Parameters for `resources/list`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpListResourcesParams {
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// A resource advertised by an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpResource {
    /// Resource URI.
    pub uri: String,
    /// Optional human-readable name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Annotations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<JsonValue>,
    /// Metadata.
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<JsonValue>,
}

/// Result of `resources/list`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpListResourcesResult {
    /// Available resources.
    pub resources: Vec<McpResource>,
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Parameters for `resources/read`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpReadResourceParams {
    /// Resource URI.
    pub uri: String,
}

/// A single resource content item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpResourceContents {
    /// Resource URI.
    pub uri: String,
    /// MIME type.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Base64-encoded binary content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
    /// Metadata.
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<JsonValue>,
}

/// Result of `resources/read`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpReadResourceResult {
    /// Resource contents.
    pub contents: Vec<McpResourceContents>,
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

/// Build the default `initialize` parameters.
pub fn default_initialize_params() -> McpInitializeParams {
    McpInitializeParams {
        protocol_version: "2025-03-26".to_string(),
        capabilities: JsonValue::Object(serde_json::Map::new()),
        client_info: McpInitializeClientInfo {
            name: "opencode-llm".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    }
}

/// Build an MCP tool prefix for a given server name.
///
/// MCP tools are namespaced as `mcp__<server>__<tool>` to avoid collisions
/// with built-in tools.
pub fn mcp_tool_prefix(server_name: &str) -> String {
    format!("mcp__{}__", normalize_name(server_name))
}

/// Build the fully qualified MCP tool name.
pub fn mcp_tool_name(server_name: &str, tool_name: &str) -> String {
    format!("{}{}", mcp_tool_prefix(server_name), normalize_name(tool_name))
}

/// Normalize a name for use in MCP tool identifiers.
pub fn normalize_name(name: &str) -> String {
    name.chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => ch,
            _ => '_',
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonrpc_request_serialization() {
        let req = JsonRpcRequest::new(1u64, "initialize", Some(serde_json::json!({})));
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["method"], "initialize");
        assert_eq!(json["id"], 1);
    }

    #[test]
    fn normalize_name_replaces_invalid_chars() {
        assert_eq!(normalize_name("hello-world"), "hello-world");
        assert_eq!(normalize_name("my server!"), "my_server_");
    }

    #[test]
    fn mcp_tool_prefix_format() {
        let prefix = mcp_tool_prefix("my-server");
        assert_eq!(prefix, "mcp__my-server__");
    }

    #[test]
    fn mcp_tool_name_qualifies() {
        let name = mcp_tool_name("my-server", "read_file");
        assert_eq!(name, "mcp__my-server__read_file");
    }
}