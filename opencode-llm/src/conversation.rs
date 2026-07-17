//! Conversation runtime — the main interaction loop.
//!
//! [`ConversationRuntime`] manages the multi-turn conversation with an LLM
//! provider: it holds message history, dispatches tool calls back to the
//! executor, and terminates when the model produces a final response.

use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::cache::{compaction_notice, PromptCache, CompactionStrategy};
use crate::error::LlmResult;
use crate::permissions::{PermissionDecision, PermissionEnforcer};
use crate::providers::Provider;
use crate::tools::{ToolContext, ToolExecutor, ToolSpec};
use crate::types::{
    InputContentBlock, InputMessage, MessageRequest, OutputContentBlock,
    ToolResultContentBlock,
};

/// Maximum number of tool-call rounds before giving up.
const MAX_TOOL_ROUNDS: usize = 64;

/// Runtime builder.
pub struct ConversationRuntimeBuilder {
    system: Option<String>,
    model: Option<String>,
    max_tokens: u32,
    temperature: Option<f64>,
    top_p: Option<f64>,
    tools: Vec<ToolSpec>,
    executors: BTreeMap<String, ToolExecutor>,
    tool_context: ToolContext,
    max_tool_rounds: usize,
    prompt_cache: Option<PromptCache>,
}

impl Default for ConversationRuntimeBuilder {
    fn default() -> Self {
        Self {
            system: None,
            model: None,
            max_tokens: 8192,
            temperature: None,
            top_p: None,
            tools: Vec::new(),
            executors: BTreeMap::new(),
            tool_context: ToolContext::default(),
            max_tool_rounds: MAX_TOOL_ROUNDS,
            prompt_cache: None,
        }
    }
}

impl ConversationRuntimeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the system prompt.
    pub fn system(mut self, s: impl Into<String>) -> Self {
        self.system = Some(s.into());
        self
    }

    /// Override the model name.
    pub fn model(mut self, m: impl Into<String>) -> Self {
        self.model = Some(m.into());
        self
    }

    /// Set max output tokens.
    pub fn max_tokens(mut self, v: u32) -> Self {
        self.max_tokens = v;
        self
    }

    /// Set temperature.
    pub fn temperature(mut self, v: f64) -> Self {
        self.temperature = Some(v);
        self
    }

    /// Set top_p.
    pub fn top_p(mut self, v: f64) -> Self {
        self.top_p = Some(v);
        self
    }

    /// Register a single tool.
    pub fn tool(mut self, spec: ToolSpec, executor: ToolExecutor) -> Self {
        let name = spec.name.clone();
        self.tools.push(spec);
        self.executors.insert(name, executor);
        self
    }

    /// Register all MVP tools.
    pub fn mvp_tools(mut self) -> Self {
        let specs = crate::tools::mvp_tool_specs();
        let executors = crate::tools::mvp_tool_executors();
        self.tools = specs;
        self.executors = executors;
        self
    }

    /// Set tool execution context.
    pub fn tool_context(mut self, ctx: ToolContext) -> Self {
        self.tool_context = ctx;
        self
    }

    /// Set maximum tool-call rounds.
    pub fn max_tool_rounds(mut self, v: usize) -> Self {
        self.max_tool_rounds = v;
        self
    }

    /// Enable automatic compaction with the given token budget.
    pub fn auto_compact(mut self, max_tokens: usize) -> Self {
        self.prompt_cache = Some(PromptCache::new(max_tokens));
        self
    }

    /// Set the compaction strategy.
    pub fn compaction_strategy(mut self, strategy: CompactionStrategy) -> Self {
        let budget = self
            .prompt_cache
            .as_ref()
            .map(|c| c.max_tokens())
            .unwrap_or(128_000);
        self.prompt_cache = Some(PromptCache::new(budget).with_strategy(strategy));
        self
    }

    /// Build the runtime.
    pub fn build<P: Provider + 'static>(self, provider: Arc<P>) -> ConversationRuntime<P> {
        ConversationRuntime {
            provider,
            system: self.system.unwrap_or_default(),
            model: self.model,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            top_p: self.top_p,
            tools: self.tools,
            executors: self.executors,
            tool_context: self.tool_context,
                permission_enforcer: None,
                max_tool_rounds: self.max_tool_rounds,
                prompt_cache: self.prompt_cache,
                history: Arc::new(Mutex::new(Vec::new())),
            }
        }
}

