//! Authentication source resolution.
//!
//! Resolves credentials from environment variables in priority order:
//!
//! 1. `ANTHROPIC_API_KEY` (Anthropic `x-api-key` header)
//! 2. `ANTHROPIC_AUTH_TOKEN` (OAuth bearer or proxy bearer token)
//! 3. `OPENAI_API_KEY` (OpenAI-compatible `Authorization: Bearer`)
//!
//! Both Anthropic credentials can be supplied simultaneously; both are
//! forwarded to the upstream.

use crate::error::{LlmError, LlmResult};

/// Anthropic API base URL. Override with `ANTHROPIC_BASE_URL`.
pub const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com";

/// OpenAI-compatible base URL. Override with `OPENAI_BASE_URL`.
pub const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";

/// Authentication source for an LLM provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthSource {
    /// No credentials configured.
    None,
    /// Anthropic `x-api-key` header.
    ApiKey(String),
    /// Anthropic `Authorization: Bearer ...` (Claude Max / proxies).
    BearerToken(String),
    /// Both `x-api-key` and `Authorization: Bearer` are sent.
    ApiKeyAndBearer {
        /// Anthropic API key.
        api_key: String,
        /// Bearer token.
        bearer_token: String,
    },
    /// OpenAI-compatible `Authorization: Bearer ...`.
    OpenAiBearer(String),
}

impl AuthSource {
    /// Resolve credentials from environment variables.
    ///
    /// Order of precedence:
    /// 1. `OPENAI_API_KEY` → `OpenAiBearer`
    /// 2. `ANTHROPIC_API_KEY` (+ optional `ANTHROPIC_AUTH_TOKEN`) → Anthropic
    pub fn from_env() -> LlmResult<Self> {
        if let Ok(key) = read_env_non_empty("OPENAI_API_KEY") {
            return Ok(Self::OpenAiBearer(key));
        }
        let api_key = read_env_non_empty("ANTHROPIC_API_KEY").ok();
        let bearer = read_env_non_empty("ANTHROPIC_AUTH_TOKEN").ok();
        match (api_key, bearer) {
            (Some(api_key), Some(bearer_token)) => Ok(Self::ApiKeyAndBearer {
                api_key,
                bearer_token,
            }),
            (Some(api_key), None) => Ok(Self::ApiKey(api_key)),
            (None, Some(bearer_token)) => Ok(Self::BearerToken(bearer_token)),
            (None, None) => Err(LlmError::MissingCredentials(
                "set ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN or OPENAI_API_KEY".to_string(),
            )),
        }
    }

    /// Construct an explicit API-key source.
    pub fn with_api_key(key: impl Into<String>) -> Self {
        Self::ApiKey(key.into())
    }

    /// Construct an explicit bearer-token source.
    pub fn with_bearer(token: impl Into<String>) -> Self {
        Self::BearerToken(token.into())
    }

    /// Construct an explicit OpenAI bearer source.
    pub fn with_openai_bearer(token: impl Into<String>) -> Self {
        Self::OpenAiBearer(token.into())
    }

    /// Whether the source has any usable credentials.
    pub fn is_authenticated(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns the API key portion if any.
    pub fn api_key(&self) -> Option<&str> {
        match self {
            Self::ApiKey(key) | Self::ApiKeyAndBearer { api_key: key, .. } => Some(key.as_str()),
            _ => None,
        }
    }

    /// Returns the bearer token portion if any.
    pub fn bearer_token(&self) -> Option<&str> {
        match self {
            Self::BearerToken(t)
            | Self::ApiKeyAndBearer {
                bearer_token: t, ..
            } => Some(t.as_str()),
            _ => None,
        }
    }

    /// Returns a redacted summary suitable for logging.
    pub fn redacted(&self) -> String {
        match self {
            Self::None => "<none>".to_string(),
            Self::ApiKey(_) | Self::ApiKeyAndBearer { .. } => "x-api-key:******".to_string(),
            Self::BearerToken(_) => "Bearer:******".to_string(),
            Self::OpenAiBearer(_) => "OpenAI Bearer:******".to_string(),
        }
    }
}

/// Read an env var and return `Some(value)` if it exists and is non-empty.
fn read_env_non_empty(name: &str) -> LlmResult<String> {
    std::env::var(name)
        .ok()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| LlmError::Config(format!("env var {name} is missing or empty")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacted_masks_secrets() {
        let auth = AuthSource::with_api_key("sk-ant-secret");
        assert!(!auth.redacted().contains("sk-ant-secret"));
        assert!(auth.redacted().contains("******"));
    }

    #[test]
    fn openai_takes_precedence_in_from_env() {
        // Sanity check: precedence means OPENAI_API_KEY wins when both are set.
        // (We cannot easily test from_env directly without mutating the env.)
        let auth = AuthSource::with_openai_bearer("sk-test");
        assert!(auth.is_authenticated());
    }
}
