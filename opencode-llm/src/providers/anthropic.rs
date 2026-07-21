//! Anthropic Messages API client.
//!
//! Implements [`Provider`] against `https://api.anthropic.com/v1/messages`,
//! supporting both streaming and non-streaming responses.

use async_trait::async_trait;
use futures::stream::StreamExt;
use reqwest::Client;
use serde_json::json;
use tracing::{debug, warn};

use crate::auth::AuthSource;
use crate::config::{resolve_anthropic_base_url, LlmConfig, ANTHROPIC_VERSION};
use crate::error::{LlmError, LlmResult};
use crate::events::{ContentDelta, StreamEvent};
use crate::providers::{EventStream, Provider};
use crate::sse::SseParser;
use crate::types::{MessageRequest, MessageResponse};

/// Anthropic Messages API client.
#[derive(Debug, Clone)]
pub struct AnthropicClient {
    http: Client,
    auth: AuthSource,
    config: LlmConfig,
}

impl AnthropicClient {
    /// Construct a new client with the given auth and config.
    pub fn new(auth: AuthSource, config: LlmConfig) -> LlmResult<Self> {
        if !auth.is_authenticated() {
            return Err(LlmError::MissingCredentials("anthropic".into()));
        }
        let http = Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()?;
        Ok(Self { http, auth, config })
    }

    /// Construct a client from environment variables.
    pub fn from_env() -> LlmResult<Self> {
        let auth = AuthSource::from_env()?;
        let mut config = LlmConfig::anthropic(
            std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-opus-4-6".to_string()),
        );
        config.base_url = resolve_anthropic_base_url();
        Self::new(auth, config)
    }

    /// Build the request URL.
    fn url(&self) -> String {
        format!("{}/v1/messages", self.config.base_url.trim_end_matches('/'))
    }

    /// Apply authentication headers to a request builder.
    fn apply_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mut b = builder
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json");
        if let Some(api_key) = self.auth.api_key() {
            b = b.header("x-api-key", api_key);
        }
        if let Some(token) = self.auth.bearer_token() {
            b = b.bearer_auth(token);
        }
        b
    }

    /// Translate our [`MessageRequest`] into the Anthropic request body.
    ///
    /// Currently the schemas are close enough that we just pass through, with
    /// a small sanity adjustment for the `system` field which Anthropic expects
    /// as a plain string (we already serialize it that way).
    fn build_body(&self, req: &MessageRequest) -> serde_json::Value {
        serde_json::to_value(req).unwrap_or_else(|_| json!({}))
    }
}

#[async_trait]
impl Provider for AnthropicClient {
    fn kind(&self) -> crate::config::ProviderKind {
        crate::config::ProviderKind::Anthropic
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    async fn send_message(&self, request: MessageRequest) -> LlmResult<MessageResponse> {
        let body = self.build_body(&request);
        debug!(model = %request.model, "anthropic send_message");
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
        let parsed: MessageResponse = response.json().await?;
        Ok(parsed)
    }

    async fn stream_message(&self, request: MessageRequest) -> LlmResult<EventStream> {
        let mut body = self.build_body(&request);
        if let Some(obj) = body.as_object_mut() {
            obj.insert("stream".into(), json!(true));
        }
        debug!(model = %request.model, "anthropic stream_message");
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
        let provider_label = self.config.model.clone();
        let stream = async_stream::try_stream! {
            let mut parser = SseParser::new();
            let mut byte_stream = byte_stream;
            while let Some(chunk) = byte_stream.next().await {
                let chunk = chunk.map_err(LlmError::Http)?;
                let events = parser.push(&chunk)?;
                for ev in events {
                    yield ev;
                }
            }
            let trailing = parser.finish()?;
            for ev in trailing {
                yield ev;
            }
            let _ = provider_label;
        };

        Ok(Box::pin(stream))
    }
}

/// Wrap the upstream's `message_start`/`content_block_*` event stream into
/// the high-level `AssistantEvent` stream consumed by the runtime. This is
/// unused for now but kept as a public helper for future integrations.
pub fn events_to_assistant(events: Vec<StreamEvent>) -> Vec<crate::events::AssistantEvent> {
    use crate::events::AssistantEvent;
    let mut out = Vec::new();
    let mut current_tool: Option<(String, String, String)> = None;
    for ev in events {
        match ev {
            StreamEvent::ContentBlockDelta {
                delta: ContentDelta::TextDelta { text },
                ..
            } => out.push(AssistantEvent::TextDelta(text)),
            StreamEvent::ContentBlockStart {
                block: crate::events::ContentBlock::ToolUse { id, name },
                ..
            } => {
                current_tool = Some((id, name, String::new()));
            }
            StreamEvent::ContentBlockDelta {
                delta: ContentDelta::InputJsonDelta { partial_json },
                index: _,
            } => {
                if let Some(t) = current_tool.as_mut() {
                    t.2.push_str(&partial_json);
                }
            }
            StreamEvent::ContentBlockStop { .. } => {
                if let Some((id, name, raw)) = current_tool.take() {
                    let input: serde_json::Value = serde_json::from_str(&raw).unwrap_or(json!({}));
                    out.push(AssistantEvent::ToolUse { id, name, input });
                }
            }
            StreamEvent::MessageDelta { stop_reason } => {
                out.push(AssistantEvent::TurnComplete {
                    stop_reason,
                    usage: crate::types::Usage::default(),
                });
            }
            StreamEvent::MessageStop => {
                // The MessageDelta already emitted TurnComplete in most cases.
                // Avoid double-emit.
            }
            other => warn!(?other, "unhandled upstream event"),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthSource;
    use crate::types::InputMessage;

    #[test]
    fn missing_credentials_rejected() {
        let cfg = LlmConfig::anthropic("claude-opus-4-6");
        let r = AnthropicClient::new(AuthSource::None, cfg);
        assert!(r.is_err());
    }

    #[test]
    fn build_body_passes_through() {
        let client = AnthropicClient::new(
            AuthSource::with_api_key("sk-test"),
            LlmConfig::anthropic("claude-opus-4-6"),
        )
        .unwrap();
        let req = MessageRequest {
            model: "claude-opus-4-6".into(),
            max_tokens: 1024,
            messages: vec![InputMessage::user_text("hi")],
            ..Default::default()
        };
        let body = client.build_body(&req);
        assert_eq!(body["model"], "claude-opus-4-6");
        assert_eq!(body["max_tokens"], 1024);
        assert_eq!(body["messages"][0]["role"], "user");
    }
}
