//! Configuration types for LLM clients.

use crate::auth::{AuthSource, DEFAULT_ANTHROPIC_BASE_URL, DEFAULT_OPENAI_BASE_URL};

/// Anthropic API version header. Bump when the API contract changes.
pub const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Model family identifiers used for capability and pricing resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelFamily {
    /// Claude Opus 4 family.
    Opus4,
    /// Claude Sonnet 4 family.
    Sonnet4,
    /// Claude Haiku 4.5 family.
    Haiku45,
    /// OpenAI o-series reasoning model.
    Reasoning,
    /// OpenAI GPT-4 family.
    Gpt4,
    /// Generic / unknown.
    Other,
}

impl ModelFamily {
    /// Resolve a family from a model identifier.
    pub fn from_model(model: &str) -> Self {
        let m = model.to_ascii_lowercase();
        if m.contains("opus-4") || m.contains("opus_4") {
            Self::Opus4
        } else if m.contains("sonnet-4") || m.contains("sonnet_4") {
            Self::Sonnet4
        } else if m.contains("haiku-4-5") || m.contains("haiku_4_5") {
            Self::Haiku45
        } else if m.starts_with("o1")
            || m.starts_with("o3")
            || m.starts_with("o4")
            || m.contains("reasoning")
        {
            Self::Reasoning
        } else if m.contains("gpt-4") {
            Self::Gpt4
        } else {
            Self::Other
        }
    }
}

/// Provider kind selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    /// Anthropic native (`api.anthropic.com`).
    Anthropic,
    /// OpenAI-compatible endpoint.
    OpenAiCompat,
}

impl ProviderKind {
    /// Resolve a provider kind from the model identifier and the auth source.
    pub fn detect(model: &str, auth: &AuthSource) -> Self {
        if matches!(auth, AuthSource::OpenAiBearer(_))
            || model.to_ascii_lowercase().contains("gpt-")
            || model.to_ascii_lowercase().contains("o1")
            || model.to_ascii_lowercase().contains("o3")
            || model.to_ascii_lowercase().contains("o4")
        {
            Self::OpenAiCompat
        } else {
            Self::Anthropic
        }
    }
}

/// Configuration for an LLM client.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Provider kind.
    pub provider: ProviderKind,
    /// API base URL.
    pub base_url: String,
    /// Model identifier.
    pub model: String,
    /// Maximum tokens to generate per turn.
    pub max_tokens: u32,
    /// Sampling temperature. `None` means "provider default".
    pub temperature: Option<f64>,
    /// Top-p sampling. `None` means "provider default".
    pub top_p: Option<f64>,
    /// Request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Whether to stream responses by default.
    pub stream: bool,
    /// Maximum number of retries on transient errors.
    pub max_retries: u32,
    /// Initial backoff for retries, in milliseconds.
    pub initial_backoff_ms: u64,
    /// Maximum backoff for retries, in milliseconds.
    pub max_backoff_ms: u64,
    /// System prompt prefix.
    pub system: Option<String>,
}

impl LlmConfig {
    /// Create a config for Anthropic with the given model.
    pub fn anthropic(model: impl Into<String>) -> Self {
        Self {
            provider: ProviderKind::Anthropic,
            base_url: DEFAULT_ANTHROPIC_BASE_URL.to_string(),
            model: model.into(),
            max_tokens: 4096,
            temperature: None,
            top_p: None,
            timeout_ms: 60_000,
            stream: true,
            max_retries: 4,
            initial_backoff_ms: 500,
            max_backoff_ms: 16_000,
            system: None,
        }
    }

    /// Create a config for an OpenAI-compatible endpoint.
    pub fn openai_compat(model: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            provider: ProviderKind::OpenAiCompat,
            base_url: base_url.into(),
            model: model.into(),
            max_tokens: 4096,
            temperature: None,
            top_p: None,
            timeout_ms: 60_000,
            stream: true,
            max_retries: 4,
            initial_backoff_ms: 500,
            max_backoff_ms: 16_000,
            system: None,
        }
    }

    /// Set the system prompt.
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Set max tokens.
    pub fn with_max_tokens(mut self, n: u32) -> Self {
        self.max_tokens = n;
        self
    }

    /// Set temperature.
    pub fn with_temperature(mut self, t: f64) -> Self {
        self.temperature = Some(t);
        self
    }

    /// Disable streaming (returns a single response).
    pub fn non_streaming(mut self) -> Self {
        self.stream = false;
        self
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self::anthropic("claude-opus-4-6")
    }
}

/// Resolve the Anthropic base URL from `ANTHROPIC_BASE_URL`, falling back to the default.
pub fn resolve_anthropic_base_url() -> String {
    std::env::var("ANTHROPIC_BASE_URL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_ANTHROPIC_BASE_URL.to_string())
}

/// Resolve the OpenAI-compatible base URL from `OPENAI_BASE_URL`, falling back to the default.
pub fn resolve_openai_base_url() -> String {
    std::env::var("OPENAI_BASE_URL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_OPENAI_BASE_URL.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_family_resolution() {
        assert_eq!(
            ModelFamily::from_model("claude-opus-4-6"),
            ModelFamily::Opus4
        );
        assert_eq!(
            ModelFamily::from_model("o1-preview"),
            ModelFamily::Reasoning
        );
        assert_eq!(ModelFamily::from_model("gpt-4o"), ModelFamily::Gpt4);
    }

    #[test]
    fn default_is_anthropic() {
        let cfg = LlmConfig::default();
        assert_eq!(cfg.provider, ProviderKind::Anthropic);
    }
}
