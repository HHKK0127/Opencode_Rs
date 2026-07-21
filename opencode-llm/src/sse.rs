//! Server-Sent Events (SSE) parser.
//!
//! Accepts raw bytes from a streaming HTTP response and yields
//! [`StreamEvent`](crate::events::StreamEvent) values once a complete event
//! frame has been received.

use crate::error::{LlmError, LlmResult};
use crate::events::StreamEvent;

/// SSE frame parser. Holds the partial buffer between chunks.
#[derive(Debug, Default)]
pub struct SseParser {
    buffer: Vec<u8>,
}

impl SseParser {
    /// Construct a new parser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a chunk of bytes into the parser and return any complete events.
    pub fn push(&mut self, chunk: &[u8]) -> LlmResult<Vec<StreamEvent>> {
        self.buffer.extend_from_slice(chunk);
        let mut events = Vec::new();

        while let Some(frame) = self.next_frame() {
            if let Some(event) = self.parse_frame(&frame)? {
                events.push(event);
            }
        }

        Ok(events)
    }

    /// Flush any trailing buffered bytes and return the final event, if any.
    pub fn finish(&mut self) -> LlmResult<Vec<StreamEvent>> {
        if self.buffer.is_empty() {
            return Ok(Vec::new());
        }
        let trailing = std::mem::take(&mut self.buffer);
        let frame = String::from_utf8_lossy(&trailing);
        match self.parse_frame(&frame)? {
            Some(event) => Ok(vec![event]),
            None => Ok(Vec::new()),
        }
    }

    fn next_frame(&mut self) -> Option<String> {
        let separator = self
            .buffer
            .windows(2)
            .position(|window| window == b"\n\n")
            .map(|pos| (pos, 2))
            .or_else(|| {
                self.buffer
                    .windows(4)
                    .position(|window| window == b"\r\n\r\n")
                    .map(|pos| (pos, 4))
            })?;

        let (pos, sep_len) = separator;
        let frame_bytes: Vec<u8> = self.buffer.drain(..pos + sep_len).collect();
        let frame_len = frame_bytes.len().saturating_sub(sep_len);
        Some(String::from_utf8_lossy(&frame_bytes[..frame_len]).into_owned())
    }

