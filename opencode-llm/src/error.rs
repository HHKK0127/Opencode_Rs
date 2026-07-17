//! Error types for `opencode-llm`.
//!
//! All public fallible APIs return [`LlmResult<T>`], which is `Result<T, LlmError>`.

use std::fmt;
use thiserror::Error;

/// Result alias used throughout the crate.
pub type LlmResult<T> = Result<T, LlmError>;

/// Top-level error type for the LLM client library.
#[derive(Debug, Error)]
pub enum LlmError {
    /// Missing or invalid authentication credentials.
    #[error("missing credentials: {0}")]
    MissingCredentials(String),

    /// HTTP transport-level error.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON (de)serialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// SSE parsing or framing error.
    #[error("sse error: {0}")]
    Sse(String),

    /// Provider returned a non-2xx response with a structured error payload.
    #[error("provider error (status {status}): {message}")]
    Provider {
        /// HTTP status code.
        status: u16,
        /// Human-readable error message from the provider.
        message: String,
    },

    /// Tool execution failed.
    #[error("tool execution failed: {0}")]
    Tool(String),

    /// I/O error (file operations, etc.).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Internal invariant violation.
    #[error("internal error: {0}")]
    Internal(String),

    /// Stream closed unexpectedly.
    #[error("stream closed unexpectedly")]
    StreamClosed,

    /// Request was cancelled by the caller.
    #[error("request cancelled")]
    Cancelled,

    /// Network error (timeout, connection failure, etc.).
    #[error("network error: {0}")]
    Network(String),

    /// API returned a structured error.
    #[error("api error: {0}")]
    ApiError(String),

    /// Operation timed out.
    #[error("timeout: {0}")]
    Timeout(String),
}

impl LlmError {
    /// Construct a provider error.
    pub fn provider(status: u16, message: impl Into<String>) -> Self {
        Self::Provider {
            status,
            message: message.into(),
        }
    }

    /// Construct an internal error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Whether the error is transient and may succeed on retry.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Http(_) | Self::Sse(_) | Self::StreamClosed | Self::Network(_) => true,
            Self::Timeout(_) => true,
            Self::Provider { status, .. } => *status >= 500 || *status == 429,
            _ => false,
        }
    }
}

/// Display wrapper for provider error contexts.
#[derive(Debug)]
pub struct ProviderErrorContext {
    /// Provider name (e.g. "anthropic", "openai").
    pub provider: String,
    /// Model identifier.
    pub model: String,
    /// HTTP status, if known.
    pub status: Option<u16>,
    /// Error body, if any.
    pub body: Option<String>,
}

impl fmt::Display for ProviderErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}:{}] status={:?} body={:?}",
            self.provider, self.model, self.status, self.body
        )
    }
}
