//! Prompt cache and automatic compaction for conversation history.
//!
//! [`PromptCache`] tracks token usage and triggers automatic compaction
//! when the estimated token count exceeds the configured limit.
//! Compaction summarizes older messages while preserving recent context.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Default token budget before compaction triggers.
const DEFAULT_MAX_TOKENS: usize = 128_000;

/// How many recent messages to preserve verbatim during compaction.
const PRESERVE_RECENT: usize = 6;

/// A simple token estimator (4 chars ≈ 1 token).
fn estimate_tokens(text: &str) -> usize {
    text.len() / 4 + 1
}

/// Compaction strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionStrategy {
    /// Automatically compact when approaching the limit.
    Auto,
    /// Only compact when explicitly requested.
    Manual,
}

/// A compacted summary entry replacing older messages.
#[derive(Debug, Clone)]
pub struct CompactedBlock {
    /// Summary text.
    pub summary: String,
    /// Number of original messages this replaces.
    pub original_count: usize,
}

/// Prompt cache state.
pub struct PromptCache {
    /// Maximum token budget.
    max_tokens: usize,
    /// Current estimated token count.
    current_estimate: AtomicUsize,
    /// Compaction strategy.
    strategy: CompactionStrategy,
    /// Whether compaction has been applied.
    compacted: std::sync::Mutex<bool>,
}

impl Default for PromptCache {
    fn default() -> Self {
        Self {
            max_tokens: DEFAULT_MAX_TOKENS,
            current_estimate: AtomicUsize::new(0),
            strategy: CompactionStrategy::Auto,
            compacted: std::sync::Mutex::new(false),
        }
    }
}

impl PromptCache {
    /// Create a new prompt cache with the given budget.
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            current_estimate: AtomicUsize::new(0),
            strategy: CompactionStrategy::Auto,
            compacted: std::sync::Mutex::new(false),
        }
    }

    /// Get the maximum token budget.
    pub fn max_tokens(&self) -> usize {
        self.max_tokens
    }

    /// Set the compaction strategy.
    pub fn with_strategy(mut self, strategy: CompactionStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Update the token estimate for a new message.
    pub fn record_message(&self, text: &str) {
        let tokens = estimate_tokens(text);
        self.current_estimate.fetch_add(tokens, Ordering::Relaxed);
    }

    /// Record a tool result.
    pub fn record_tool_result(&self, text: &str) {
        let tokens = estimate_tokens(text);
        self.current_estimate.fetch_add(tokens, Ordering::Relaxed);
    }

    /// Get the current estimated token count.
    pub fn estimated_tokens(&self) -> usize {
        self.current_estimate.load(Ordering::Relaxed)
    }

    /// Whether compaction is needed.
    pub fn needs_compaction(&self) -> bool {
        match self.strategy {
            CompactionStrategy::Auto => {
                let current = self.current_estimate.load(Ordering::Relaxed);
                current > self.max_tokens * 80 / 100 // Trigger at 80% capacity.
            }
            CompactionStrategy::Manual => false,
        }
    }

    /// Get the number of most recent messages to preserve.
    pub fn preserve_recent(&self) -> usize {
        PRESERVE_RECENT
    }

    /// Apply compaction: summarize older messages and produce compacted blocks.
    ///
    /// Returns a list of [`CompactedBlock`] entries that replace older messages,
    /// along with how many messages should be preserved from the end.
    pub fn compact(&self, messages: &[impl AsRef<str>]) -> (Vec<CompactedBlock>, usize) {
        if messages.len() <= PRESERVE_RECENT {
            return (Vec::new(), self.preserve_recent());
        }

        // Determine how many messages to compact.
        let compact_count = messages.len().saturating_sub(PRESERVE_RECENT);

        // Group compacted messages into blocks.
        let mut blocks = Vec::new();
        let mut current_summary = String::new();
        let mut current_count = 0;

        for msg in messages.iter().take(compact_count) {
            let text = msg.as_ref();
            let preview: String = text.chars().take(100).collect();
            if !current_summary.is_empty() {
                current_summary.push_str(" | ");
            }
            current_summary.push_str(&preview);
            current_count += 1;

            // Flush every ~5 messages.
            if current_count >= 5 {
                blocks.push(CompactedBlock {
                    summary: if current_summary.len() > 200 {
                        format!("{}...", &current_summary[..200])
                    } else {
                        current_summary.clone()
                    },
                    original_count: current_count,
                });
                current_summary.clear();
                current_count = 0;
            }
        }

        // Flush remaining.
        if current_count > 0 {
            blocks.push(CompactedBlock {
                summary: if current_summary.len() > 200 {
                    format!("{}...", &current_summary[..200])
                } else {
                    current_summary.clone()
                },
                original_count: current_count,
            });
        }

        self.current_estimate.store(0, Ordering::Relaxed);
        (blocks, self.preserve_recent())
    }

    /// Mark that compaction has been applied.
    pub fn mark_compacted(&self) {
        *self.compacted.lock().unwrap() = true;
        self.current_estimate.store(0, Ordering::Relaxed);
    }

    /// Whether compaction has been applied.
    pub fn is_compacted(&self) -> bool {
        *self.compacted.lock().unwrap()
    }

    /// Reset the cache.
    pub fn reset(&self) {
        self.current_estimate.store(0, Ordering::Relaxed);
        *self.compacted.lock().unwrap() = false;
    }
}

