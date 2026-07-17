//! OpenAI-compatible chat-completions client.
//!
//! Implements [`Provider`] against `/v1/chat/completions` with SSE streaming.
//! Targets local OpenAI-compatible servers, OpenRouter, vLLM, etc.

use async_trait::async_trait;
use futures::stream::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};
use tracing::debug;

use crate::auth::AuthSource;
use crate::config::{resolve_openai_base_url, LlmConfig};
use crate::error::{LlmError, LlmResult};
use crate::providers::{EventStream, Provider};
use crate::sse::SseParser;
use crate::types::{MessageRequest, MessageResponse, OutputContentBlock};

/// OpenAI-compatible chat-completions client.
#[derive(Debug, Clone)]
pub struct OpenAiCompatClient {
    http: Client,
    auth: AuthSource,
    config: LlmConfig,
}

impl OpenAiCompatClient {
    /// Construct a new client with the given auth and config.
    pub fn new(auth: AuthSource, config: LlmConfig) -> LlmResult<Self> {
        if !auth.is_authenticated() {
            return Err(LlmError::MissingCredentials("openai-compat".into()));
        }
        let http = Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()?;
        Ok(Self {
            http,
            auth,
            config,
        })
    }

    /// Construct a client from environment variables.
    pub fn from_env() -> LlmResult<Self> {
        let auth = AuthSource::from_env()?;
        let mut config = LlmConfig::openai_compat(
            std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string()),
            resolve_openai_base_url(),
        );
        if let Ok(base) = std::env::var("OPENAI_BASE_URL") {
            config.base_url = base;
        }
        Self::new(auth, config)
    }

    /// Build the chat-completions URL.
    fn url(&self) -> String {
        let base = self.config.base_url.trim_end_matches('/');
        if base.ends_with("/chat/completions") {
            base.to_string()
        } else {
            format!("{base}/chat/completions")
        }
    }

    /// Apply the Authorization header.
    fn apply_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(bearer) = self.auth.bearer_token() {
            builder.bearer_auth(bearer)
        } else if let Some(key) = self.auth.api_key() {
            builder.bearer_auth(key)
        } else {
            builder
        }
    }

    /// Translate our Anthropic-style [`MessageRequest`] into a chat-completions
    /// payload. Tool definitions and tool results are mapped 1:1.
    fn build_body(&self, req: &MessageRequest) -> Value {
        let mut messages: Vec<Value> = Vec::new();

        if let Some(system) = &req.system {
            messages.push(json!({"role": "system", "content": system}));
        }
        for m in &req.messages {
            match m.role.as_str() {
                "user" => {
                    // Concatenate text blocks (OpenAI does not support multi-modal
                    // tool-result blocks directly — convert to a string).
                    let text = m
                        .content
                        .iter()
                        .filter_map(|b| match b {
                            crate::types::InputContentBlock::Text { text } => Some(text.clone()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("");
                    if !text.is_empty() {
                        messages.push(json!({"role": "user", "content": text}));
                    }
                    for b in &m.content {
                        if let crate::types::InputContentBlock::ToolResult {
                            tool_use_id,
                            content,
                            ..
                        } = b
                        {
                            let payload = content
                                .iter()
                                .filter_map(|c| match c {
                                    crate::types::ToolResultContentBlock::Text { text } => {
                                        Some(text.clone())
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("");
                            messages.push(json!({
                                "role": "tool",
                                "tool_call_id": tool_use_id,
                                "content": payload,
                            }));
                        }
                    }
                }
                "assistant" => {
                    let text = m
                        .content
                        .iter()
                        .filter_map(|b| match b {
                            crate::types::InputContentBlock::Text { text } => Some(text.clone()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("");
                    messages.push(json!({"role": "assistant", "content": text}));
                }
                _ => {
                    // Pass through other roles as-is.
                    messages.push(serde_json::to_value(m).unwrap_or(json!({})));
                }
            }
        }

        let mut body = json!({
            "model": req.model,
            "messages": messages,
            "max_tokens": req.max_tokens,
            "stream": req.stream,
        });
        if let Some(t) = req.temperature {
            body["temperature"] = json!(t);
        }
        if let Some(p) = req.top_p {
            body["top_p"] = json!(p);
        }
        if let Some(stop) = &req.stop {
            body["stop"] = json!(stop);
        }
        for (k, v) in &req.extra_body {
            body[k] = v.clone();
        }
        body
    }
}

#[async_trait]
impl Provider for OpenAiCompatClient {
    fn kind(&self) -> crate::config::ProviderKind {
        crate::config::ProviderKind::OpenAiCompat
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    async fn send_message(&self, request: MessageRequest) -> LlmResult<MessageResponse> {
        let body = self.build_body(&request);
        debug!(model = %request.model, "openai_compat send_message");
        let response = self
            .apply_auth(self.http.post(self.url()))
            .json(&body)
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::provider(status.as_u16(), text));
        }
        let raw: Value = response.json().await?;
        Ok(openai_response_to_message(&raw))
    }

    async fn stream_message(&self, request: MessageRequest) -> LlmResult<EventStream> {
        let mut body = self.build_body(&request);
        body["stream"] = json!(true);
        debug!(model = %request.model, "openai_compat stream_message");
        let response = self
            .apply_auth(self.http.post(self.url()))
            .json(&body)
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::provider(status.as_u16(), text));
        }
        let byte_stream = response.bytes_stream();
        let stream = async_stream::try_stream! {
            let mut parser = SseParser::new();
            let mut byte_stream = byte_stream;
            while let Some(chunk) = byte_stream.next().await {
                let chunk = chunk.map_err(LlmError::Http)?;
                // OpenAI uses `data: [DONE]` as the terminator and the
                // event-name header is omitted, so the SseParser maps that
                // to a MessageStop already.
                let events = parser.push(&chunk)?;
                for ev in events {
                    yield ev;
                }
            }
            let trailing = parser.finish()?;
            for ev in trailing {
                yield ev;
            }
        };
        Ok(Box::pin(stream))
    }
}

/// Translate a non-streaming chat-completions payload into our
/// [`MessageResponse`].
pub fn openai_response_to_message(value: &Value) -> MessageResponse {
    let id = value
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let model = value
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let stop_reason = value
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("finish_reason"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut content = Vec::new();
    if let Some(choice) = value.get("choices").and_then(|c| c.get(0)) {
        if let Some(msg) = choice.get("message") {
            if let Some(text) = msg.get("content").and_then(|v| v.as_str()) {
                if !text.is_empty() {
                    content.push(OutputContentBlock::Text {
                        text: text.to_string(),
                    });
                }
            }
            if let Some(tool_calls) = msg.get("tool_calls").and_then(|v| v.as_array()) {
                for tc in tool_calls {
                    let id = tc
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = tc
                        .get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let args = tc
                        .get("function")
                        .and_then(|f| f.get("arguments"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| serde_json::from_str::<Value>(s).ok())
                        .unwrap_or(Value::Null);
                    content.push(OutputContentBlock::ToolUse {
                        id,
                        name,
                        input: args,
                    });
                }
            }
        }
    }

    let usage = value
        .get("usage")
        .and_then(|u| {
            Some(crate::types::Usage {
                input_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                output_tokens: u.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
                    as u32,
                cache_read_input_tokens: 0,
                cache_creation_input_tokens: 0,
            })
        })
        .unwrap_or_default();

    MessageResponse {
        id,
        model,
        stop_reason,
        content,
        usage,
    }
}
