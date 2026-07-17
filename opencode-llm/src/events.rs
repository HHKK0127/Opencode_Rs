//! Stream events emitted by provider clients.
//!
//! The `Stream` returned by a provider yields a series of [`StreamEvent`] values
//! that the [`ConversationRuntime`](crate::conversation::ConversationRuntime)
//! consumes to update its internal state.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{ToolUseBlock, Usage};

/// Events emitted by an LLM provider during a streamed turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// Stream started; carries the message ID and initial usage.
    MessageStart {
        /// Response identifier.
        id: String,
        /// Model that produced the response.
        model: String,
        /// Initial usage (input tokens etc).
        usage: Usage,
    },
    /// A new content block has begun.
    ContentBlockStart {
        /// Index of the new block.
        index: u32,
        /// The block — currently either text or tool use.
        block: ContentBlock,
    },
    /// Incremental delta within a content block.
    ContentBlockDelta {
        /// Index of the block being updated.
        index: u32,
        /// The delta payload.
        delta: ContentDelta,
    },
    /// A content block finished.
    ContentBlockStop {
        /// Index of the block that finished.
        index: u32,
    },
    /// A top-level message delta (currently used for stop_reason).
    MessageDelta {
        /// Stop reason emitted by the provider.
        stop_reason: Option<String>,
    },
    /// Stream finished normally.
    MessageStop,
    /// Non-fatal ping (keep-alive).
    Ping,
}

/// A content block as it appears in a `ContentBlockStart` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Empty text block — model is about to stream text deltas.
    Text,
    /// Empty tool-use block — model is about to stream tool-input JSON deltas.
    ToolUse {
        /// Tool invocation ID.
        id: String,
        /// Tool name.
        name: String,
    },
}

/// A delta within a content block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentDelta {
    /// Incremental text.
    TextDelta {
        /// The new text.
        text: String,
    },
    /// Incremental JSON input for a tool call.
    InputJsonDelta {
        /// Partial JSON.
        partial_json: String,
    },
}

/// Convenience: extract just the text deltas in order.
pub fn collect_text_deltas(events: &[StreamEvent]) -> String {
    let mut out = String::new();
    for ev in events {
        if let StreamEvent::ContentBlockDelta {
            delta: ContentDelta::TextDelta { text },
            ..
        } = ev
        {
            out.push_str(text);
        }
    }
    out
}

/// Convenience: extract the first tool-use block from the event stream.
pub fn first_tool_use(events: &[StreamEvent]) -> Option<ToolUseBlock> {
    use std::collections::BTreeMap;
    let mut starts: BTreeMap<u32, (String, String)> = BTreeMap::new();
    let mut inputs: BTreeMap<u32, String> = BTreeMap::new();

    for ev in events {
        match ev {
            StreamEvent::ContentBlockStart {
                index,
                block: ContentBlock::ToolUse { id, name },
            } => {
                starts.insert(*index, (id.clone(), name.clone()));
            }
            StreamEvent::ContentBlockDelta {
                index,
                delta: ContentDelta::InputJsonDelta { partial_json },
            } => {
                inputs.entry(*index).or_default().push_str(partial_json);
            }
            _ => {}
        }
    }

    let (index, (id, name)) = starts.into_iter().next()?;
    let raw = inputs.remove(&index).unwrap_or_default();
    let input: Value = if raw.trim().is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&raw).unwrap_or(Value::Null)
    };
    Some(ToolUseBlock { id, name, input })
}

/// Assistant-side events that the runtime emits to the consumer callback.
#[derive(Debug, Clone)]
pub enum AssistantEvent {
    /// A piece of text was produced.
    TextDelta(String),
    /// The assistant invoked a tool.
    ToolUse {
        /// Tool invocation ID.
        id: String,
        /// Tool name.
        name: String,
        /// Tool input.
        input: Value,
    },
    /// The assistant finished its turn.
    TurnComplete {
        /// Stop reason.
        stop_reason: Option<String>,
        /// Token usage for the turn.
        usage: Usage,
    },
}