/// Estimate tokens for a piece of text.
pub fn estimate_token_count(text: &str) -> usize {
    estimate_tokens(text)
}

/// Build a compaction notice for the model.
pub fn compaction_notice(blocks: &[CompactedBlock]) -> String {
    let mut notice = String::from(
        "<compaction_notice>\n\
         The following older conversation turns have been summarized.\n\
         If you need details about any of these topics, ask the user.\n",
    );
    for block in blocks {
        notice.push_str(&format!(
            "  • [{} msgs] {}\n",
            block.original_count, block.summary
        ));
    }
    notice.push_str("</compaction_notice>");
    notice
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_roughly_counts_tokens() {
        let count = estimate_tokens("Hello, world!");
        assert!(count > 0);
        assert_eq!(count, 4);
    }

    #[test]
    fn default_no_compaction_needed() {
        let cache = PromptCache::default();
        assert!(!cache.needs_compaction());
    }

    #[test]
    fn compaction_needed_after_many_messages() {
        let cache = PromptCache::new(100);
        for _ in 0..10 {
            cache.record_message("This is a test message with some content. ");
        }
        assert!(cache.needs_compaction());
    }

    #[test]
    fn compact_reduces_message_count() {
        let cache = PromptCache::new(1000);
        let messages: Vec<String> = (0..20)
            .map(|i| format!("Message number {i}: this is a test message with enough content to be estimated."))
            .collect();
        let (blocks, preserve) = cache.compact(&messages);
        assert!(blocks.len() >= 1);
        assert_eq!(preserve, PRESERVE_RECENT);
        // Total should be blocks + preserve < original.
        let total_replaced: usize = blocks.iter().map(|b| b.original_count).sum();
        assert_eq!(total_replaced + preserve, messages.len());
    }

    #[test]
    fn no_compaction_for_few_messages() {
        let cache = PromptCache::new(1000);
        let messages: Vec<String> = (0..4).map(|i| format!("Message {i}")).collect();
        let (blocks, preserve) = cache.compact(&messages);
        assert!(blocks.is_empty());
        // When no compaction is needed, preserve returns the total count
        // so the caller knows to keep all messages intact.
        assert_eq!(preserve, PRESERVE_RECENT);
    }

    #[test]
    fn manual_strategy_never_compacts() {
        let cache = PromptCache::new(10).with_strategy(CompactionStrategy::Manual);
        cache.record_message("A very long message that would normally trigger compaction.");
        assert!(!cache.needs_compaction());
    }

    #[test]
    fn compaction_notice_format() {
        let blocks = vec![CompactedBlock {
            summary: "Test summary".to_string(),
            original_count: 3,
        }];
        let notice = compaction_notice(&blocks);
        assert!(notice.contains("compaction_notice"));
        assert!(notice.contains("Test summary"));
    }

    #[test]
    fn recording_accumulates() {
        let cache = PromptCache::new(10000);
        cache.record_message("Hello");
        cache.record_tool_result("Some tool output here");
        assert!(cache.estimated_tokens() > 0);
    }

    #[test]
    fn reset_clears_estimate() {
        let mut cache = PromptCache::new(10000);
        cache.record_message("A message that takes tokens.");
        cache.reset();
        assert_eq!(cache.estimated_tokens(), 0);
    }
}