    fn parse_frame(&self, frame: &str) -> LlmResult<Option<StreamEvent>> {
        let mut data_lines: Vec<&str> = Vec::new();
        let mut event_name: Option<&str> = None;

        for line in frame.lines() {
            if let Some(rest) = line.strip_prefix("event:") {
                event_name = Some(rest.trim());
            } else if let Some(rest) = line.strip_prefix("data:") {
                data_lines.push(rest.trim_start());
            }
            // Ignore "id:", "retry:", comments, and unknown fields.
        }

        if data_lines.is_empty() {
            return Ok(None);
        }

        // An SSE event with `data: [DONE]` signals the end of a stream.
        if data_lines.contains(&"[DONE]") {
            return Ok(Some(StreamEvent::MessageStop));
        }

        let payload = data_lines.join("\n");
        match event_name {
            Some("message_start") | None => {
                let v: serde_json::Value = serde_json::from_str(&payload)
                    .map_err(|e| LlmError::Sse(format!("invalid message_start JSON: {e}")))?;
                let event = crate::events::StreamEvent::MessageStart {
                    id: v
                        .get("message")
                        .and_then(|m| m.get("id"))
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string(),
                    model: v
                        .get("message")
                        .and_then(|m| m.get("model"))
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string(),
                    usage: serde_json::from_value(
                        v.get("message")
                            .and_then(|m| m.get("usage"))
                            .cloned()
                            .unwrap_or(serde_json::json!({})),
                    )
                    .map_err(|e| LlmError::Sse(format!("invalid usage JSON: {e}")))?,
                };
                Ok(Some(event))
            }
            Some("content_block_start") => {
                let v: serde_json::Value = serde_json::from_str(&payload)
                    .map_err(|e| LlmError::Sse(format!("invalid content_block_start JSON: {e}")))?;
                let index =
                    v.get("index").and_then(|x| x.as_u64()).ok_or_else(|| {
                        LlmError::Sse("missing index in content_block_start".into())
                    })? as u32;
                let block = v
                    .get("content_block")
                    .ok_or_else(|| LlmError::Sse("missing content_block".into()))?;
                let block_type = block
                    .get("type")
                    .and_then(|x| x.as_str())
                    .ok_or_else(|| LlmError::Sse("missing content_block.type".into()))?;
                let cb = match block_type {
                    "text" => crate::events::ContentBlock::Text,
                    "tool_use" => crate::events::ContentBlock::ToolUse {
                        id: block
                            .get("id")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string(),
                        name: block
                            .get("name")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string(),
                    },
                    other => {
                        return Err(LlmError::Sse(format!(
                            "unsupported content_block type: {other}"
                        )))
                    }
                };
                Ok(Some(crate::events::StreamEvent::ContentBlockStart {
                    index,
                    block: cb,
                }))
            }
            Some("content_block_delta") => {
                let v: serde_json::Value = serde_json::from_str(&payload)
                    .map_err(|e| LlmError::Sse(format!("invalid content_block_delta JSON: {e}")))?;
                let index =
                    v.get("index").and_then(|x| x.as_u64()).ok_or_else(|| {
                        LlmError::Sse("missing index in content_block_delta".into())
                    })? as u32;
                let delta = v
                    .get("delta")
                    .ok_or_else(|| LlmError::Sse("missing delta in content_block_delta".into()))?;
                let delta_type = delta
                    .get("type")
                    .and_then(|x| x.as_str())
                    .ok_or_else(|| LlmError::Sse("missing delta.type".into()))?;
                let cd = match delta_type {
                    "text_delta" => crate::events::ContentDelta::TextDelta {
                        text: delta
                            .get("text")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string(),
                    },
                    "input_json_delta" => crate::events::ContentDelta::InputJsonDelta {
                        partial_json: delta
                            .get("partial_json")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string(),
                    },
                    other => return Err(LlmError::Sse(format!("unsupported delta type: {other}"))),
                };
                Ok(Some(crate::events::StreamEvent::ContentBlockDelta {
                    index,
                    delta: cd,
                }))
            }
            Some("content_block_stop") => {
                let v: serde_json::Value = serde_json::from_str(&payload)
                    .map_err(|e| LlmError::Sse(format!("invalid content_block_stop JSON: {e}")))?;
                let index =
                    v.get("index").and_then(|x| x.as_u64()).ok_or_else(|| {
                        LlmError::Sse("missing index in content_block_stop".into())
                    })? as u32;
                Ok(Some(crate::events::StreamEvent::ContentBlockStop { index }))
            }
            Some("message_delta") => {
                let v: serde_json::Value = serde_json::from_str(&payload)
                    .map_err(|e| LlmError::Sse(format!("invalid message_delta JSON: {e}")))?;
                let stop_reason = v
                    .get("delta")
                    .and_then(|d| d.get("stop_reason"))
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string());
                Ok(Some(crate::events::StreamEvent::MessageDelta {
                    stop_reason,
                }))
            }
            Some("message_stop") => Ok(Some(StreamEvent::MessageStop)),
            Some("ping") => Ok(Some(StreamEvent::Ping)),
            Some(other) => Err(LlmError::Sse(format!("unknown SSE event: {other}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_message_start() {
        let mut p = SseParser::new();
        let frame = b"event: message_start\ndata: {\"message\":{\"id\":\"msg_1\",\"model\":\"claude-opus-4-6\",\"usage\":{\"input_tokens\":10,\"output_tokens\":0}}}\n\n";
        let events = p.push(frame).unwrap();
        assert_eq!(events.len(), 1);
        match &events[0] {
            StreamEvent::MessageStart { id, model, usage } => {
                assert_eq!(id, "msg_1");
                assert_eq!(model, "claude-opus-4-6");
                assert_eq!(usage.input_tokens, 10);
            }
            _ => panic!("expected MessageStart"),
        }
    }

    #[test]
    fn handles_done_sentinel() {
        let mut p = SseParser::new();
        let events = p.push(b"data: [DONE]\n\n").unwrap();
        assert!(matches!(events.as_slice(), [StreamEvent::MessageStop]));
    }

    #[test]
    fn parses_text_delta() {
        let mut p = SseParser::new();
        let events = p
            .push(
                b"event: content_block_delta\ndata: {\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"hi\"}}\n\n",
            )
            .unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            StreamEvent::ContentBlockDelta {
                delta: crate::events::ContentDelta::TextDelta { text },
                ..
            } if text == "hi"
        ));
    }
}
