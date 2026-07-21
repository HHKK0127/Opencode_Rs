//! LSP (Language Server Protocol) client.
//!
//! Connects to a language server via stdio JSON-RPC and provides
//! code intelligence: diagnostics, completions, hover, go-to-definition.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{LlmError, LlmResult};

/// JSON-RPC message header.
const LSP_HEADER: &str = "Content-Length";

/// LSP client state.
pub struct LspClient {
    /// Child process handle.
    process: Mutex<Child>,
    /// stdin to the server.
    stdin: Mutex<ChildStdin>,
    /// Reader for stdout messages.
    reader: Mutex<BufReader<std::process::ChildStdout>>,
    /// Request ID counter.
    next_id: AtomicU64,
    /// Server capabilities (cached after initialize).
    capabilities: Mutex<Option<ServerCapabilities>>,
    /// Whether the client is initialized.
    initialized: Mutex<bool>,
}

/// Server capabilities returned by the `initialize` response.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ServerCapabilities {
    /// Whether the server supports textDocument/completion.
    #[serde(default)]
    pub completion_provider: Option<Value>,
    /// Whether the server supports textDocument/hover.
    #[serde(default)]
    pub hover_provider: Option<Value>,
    /// Whether the server supports textDocument/definition.
    #[serde(default)]
    pub definition_provider: Option<Value>,
    /// Whether the server supports textDocument/references.
    #[serde(default)]
    pub references_provider: Option<Value>,
    /// Whether the server supports textDocument/codeAction.
    #[serde(default)]
    pub code_action_provider: Option<Value>,
    /// Whether the server supports textDocument/formatting.
    #[serde(default)]
    pub document_formatting_provider: Option<Value>,
    /// Whether the server supports workspace/symbol.
    #[serde(default)]
    pub workspace_symbol_provider: Option<Value>,
}

/// A text document item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentItem {
    /// Document URI (file:///path).
    pub uri: String,
    /// Language ID (e.g., "rust", "python").
    pub language_id: String,
    /// Version number (increment on each change).
    pub version: i64,
    /// Full text content.
    pub text: String,
}

/// A position in a text document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    /// 0-based line number.
    pub line: u64,
    /// 0-based character offset.
    pub character: u64,
}

/// A range in a text document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Range {
    /// Start position.
    pub start: Position,
    /// End position.
    pub end: Position,
}

/// A diagnostic item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Range of the diagnostic.
    pub range: Range,
    /// Severity: 1=error, 2=warning, 3=info, 4=hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<u64>,
    /// Diagnostic message.
    pub message: String,
    /// Source (e.g., "rustc", "rust-analyzer").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// A completion item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    /// Label to display.
    pub label: String,
    /// Kind: 1=text, 2=method, 3=function, 4=constructor, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<u64>,
    /// Detail text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Documentation string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
}

/// A location (for go-to-definition, references).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// Document URI.
    pub uri: String,
    /// Range.
    pub range: Range,
}

/// A text edit (for formatting, code actions).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    /// Range to replace.
    pub range: Range,
    /// New text.
    pub new_text: String,
}

/// A workspace edit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceEdit {
    /// Changes keyed by document URI.
    #[serde(default)]
    pub changes: BTreeMap<String, Vec<TextEdit>>,
}

