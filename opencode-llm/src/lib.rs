//! # opencode-llm
//!
//! LLM client library for OpenCode_Rs.
//!
//! Provides Anthropic and OpenAI-compatible provider clients with SSE streaming,
//! tool execution, and a conversation runtime.
//!
//! Architecture (inspired by `claw-code`'s Rust workspace):
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │  providers::anthropic / openai_compat       │  ← HTTP / SSE / auth
//! │  - AuthSource (ApiKey, BearerToken)         │
//! │  - ProviderClient trait + AnthropicClient    │
//! │  - OpenAiCompatClient                        │
//! └──────────────┬──────────────────────────────┘
//!                │ Stream<StreamEvent>
//! ┌──────────────▼──────────────────────────────┐
//! │  conversation::ConversationRuntime          │  ← turn loop
//! │  - send_prompt / run_turn                    │
//! │  - tool dispatch via StaticToolExecutor     │
//! │  - auto-compaction hook                      │
//! └──────────────┬──────────────────────────────┘
//!                │ execute_tool(name, args)
//! ┌──────────────▼──────────────────────────────┐
//! │  tools::ToolSpec / mvp_tool_specs()          │  ← built-in tool set
//! │  - bash, read_file, write_file, edit_file,   │
//! │    glob_search, grep_search, web_*           │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! ## Quick start
//!
//! ```ignore
//! use opencode_llm::prelude::*;
//! use opencode_llm::config::LlmConfig;
//! use std::sync::Arc;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let auth = AuthSource::from_env()?;
//! let client = Arc::new(AnthropicClient::new(auth, LlmConfig::anthropic("claude-opus-4-6"))?);
//! let runtime = ConversationRuntime::builder()
//!     .model("claude-opus-4-6")
//!     .max_tokens(4096)
//!     .mvp_tools()
//!     .build(client);
//! let answer = runtime.run("Explain this codebase").await?;
//! println!("{answer}");
//! # Ok(()) }
//! ```

#![warn(missing_docs)]

pub mod auth;
pub mod cache;
pub mod config;
pub mod conversation;
pub mod error;
pub mod events;
pub mod git_context;
pub mod lsp;
pub mod mcp;
pub mod oauth;
pub mod permissions;
pub mod plugin;
pub mod session;
pub mod pricing;
pub mod providers;
pub mod registry;
pub mod sse;
pub mod tools;
pub mod types;

pub use auth::AuthSource;
pub use conversation::{ConversationRuntime, ConversationRuntimeBuilder};
pub use error::{LlmError, LlmResult};
pub use events::{AssistantEvent, StreamEvent};
pub use providers::{anthropic::AnthropicClient, openai_compat::OpenAiCompatClient, Provider};
pub use tools::{mvp_tool_specs, ToolContext, ToolError, ToolExecutor, ToolSpec};
pub use types::{
    InputContentBlock, InputMessage, MessageRequest, MessageResponse, ToolDefinition, Usage,
};

/// Re-exports for the most common items.
pub mod prelude {
    pub use crate::auth::AuthSource;
    pub use crate::conversation::{ConversationRuntime, ConversationRuntimeBuilder};
    pub use crate::error::{LlmError, LlmResult};
    pub use crate::events::{AssistantEvent, StreamEvent};
    pub use crate::config::LlmConfig;
    pub use crate::providers::anthropic::AnthropicClient;
    pub use crate::providers::openai_compat::OpenAiCompatClient;
    pub use crate::providers::Provider;
    pub use crate::tools::{mvp_tool_specs, ToolContext, ToolError, ToolExecutor, ToolSpec};
    pub use crate::types::{
        InputContentBlock, InputMessage, MessageRequest, MessageResponse, ToolDefinition, Usage,
    };
}