/// Multi-turn conversation runtime.
pub struct ConversationRuntime<P: Provider + 'static> {
    provider: Arc<P>,
    system: String,
    model: Option<String>,
    max_tokens: u32,
    temperature: Option<f64>,
        top_p: Option<f64>,
    tools: Vec<ToolSpec>,
    executors: BTreeMap<String, ToolExecutor>,
    tool_context: ToolContext,
    permission_enforcer: Option<Arc<PermissionEnforcer>>,
    max_tool_rounds: usize,
        history: Arc<Mutex<Vec<InputMessage>>>,
        prompt_cache: Option<PromptCache>,
}

    impl<P: Provider + 'static> ConversationRuntime<P> {
        /// Access the builder.
        pub fn builder() -> ConversationRuntimeBuilder {
            ConversationRuntimeBuilder::new()
        }

        /// Set a permission enforcer (must be called before run).
        pub fn with_permissions(mut self, enf: Arc<PermissionEnforcer>) -> Self {
            self.permission_enforcer = Some(enf);
            self
        }

    /// Run a single user-turn, executing tool calls automatically until the
    /// model produces a final text response.
    pub async fn run(&self, user_message: &str) -> LlmResult<String> {
        let mut round = 0usize;
        let mut accumulated_text = String::new();
        // Add user message to history.
        {
            let mut hist = self.history.lock().await;
            hist.push(InputMessage::user_text(user_message));
        }

        loop {
            if round >= self.max_tool_rounds {
                warn!(round, "exceeded max tool rounds");
                break;
            }
            round += 1;

            // Compact history if needed.
            if let Some(ref cache) = self.prompt_cache {
                if cache.needs_compaction() {
                    self.compact_history().await;
                }
            }

            let request = self.build_request()?;
            let response = self.provider.send_message(request).await?;

            // Record assistant turn.
            {
                let mut hist = self.history.lock().await;
                hist.push(InputMessage::assistant_from_blocks(&response.content));
            }

            // Inspect response content.
            let mut tool_calls = Vec::new();
            for block in &response.content {
                match block {
                    OutputContentBlock::Text { text } => {
                        accumulated_text.push_str(text);
                    }
                    OutputContentBlock::ToolUse { id, name, input } => {
                        tool_calls.push((id.clone(), name.clone(), input.clone()));
                    }
                }
            }

            if tool_calls.is_empty() {
                info!(round, tool_calls=0, "conversation finished");
                break; // No tool calls → final answer.
            }

            info!(round, tool_calls=tool_calls.len(), "executing tool calls");

            self.execute_tool_calls_inner(&tool_calls, &mut accumulated_text).await;
        }

        Ok(accumulated_text)
    }

    /// Run a single user-turn with streaming, sending text deltas through a channel.
    ///
    /// The first round streams text deltas via `tx`. Subsequent tool-call rounds
    /// are non-streaming. The final accumulated text is returned.
    pub async fn stream_to_channel(
        &self,
        user_message: &str,
        tx: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> LlmResult<String> {
        let mut round = 0usize;
        let mut accumulated_text = String::new();
        {
            let mut hist = self.history.lock().await;
            hist.push(InputMessage::user_text(user_message));
        }

        loop {
            if round >= self.max_tool_rounds {
                warn!(round, "exceeded max tool rounds");
                break;
            }
            round += 1;

            if let Some(ref cache) = self.prompt_cache {
                if cache.needs_compaction() {
                    self.compact_history().await;
                }
            }

            if round == 1 {
                // Streaming round.
                let request = self.build_streaming_request()?;
                let mut stream = self.provider.stream_message(request).await?;
                let mut assistant_blocks: Vec<OutputContentBlock> = Vec::new();
                let mut current_block_text = String::new();

                use futures::StreamExt;
                while let Some(event) = stream.next().await {
                    let event = event?;
                    match &event {
                        crate::events::StreamEvent::ContentBlockDelta { delta, .. } => {
                            if let crate::events::ContentDelta::TextDelta { text } = delta {
                                current_block_text.push_str(text);
                                accumulated_text.push_str(text);
                                let _ = tx.send(text.clone());
                            }
                        }
                        crate::events::StreamEvent::ContentBlockStop { .. } => {
                            if !current_block_text.is_empty() {
                                assistant_blocks.push(OutputContentBlock::Text {
                                    text: std::mem::take(&mut current_block_text),
                                });
                            }
                        }
                        crate::events::StreamEvent::MessageStop => break,
                        crate::events::StreamEvent::ContentBlockStart {
                            block: crate::events::ContentBlock::ToolUse { id, name },
                            ..
                        } => {
                            if !current_block_text.is_empty() {
                                assistant_blocks.push(OutputContentBlock::Text {
                                    text: std::mem::take(&mut current_block_text),
                                });
                            }
                            assistant_blocks.push(OutputContentBlock::ToolUse {
                                id: id.clone(),
                                name: name.clone(),
                                input: serde_json::Value::Null,
                            });
                        }
                        _ => {}
                    }
                }

                {
                    let mut hist = self.history.lock().await;
                    hist.push(InputMessage::assistant_from_blocks(&assistant_blocks));
                }

                let mut tool_calls = Vec::new();
                for block in &assistant_blocks {
                    if let OutputContentBlock::ToolUse { id, name, input } = block {
                        tool_calls.push((id.clone(), name.clone(), input.clone()));
                    }
                }

                if tool_calls.is_empty() {
                    info!(round, tool_calls=0, "conversation finished (streaming)");
                    break;
                }

                info!(round, tool_calls=tool_calls.len(), "executing tool calls (post-stream)");
                self.execute_tool_calls_inner(&tool_calls, &mut accumulated_text).await;
            } else {
                // Non-streaming rounds (tool results → model).
                let request = self.build_request()?;
                let response = self.provider.send_message(request).await?;

                {
                    let mut hist = self.history.lock().await;
                    hist.push(InputMessage::assistant_from_blocks(&response.content));
                }

                let mut tool_calls = Vec::new();
                for block in &response.content {
                    match block {
                        OutputContentBlock::Text { text } => {
                            accumulated_text.push_str(text);
                        }
                        OutputContentBlock::ToolUse { id, name, input } => {
                            tool_calls.push((id.clone(), name.clone(), input.clone()));
                        }
                    }
                }

                if tool_calls.is_empty() {
                    info!(round, tool_calls=0, "conversation finished (non-streaming)");
                    break;
                }

                info!(round, tool_calls=tool_calls.len(), "executing tool calls");
                self.execute_tool_calls_inner(&tool_calls, &mut accumulated_text).await;
            }
        }

        Ok(accumulated_text)
    }

    /// Execute tool calls and add results to history (shared by run and stream_to_channel).
    async fn execute_tool_calls_inner(
        &self,
        tool_calls: &[(String, String, serde_json::Value)],
        _accumulated_text: &mut String,
    ) {
        let mut results = Vec::new();
        for (_tool_id, name, input) in tool_calls {
            if let Some(ref enf) = self.permission_enforcer {
                let desc = format!("execute tool `{name}` with args: {input}");
                match enf.evaluate(name, &desc) {
                    PermissionDecision::Denied { reason } => {
                        results.push(ToolResultContentBlock::Text {
                            text: format!("[permission denied: {reason}]"),
                        });
                        continue;
                    }
                    PermissionDecision::NeedsConfirmation { target, description } => {
                        results.push(ToolResultContentBlock::Text {
                            text: format!("[permission needed: `{target}` — {description}]"),
                        });
                        continue;
                    }
                    PermissionDecision::Allowed { .. } => {}
                }
            }
            let result = match self.executors.get(name) {
                Some(exec) => match exec.execute(input.clone(), &self.tool_context).await {
                    Ok(output) => output,
                    Err(e) => format!("[tool error: {e}]"),
                },
                None => format!("[unknown tool: {name}]"),
            };
            results.push(ToolResultContentBlock::Text { text: result });
        }

        let mut hist = self.history.lock().await;
        for ((tool_id, _name, _input), result) in tool_calls.iter().zip(results.iter()) {
            let block = InputContentBlock::ToolResult {
                tool_use_id: tool_id.clone(),
                content: vec![result.clone()],
                is_error: false,
            };
            hist.push(InputMessage::new("user", vec![block]));
        }
    }

    /// Build a [`MessageRequest`] from the current history and config.
    fn build_request(&self) -> LlmResult<MessageRequest> {
        let history = self.history.blocking_lock();
        let model = self
            .model
            .clone()
            .unwrap_or_else(|| self.provider.model().to_string());
        Ok(MessageRequest {
            model,
            messages: history.clone(),
            system: if self.system.is_empty() {
                None
            } else {
                Some(self.system.clone())
            },
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            top_p: self.top_p,
            stop: None,
            tools: Some(self.tools.iter().map(|s| s.to_definition()).collect()),
            tool_choice: None,
            stream: false,
            extra_body: BTreeMap::new(),
        })
    }

    /// Build a streaming [`MessageRequest`] (sets `stream: true`).
    fn build_streaming_request(&self) -> LlmResult<MessageRequest> {
        let mut req = self.build_request()?;
        req.stream = true;
        Ok(req)
    }

    /// Append an external message (e.g. from a previous session).
    pub async fn push_message(&self, msg: InputMessage) {
        self.history.lock().await.push(msg);
    }

    /// Access the current message history (clone).
    pub async fn history(&self) -> Vec<InputMessage> {
        self.history.lock().await.clone()
    }

    /// Clear the message history.
    pub async fn clear_history(&self) {
        self.history.lock().await.clear();
        if let Some(ref cache) = self.prompt_cache {
            cache.reset();
        }
    }

    /// Record a message in the prompt cache (estimates tokens).
    fn maybe_record_message(&self, msg: &str) {
        if let Some(ref cache) = self.prompt_cache {
            cache.record_message(msg);
        }
    }

    /// Compact history using the prompt cache.
    async fn compact_history(&self) {
        let texts: Vec<String> = {
            let hist = self.history.lock().await;
            hist.iter()
                .filter_map(|m| {
                    m.content.first().map(|b| match b {
                        crate::types::InputContentBlock::Text { text } => text.clone(),
                        _ => String::new(),
                    })
                })
                .collect()
        };
        if texts.len() <= 6 {
            return;
        }
        let (blocks, preserve) = match self.prompt_cache {
            Some(ref cache) => cache.compact(&texts),
            None => return,
        };
        if blocks.is_empty() {
            return;
        }
        // Replace history with a compacted system message + recent messages.
        let notice = compaction_notice(&blocks);
        let compact_msg = InputMessage::new("user", vec![crate::types::InputContentBlock::Text { text: notice }]);
        let mut hist = self.history.lock().await;
        let keep_from = hist.len().saturating_sub(preserve);
        let recent: Vec<InputMessage> = hist.drain(keep_from..).collect();
        *hist = vec![compact_msg];
        hist.extend(recent);
        if let Some(ref cache) = self.prompt_cache {
            cache.mark_compacted();
        }
    }

    /// Return the provider reference.
    pub fn provider(&self) -> &Arc<P> {
        &self.provider
    }
}