impl LspClient {
    /// Start a new LSP server process.
    ///
    /// `command` is the server executable (e.g., "rust-analyzer", "pylsp").
    /// `args` are additional arguments.
    pub fn start(command: &str, args: &[&str]) -> LlmResult<Self> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                LlmError::Config(format!("failed to start LSP server `{command}`: {e}"))
            })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| LlmError::Config("failed to capture LSP server stdin".to_string()))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| LlmError::Config("failed to capture LSP server stdout".to_string()))?;

        Ok(Self {
            process: Mutex::new(child),
            stdin: Mutex::new(stdin),
            reader: Mutex::new(BufReader::new(stdout)),
            next_id: AtomicU64::new(1),
            capabilities: Mutex::new(None),
            initialized: Mutex::new(false),
        })
    }

    /// Initialize the LSP server.
    ///
    /// Sends `initialize` and `initialized` notifications.
    pub fn initialize(
        &self,
        root_uri: &str,
        workspace_folders: Option<Vec<String>>,
    ) -> LlmResult<ServerCapabilities> {
        let capabilities = serde_json::json!({
            "textDocument": {
                "synchronization": {
                    "didOpen": true,
                    "didChange": true,
                    "willSave": true,
                    "willSaveWaitUntil": false,
                    "didClose": true
                },
                "completion": {
                    "completionItem": {
                        "snippetSupport": true
                    }
                },
                "hover": {
                    "contentFormat": ["markdown", "plaintext"]
                },
                "definition": {},
                "references": {},
                "codeAction": {},
                "formatting": {}
            },
            "workspace": {
                "symbol": true
            }
        });

        let params = serde_json::json!({
            "processId": std::process::id(),
            "clientInfo": {
                "name": "opencode-llm",
                "version": "0.1.0"
            },
            "capabilities": capabilities,
            "rootUri": root_uri,
            "workspaceFolders": workspace_folders.map(|folders| {
                folders.into_iter().map(|f| {
                    serde_json::json!({
                        "uri": f,
                        "name": f.rsplit('/').next().unwrap_or("workspace")
                    })
                }).collect::<Vec<_>>()
            })
        });

        let response: Value = self.send_request("initialize", params)?;
        let capabilities: ServerCapabilities =
            serde_json::from_value(response.get("capabilities").cloned().unwrap_or_default())
                .map_err(LlmError::Json)?;

        *self.capabilities.lock().unwrap() = Some(capabilities.clone());

        // Send initialized notification.
        self.send_notification("initialized", serde_json::json!({}))?;
        *self.initialized.lock().unwrap() = true;

        Ok(capabilities)
    }

    /// Open a text document.
    pub fn did_open(&self, document: &TextDocumentItem) -> LlmResult<()> {
        self.send_notification(
            "textDocument/didOpen",
            serde_json::json!({
                "textDocument": document
            }),
        )
    }

    /// Close a text document.
    pub fn did_close(&self, uri: &str) -> LlmResult<()> {
        self.send_notification(
            "textDocument/didClose",
            serde_json::json!({
                "textDocument": { "uri": uri }
            }),
        )
    }

    /// Request completions at a position.
    pub fn completion(
        &self,
        uri: &str,
        line: u64,
        character: u64,
    ) -> LlmResult<Vec<CompletionItem>> {
        let response: Value = self.send_request(
            "textDocument/completion",
            serde_json::json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character }
            }),
        )?;

        // The response may be a CompletionList or a CompletionItem array.
        let items = if let Some(list) = response.get("items") {
            list.clone()
        } else {
            response
        };

        let items: Vec<CompletionItem> = serde_json::from_value(items).map_err(LlmError::Json)?;
        Ok(items)
    }

    /// Request hover information at a position.
    pub fn hover(&self, uri: &str, line: u64, character: u64) -> LlmResult<Option<String>> {
        let response: Value = self.send_request(
            "textDocument/hover",
            serde_json::json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character }
            }),
        )?;

        if let Some(contents) = response.get("contents") {
            let value = match contents {
                Value::String(s) => s.clone(),
                Value::Object(map) => map
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                _ => contents.to_string(),
            };
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Request go-to-definition.
    pub fn definition(&self, uri: &str, line: u64, character: u64) -> LlmResult<Vec<Location>> {
        let response: Value = self.send_request(
            "textDocument/definition",
            serde_json::json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character }
            }),
        )?;

        // Response can be a single Location or an array.
        let locations: Vec<Location> = if response.is_array() {
            serde_json::from_value(response).map_err(LlmError::Json)?
        } else {
            let loc: Location = serde_json::from_value(response).map_err(LlmError::Json)?;
            vec![loc]
        };
        Ok(locations)
    }

    /// Request references.
    pub fn references(&self, uri: &str, line: u64, character: u64) -> LlmResult<Vec<Location>> {
        let response: Value = self.send_request(
            "textDocument/references",
            serde_json::json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character },
                "context": { "includeDeclaration": true }
            }),
        )?;

        let locations: Vec<Location> = serde_json::from_value(response).map_err(LlmError::Json)?;
        Ok(locations)
    }

    /// Request document formatting.
    pub fn formatting(&self, uri: &str) -> LlmResult<Vec<TextEdit>> {
        let response: Value = self.send_request(
            "textDocument/formatting",
            serde_json::json!({
                "textDocument": { "uri": uri },
                "options": {
                    "tabSize": 4,
                    "insertSpaces": true
                }
            }),
        )?;

        let edits: Vec<TextEdit> = serde_json::from_value(response).map_err(LlmError::Json)?;
        Ok(edits)
    }

    /// Request workspace symbols.
    pub fn workspace_symbols(&self, query: &str) -> LlmResult<Vec<Value>> {
        let response: Value = self.send_request(
            "workspace/symbol",
            serde_json::json!({
                "query": query
            }),
        )?;

        let symbols: Vec<Value> = serde_json::from_value(response).map_err(LlmError::Json)?;
        Ok(symbols)
    }

    /// Shutdown the LSP server.
    pub fn shutdown(&self) -> LlmResult<()> {
        self.send_request("shutdown", serde_json::json!({}))?;
        self.send_notification("exit", serde_json::json!({}))?;
        if let Ok(mut proc) = self.process.lock() {
            let _ = proc.wait();
        }
        Ok(())
    }

    // --- Internal methods ---

    /// Send a JSON-RPC request and wait for the response.
    fn send_request(&self, method: &str, params: Value) -> LlmResult<Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        self.write_message(&request)?;
        self.read_response(id)
    }

    /// Send a JSON-RPC notification (no response expected).
    fn send_notification(&self, method: &str, params: Value) -> LlmResult<()> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        self.write_message(&notification)
    }

    /// Write a JSON-RPC message to the server's stdin.
    fn write_message(&self, message: &Value) -> LlmResult<()> {
        let body = serde_json::to_string(message).map_err(LlmError::Json)?;
        let header = format!("{LSP_HEADER}: {}\r\n\r\n", body.len());
        let mut stdin = self.stdin.lock().unwrap();
        stdin.write_all(header.as_bytes()).map_err(LlmError::Io)?;
        stdin.write_all(body.as_bytes()).map_err(LlmError::Io)?;
        stdin.flush().map_err(LlmError::Io)?;
        Ok(())
    }

    /// Read a JSON-RPC response for a specific request ID.
    fn read_response(&self, expected_id: u64) -> LlmResult<Value> {
        let mut reader = self.reader.lock().unwrap();
        loop {
            let mut header = String::new();
            // Read headers.
            let mut content_length: Option<usize> = None;
            loop {
                header.clear();
                if reader.read_line(&mut header).map_err(LlmError::Io)? == 0 {
                    return Err(LlmError::StreamClosed);
                }
                let trimmed = header.trim();
                if trimmed.is_empty() {
                    break; // End of headers.
                }
                if let Some(len_str) = trimmed.strip_prefix("Content-Length: ") {
                    content_length =
                        Some(len_str.parse::<usize>().map_err(|_| {
                            LlmError::Internal("invalid Content-Length".to_string())
                        })?);
                }
            }
            let len = content_length
                .ok_or_else(|| LlmError::Internal("missing Content-Length header".to_string()))?;
            let mut buf = vec![0u8; len];
            reader.read_exact(&mut buf).map_err(LlmError::Io)?;
            let msg: Value = serde_json::from_slice(&buf).map_err(LlmError::Json)?;
            // Check for response with matching ID.
            if let Some(msg_id) = msg.get("id").and_then(|v| v.as_u64()) {
                if msg_id == expected_id {
                    // Check for error.
                    if let Some(error) = msg.get("error") {
                        let code = error.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
                        let message = error
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown error");
                        return Err(LlmError::ApiError(format!(
                            "LSP error (code {code}): {message}"
                        )));
                    }
                    if let Some(result) = msg.get("result") {
                        return Ok(result.clone());
                    }
                    return Ok(serde_json::json!(null));
                }
            }
            // Ignore notifications (no ID) or unmatched responses.
        }
    }

    /// Get the cached server capabilities.
    pub fn capabilities(&self) -> Option<ServerCapabilities> {
        self.capabilities.lock().unwrap().clone()
    }

    /// Whether the client is initialized.
    pub fn is_initialized(&self) -> bool {
        *self.initialized.lock().unwrap()
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.send_request("shutdown", serde_json::json!({}));
        self.send_notification("exit", serde_json::json!({})).ok();
        if let Ok(mut proc) = self.process.lock() {
            let _ = proc.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_fails_for_nonexistent_server() {
        let result = LspClient::start("nonexistent-lsp-server-12345", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn position_creation() {
        let pos = Position {
            line: 0,
            character: 5,
        };
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 5);
    }

    #[test]
    fn range_creation() {
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 1,
                character: 0,
            },
        };
        assert_eq!(range.start.line, 0);
        assert_eq!(range.end.line, 1);
    }

    #[test]
    fn text_document_item_creation() {
        let doc = TextDocumentItem {
            uri: "file:///test.rs".to_string(),
            language_id: "rust".to_string(),
            version: 1,
            text: "fn main() {}".to_string(),
        };
        assert_eq!(doc.language_id, "rust");
        assert_eq!(doc.version, 1);
    }

    #[test]
    fn diagnostic_creation() {
        let diag = Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 5,
                },
            },
            severity: Some(1),
            message: "test error".to_string(),
            source: Some("rustc".to_string()),
        };
        assert_eq!(diag.severity, Some(1));
        assert_eq!(diag.message, "test error");
    }

    #[test]
    fn completion_item_creation() {
        let item = CompletionItem {
            label: "println".to_string(),
            kind: Some(3),
            detail: Some("macro".to_string()),
            documentation: Some("Print to stdout".to_string()),
        };
        assert_eq!(item.label, "println");
        assert_eq!(item.kind, Some(3));
    }

    #[test]
    fn location_creation() {
        let loc = Location {
            uri: "file:///test.rs".to_string(),
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 1,
                    character: 0,
                },
            },
        };
        assert_eq!(loc.uri, "file:///test.rs");
    }

    #[test]
    fn text_edit_creation() {
        let edit = TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 5,
                },
            },
            new_text: "hello".to_string(),
        };
        assert_eq!(edit.new_text, "hello");
    }

    #[test]
    fn workspace_edit_creation() {
        let mut changes = BTreeMap::new();
        changes.insert(
            "file:///test.rs".to_string(),
            vec![TextEdit {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 5,
                    },
                },
                new_text: "hello".to_string(),
            }],
        );
        let edit = WorkspaceEdit { changes };
        assert_eq!(edit.changes.len(), 1);
    }
}
