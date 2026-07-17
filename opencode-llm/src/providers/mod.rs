//! LLM provider clients.
//!
//! Each provider implements the [`Provider`] trait, which exposes:
//!
//! - [`send_message`](Provider::send_message) — non-streaming message send
//! - [`stream_message`](Provider::stream_message) — streaming message send
//!
//! Providers are constructed via their respective `new` constructors and
//! authenticated via [`AuthSource`](crate::auth::AuthSource).

pub mod anthropic;
pub mod openai_compat;

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use crate::config::ProviderKind;
use crate::error::LlmResult;
use crate::events::StreamEvent;
use crate::types::{MessageRequest, MessageResponse};

/// A boxed stream of [`StreamEvent`]s.
pub type EventStream =
    Pin<Box<dyn Stream<Item = LlmResult<StreamEvent>> + Send + Sync + 'static>>;

/// Common interface for an LLM provider.
#[async_trait]
pub trait Provider: Send + Sync {
    /// The provider kind enum value.
    fn kind(&self) -> ProviderKind;

    /// The model identifier this provider is configured for.
    fn model(&self) -> &str;

    /// Send a non-streaming request and return the full response.
    async fn send_message(&self, request: MessageRequest) -> LlmResult<MessageResponse>;

    /// Send a streaming request and return a stream of events.
    async fn stream_message(&self, request: MessageRequest) -> LlmResult<EventStream>;
}
