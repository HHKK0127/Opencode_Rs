//! Core request / response types for the LLM clients.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// A request to `/v1/messages` (Anthropic) or `/chat/completions` (OpenAI-compat).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageRequest {
    /// Model identifier.
    pub model: String,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Conversation history.
    pub messages: Vec<InputMessage>,
    /// Optional system prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Tool definitions available to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    /// Tool-choice directive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    /// Whether to stream the response.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub stream: bool,
    /// Sampling temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Top-p sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Stop sequences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    /// Provider-specific extra body parameters.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra_body: BTreeMap<String, Value>,
}

impl MessageRequest {
    /// Enable streaming on this request.
    pub fn with_streaming(mut self) -> Self {
        self.stream = true;
        self
    }

    /// Set the system prompt.
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Set the model.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set max tokens.
    pub fn with_max_tokens(mut self, n: u32) -> Self {
        self.max_tokens = n;
        self
    }
}

/// A role-tagged message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputMessage {
    /// Role (e.g. `user`, `assistant`).
    pub role: String,
    /// Content blocks.
    pub content: Vec<InputContentBlock>,
}

impl InputMessage {
    /// Build a user text message.
    pub fn user_text(text: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: vec![InputContentBlock::Text { text: text.into() }],
        }
    }

    /// Build an assistant text message.
    pub fn assistant_text(text: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: vec![InputContentBlock::Text { text: text.into() }],
        }
    }

    /// Build a user message carrying a tool result.
    pub fn user_tool_result(
        tool_use_id: impl Into<String>,
        content: impl Into<String>,
        is_error: bool,
    ) -> Self {
        Self {
            role: "user".to_string(),
            content: vec![InputContentBlock::ToolResult {
                tool_use_id: tool_use_id.into(),
                content: vec![ToolResultContentBlock::Text {
                    text: content.into(),
                }],
                is_error,
            }],
        }
    }

    /// Build a user message with the given content blocks.
    pub fn user(content: Vec<InputContentBlock>) -> Self {
        Self {
            role: "user".to_string(),
            content,
        }
    }

    /// Build a message with the given role and content blocks.
    pub fn new(role: impl Into<String>, content: Vec<InputContentBlock>) -> Self {
        Self {
            role: role.into(),
            content,
        }
    }

    /// Build an assistant message from output blocks.
    pub fn assistant_from_blocks(blocks: &[OutputContentBlock]) -> Self {
        let content = blocks
            .iter()
            .map(|b| match b {
                OutputContentBlock::Text { text } => InputContentBlock::Text { text: text.clone() },
                OutputContentBlock::ToolUse { id, name, input } => InputContentBlock::ToolUse {
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                },
            })
            .collect();
        Self {
            role: "assistant".to_string(),
            content,
        }
    }
}

/// Content block carried in an input message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputContentBlock {
    /// Plain text.
    Text {
        /// The text content.
        text: String,
    },
    /// Image (base64 data or URL — provider-specific).
    Image {
        /// Source descriptor.
        source: ImageSource,
    },
    /// Result of a tool invocation.
    ToolResult {
        /// ID of the corresponding tool-use block.
        tool_use_id: String,
        /// Result content blocks.
        content: Vec<ToolResultContentBlock>,
        /// Whether the tool call resulted in an error.
        #[serde(default)]
        is_error: bool,
    },
    /// Tool invocation (assistant role only).
    ToolUse {
        /// Tool invocation ID.
        id: String,
        /// Tool name.
        name: String,
        /// Tool input (validated against the tool's JSON schema by the provider).
        input: Value,
    },
}

/// Image source for [`InputContentBlock::Image`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    /// Base64-encoded image data.
    Base64 {
        /// Media type (e.g. `image/png`).
        media_type: String,
        /// Base64 payload.
        data: String,
    },
    /// URL reference.
    Url {
        /// Image URL.
        url: String,
    },
}

/// Content block within a tool result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolResultContentBlock {
    /// Text content.
    Text {
        /// Text payload.
        text: String,
    },
}

/// Tool-use block within an assistant response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolUseBlock {
    /// Tool invocation ID.
    pub id: String,
    /// Tool name.
    pub name: String,
    /// Tool input.
    pub input: Value,
}

/// Tool definition sent to the provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolDefinition {
    /// Tool name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// JSON schema describing the tool input.
    pub input_schema: Value,
}

impl ToolDefinition {
    /// Construct a tool definition.
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
}

/// Tool-choice directive.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolChoice {
    /// No tool calls.
    None,
    /// Provider decides.
    Auto,
    /// Must call at least one tool.
    Any,
    /// Must call a specific tool.
    Tool {
        /// Tool name.
        name: String,
    },
}

/// A non-streaming response from the provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    /// Response identifier.
    pub id: String,
    /// Model that produced the response.
    pub model: String,
    /// Stop reason (`end_turn`, `tool_use`, `max_tokens`, …).
    pub stop_reason: Option<String>,
    /// Output content blocks.
    pub content: Vec<OutputContentBlock>,
    /// Token usage.
    #[serde(default)]
    pub usage: Usage,
}

/// Content block in an output (assistant) response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputContentBlock {
    /// Plain text.
    Text {
        /// The text.
        text: String,
    },
    /// Tool invocation.
    ToolUse {
        /// Tool invocation ID.
        id: String,
        /// Tool name.
        name: String,
        /// Tool input.
        input: Value,
    },
}

/// Token usage and cost estimate.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Usage {
    /// Input tokens consumed.
    pub input_tokens: u32,
    /// Output tokens generated.
    pub output_tokens: u32,
    /// Tokens read from cache (provider-specific).
    #[serde(default)]
    pub cache_read_input_tokens: u32,
    /// Tokens written to cache (provider-specific).
    #[serde(default)]
    pub cache_creation_input_tokens: u32,
}

impl Usage {
    /// Add usage from another request.
    pub fn add_assign(&mut self, other: &Usage) {
        self.input_tokens = self.input_tokens.saturating_add(other.input_tokens);
        self.output_tokens = self.output_tokens.saturating_add(other.output_tokens);
        self.cache_read_input_tokens = self
            .cache_read_input_tokens
            .saturating_add(other.cache_read_input_tokens);
        self.cache_creation_input_tokens = self
            .cache_creation_input_tokens
            .saturating_add(other.cache_creation_input_tokens);
    }

    /// Total tokens (input + output).
    pub fn total(&self) -> u32 {
        self.input_tokens.saturating_add(self.output_tokens)
    }
}
