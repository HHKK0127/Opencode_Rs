//! MCP stdio transport — spawns a child process and communicates over
//! stdin/stdout using the MCP HTTP-like frame protocol (Content-Length headers).

use std::io;
use std::process::Stdio;
use std::collections::BTreeMap;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

use crate::mcp::jsonrpc::{
    JsonRpcId, JsonRpcRequest, JsonRpcResponse, McpInitializeParams,
    McpInitializeResult, McpListToolsParams, McpListToolsResult, McpToolCallParams,
    McpToolCallResult, McpListResourcesParams, McpListResourcesResult, McpReadResourceParams,
    McpReadResourceResult,
};

/// A stdio-based MCP process.
///
/// Wraps a child process and provides methods to send/receive JSON-RPC
/// messages framed with `Content-Length` headers.
#[derive(Debug)]
pub struct McpStdioProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl McpStdioProcess {
    /// Spawn a new stdio MCP process.
    pub fn spawn(command: &str, args: &[String], env: &BTreeMap<String, String>) -> io::Result<Self> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        for (key, value) in env {
            cmd.env(key, value);
        }

        let mut child = cmd.spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("MCP process missing stdin pipe"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("MCP process missing stdout pipe"))?;

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    // -----------------------------------------------------------------------
    // Raw I/O
    // -----------------------------------------------------------------------

    /// Write raw bytes to stdin.
    pub async fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.stdin.write_all(bytes).await
    }

    /// Flush stdin.
    pub async fn flush(&mut self) -> io::Result<()> {
        self.stdin.flush().await
    }

    /// Write a line (appends `\n`).
    pub async fn write_line(&mut self, line: &str) -> io::Result<()> {
        self.write_all(line.as_bytes()).await?;
        self.write_all(b"\n").await?;
        self.flush().await
    }

    /// Read a single line from stdout.
    pub async fn read_line(&mut self) -> io::Result<String> {
        let mut line = String::new();
        let bytes_read = self.stdout.read_line(&mut line).await?;
        if bytes_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "MCP stdio stream closed while reading line",
            ));
        }
        Ok(line)
    }

    // -----------------------------------------------------------------------
    // Frame protocol (Content-Length headers)
    // -----------------------------------------------------------------------

    /// Write a framed message (Content-Length header + JSON body).
    pub async fn write_frame(&mut self, payload: &[u8]) -> io::Result<()> {
        let header = format!("Content-Length: {}\r\n\r\n", payload.len());
        self.write_all(header.as_bytes()).await?;
        self.write_all(payload).await?;
        self.flush().await
    }

    /// Read a framed message (headers + body).
    pub async fn read_frame(&mut self) -> io::Result<Vec<u8>> {
        let mut content_length = None;
        loop {
            let mut line = String::new();
            let bytes_read = self.stdout.read_line(&mut line).await?;
            if bytes_read == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "MCP stdio stream closed while reading headers",
                ));
            }
            // Empty line (CRLF) marks end of headers.
            if line == "\r\n" {
                break;
            }
            let header = line.trim_end_matches(['\r', '\n']);
            if let Some((name, value)) = header.split_once(':') {
                if name.trim().eq_ignore_ascii_case("Content-Length") {
                    let parsed = value
                        .trim()
                        .parse::<usize>()
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    content_length = Some(parsed);
                }
            }
        }

        let content_length = content_length.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "missing Content-Length header",
            )
        })?;
        let mut payload = vec![0u8; content_length];
        self.stdout.read_exact(&mut payload).await?;
        Ok(payload)
    }

    // -----------------------------------------------------------------------
    // JSON-RPC message helpers
    // -----------------------------------------------------------------------

    /// Serialize and send a JSON-RPC message.
    pub async fn write_jsonrpc<T: Serialize>(&mut self, message: &T) -> io::Result<()> {
        let body =
            serde_json::to_vec(message).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.write_frame(&body).await
    }

    /// Read and deserialize a JSON-RPC response.
    pub async fn read_jsonrpc<T: DeserializeOwned>(&mut self) -> io::Result<T> {
        let payload = self.read_frame().await?;
        serde_json::from_slice(&payload)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Send a JSON-RPC request and read the response (generic).
    pub async fn request<TParams: Serialize, TResult: DeserializeOwned>(
        &mut self,
        id: JsonRpcId,
        method: impl Into<String>,
        params: Option<TParams>,
    ) -> io::Result<JsonRpcResponse<TResult>> {
        let method = method.into();
        let request = JsonRpcRequest::new(id.clone(), &method, params);
        self.write_jsonrpc(&request).await?;
        let response: JsonRpcResponse<TResult> = self.read_jsonrpc().await?;

        // Validate protocol version.
        if response.jsonrpc != "2.0" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "MCP response for {method} used unsupported jsonrpc version `{}`",
                    response.jsonrpc
                ),
            ));
        }

        // Validate response id matches request id.
        if response.id != id {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "MCP response for {method} used mismatched id: expected {id:?}, got {:?}",
                    response.id
                ),
            ));
        }

        Ok(response)
    }

    // -----------------------------------------------------------------------
    // MCP-specific methods
    // -----------------------------------------------------------------------

    /// Perform the `initialize` handshake.
    pub async fn initialize(
        &mut self,
        id: JsonRpcId,
        params: McpInitializeParams,
    ) -> io::Result<JsonRpcResponse<McpInitializeResult>> {
        self.request(id, "initialize", Some(params)).await
    }

    /// List available tools.
    pub async fn list_tools(
        &mut self,
        id: JsonRpcId,
        params: Option<McpListToolsParams>,
    ) -> io::Result<JsonRpcResponse<McpListToolsResult>> {
        self.request(id, "tools/list", params).await
    }

    /// Call a tool.
    pub async fn call_tool(
        &mut self,
        id: JsonRpcId,
        params: McpToolCallParams,
    ) -> io::Result<JsonRpcResponse<McpToolCallResult>> {
        self.request(id, "tools/call", Some(params)).await
    }

    /// List available resources.
    pub async fn list_resources(
        &mut self,
        id: JsonRpcId,
        params: Option<McpListResourcesParams>,
    ) -> io::Result<JsonRpcResponse<McpListResourcesResult>> {
        self.request(id, "resources/list", params).await
    }

    /// Read a resource.
    pub async fn read_resource(
        &mut self,
        id: JsonRpcId,
        params: McpReadResourceParams,
    ) -> io::Result<JsonRpcResponse<McpReadResourceResult>> {
        self.request(id, "resources/read", Some(params)).await
    }

    /// Gracefully terminate the child process.
    pub async fn shutdown(&mut self) -> io::Result<()> {
        if self.child.try_wait()?.is_none() {
            match self.child.kill().await {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::InvalidInput => {}
                Err(e) => return Err(e),
            }
        }
        let _ = self.child.wait().await?;
        Ok(())
    }

    /// Check whether the child process has exited.
    pub fn has_exited(&mut self) -> io::Result<bool> {
        Ok(self.child.try_wait()?.is_some())
    }
}

impl Drop for McpStdioProcess {
    fn drop(&mut self) {
        // Best-effort cleanup in a blocking context.
        let _ = self.child.try_wait();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn encode_frame_format() {
        let payload = b"{}";
        let header = format!("Content-Length: {}\r\n\r\n", payload.len());
        let mut framed = header.into_bytes();
        framed.extend_from_slice(payload);
        let expected = b"Content-Length: 2\r\n\r\n{}";
        assert_eq!(&framed, expected);
    }
}