//! OAuth token resolution utilities.
//!
//! Supports:
//! - Google Cloud IAM OIDC token exchange (Workload Identity Federation)
//! - GitHub Actions OIDC token exchange
//! - Generic OAuth2 device-code flow

use std::collections::BTreeMap;

use crate::error::{LlmError, LlmResult};

/// Resolved OAuth token with metadata.
#[derive(Debug, Clone)]
pub struct OAuthToken {
    /// The access token.
    pub access_token: String,
    /// Token type (e.g., "Bearer").
    pub token_type: String,
    /// Expiry timestamp (seconds since epoch), if known.
    pub expires_at: Option<u64>,
}

/// A resolved credential that can be converted to an auth header.
#[derive(Debug, Clone)]
pub enum ResolvedCredential {
    /// Static API key.
    ApiKey(String),
    /// Static bearer token.
    BearerToken(String),
    /// OAuth-resolved token.
    OAuth(OAuthToken),
}

impl ResolvedCredential {
    /// Format as an HTTP `Authorization` header value.
    pub fn to_header(&self) -> String {
        match self {
            Self::ApiKey(key) => format!("x-api-key: {key}"),
            Self::BearerToken(token) => format!("Bearer {token}"),
            Self::OAuth(token) => format!("{} {}", token.token_type, token.access_token),
        }
    }

    /// Format as an HTTP header pair (name, value).
    pub fn header_pair(&self) -> (&str, String) {
        match self {
            Self::ApiKey(key) => ("x-api-key", key.clone()),
            Self::BearerToken(token) => ("Authorization", format!("Bearer {token}")),
            Self::OAuth(token) => (
                "Authorization",
                format!("{} {}", token.token_type, token.access_token),
            ),
        }
    }
}

/// Represents which OAuth provider to use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OAuthProvider {
    /// Generic OAuth2 with configurable endpoints.
    Generic {
        /// Authorization endpoint.
        auth_url: String,
        /// Token endpoint.
        token_url: String,
        /// OAuth client ID.
        client_id: String,
    },
    /// Google Cloud IAM OIDC (Workload Identity Federation).
    GcpIam {
        /// The workload identity pool provider resource name.
        audience: String,
    },
    /// GitHub Actions OIDC.
    GithubActions {
        /// The URL to exchange the OIDC JWT for an access token.
        token_url: String,
    },
}

/// OAuth token resolver.
#[derive(Debug, Clone)]
pub struct OAuthResolver {
    /// The provider configuration.
    pub provider: OAuthProvider,
    /// Optional scope for the token request.
    pub scope: Option<String>,
}

impl OAuthResolver {
    /// Create a new resolver for the given provider.
    pub fn new(provider: OAuthProvider) -> Self {
        Self {
            provider,
            scope: None,
        }
    }

    /// Set the OAuth scope.
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Resolve an OAuth token.
    ///
    /// This may block on HTTP requests or subprocess invocations.
    pub async fn resolve(&self) -> LlmResult<OAuthToken> {
        match &self.provider {
            OAuthProvider::Generic {
                auth_url,
                token_url,
                client_id,
            } => self.resolve_generic(auth_url, token_url, client_id).await,
            OAuthProvider::GcpIam { audience } => self.resolve_gcp_iam(audience).await,
            OAuthProvider::GithubActions { token_url } => {
                self.resolve_github_actions(token_url).await
            }
        }
    }

    /// Generic OAuth2 device-code flow.
    async fn resolve_generic(
        &self,
        _auth_url: &str,
        token_url: &str,
        client_id: &str,
    ) -> LlmResult<OAuthToken> {
        // 1. Start device authorization.
        let device_code = self.request_device_code(token_url, client_id).await?;

        // 2. Prompt user to visit URL.
        eprintln!(
            "\n🔐 Open this URL in your browser to authenticate:\n  {}\n\
             Then enter the code: {}\n",
            device_code.verification_uri, device_code.user_code,
        );

        // 3. Poll for token.
        self.poll_token(token_url, client_id, &device_code.device_code)
            .await
    }

    /// Request a device code from the authorization endpoint.
    async fn request_device_code(
        &self,
        token_url: &str,
        client_id: &str,
    ) -> LlmResult<DeviceCodeResponse> {
        let client = reqwest::Client::new();
        let params = [
            ("client_id", client_id),
            ("scope", self.scope.as_deref().unwrap_or("openid")),
        ];
        let auth_url = token_url.replace("/token", "/auth");
        let resp = client
            .post(&auth_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| LlmError::Network(format!("device auth request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(format!(
                "device auth returned {status}: {body}",
            )));
        }

        let body: BTreeMap<String, serde_json::Value> = resp.json().await.map_err(|e| {
            LlmError::ApiError(format!("failed to parse device auth response: {e}"))
        })?;

        Ok(DeviceCodeResponse {
            device_code: body
                .get("device_code")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            user_code: body
                .get("user_code")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            verification_uri: body
                .get("verification_uri")
                .or_else(|| body.get("verification_url"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            interval: body.get("interval").and_then(|v| v.as_u64()).unwrap_or(5),
        })
    }

    /// Poll the token endpoint until the user completes authorization.
    async fn poll_token(
        &self,
        token_url: &str,
        client_id: &str,
        device_code: &str,
    ) -> LlmResult<OAuthToken> {
        let client = reqwest::Client::new();
        let params = [
            ("client_id", client_id),
            ("device_code", device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ];

        // Poll up to 300 seconds (5 min) with 5-second intervals.
        for _ in 0..60 {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            let resp = client
                .post(token_url)
                .form(&params)
                .send()
                .await
                .map_err(|e| LlmError::Network(format!("token poll failed: {e}")))?;

            let status = resp.status();
            let body: BTreeMap<String, serde_json::Value> = resp
                .json()
                .await
                .map_err(|e| LlmError::ApiError(format!("failed to parse token response: {e}")))?;

            if status.is_success() {
                let access_token = body
                    .get("access_token")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        LlmError::ApiError("token response missing access_token".to_string())
                    })?;
                let expires_in = body.get("expires_in").and_then(|v| v.as_u64());
                let expires_at = expires_in.map(|secs| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        + secs
                });
                let token_type = body
                    .get("token_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Bearer")
                    .to_string();
                return Ok(OAuthToken {
                    access_token: access_token.to_string(),
                    token_type,
                    expires_at,
                });
            }

            // authorization_pending is expected; error means something went wrong.
            let error = body.get("error").and_then(|v| v.as_str());
            if let Some(e) = error {
                if e != "authorization_pending" && e != "slow_down" {
                    return Err(LlmError::ApiError(format!("OAuth error: {e}")));
                }
            }
        }

        Err(LlmError::Timeout(
            "OAuth device code flow timed out after 5 minutes".to_string(),
        ))
    }

    /// Resolve a Google Cloud IAM OIDC token via the GCE metadata server.
    async fn resolve_gcp_iam(&self, _audience: &str) -> LlmResult<OAuthToken> {
        // Use the GCE metadata server to get an identity token.
        // If not running on GCE, fall back to gcloud CLI.
        let access_token = match get_gce_identity_token(_audience).await {
            Ok(token) => token,
            Err(_) => get_gcloud_identity_token(_audience)?,
        };

        Ok(OAuthToken {
            access_token,
            token_type: "Bearer".to_string(),
            expires_at: None,
        })
    }

    /// Resolve a GitHub Actions OIDC token.
    async fn resolve_github_actions(&self, token_url: &str) -> LlmResult<OAuthToken> {
        // Try ACTIONS_ID_TOKEN_REQUEST_TOKEN first (most common for GitHub Actions).
        if let Ok(request_token) = std::env::var("ACTIONS_ID_TOKEN_REQUEST_TOKEN") {
            return self.exchange_github_oidc(token_url, &request_token).await;
        }

        // Fall back to ACTIONS_ID_TOKEN_REQUEST_URL.
        if let Ok(url) = std::env::var("ACTIONS_ID_TOKEN_REQUEST_URL") {
            let runtime_token = std::env::var("ACTIONS_RUNTIME_TOKEN").unwrap_or_default();
            let client = reqwest::Client::new();
            let resp = client
                .get(&url)
                .header("Authorization", format!("Bearer {runtime_token}"))
                .send()
                .await
                .map_err(|e| LlmError::Network(format!("failed to get OIDC token: {e}")))?;
            let body: BTreeMap<String, serde_json::Value> = resp.json().await.map_err(|e| {
                LlmError::ApiError(format!("failed to parse OIDC token response: {e}"))
            })?;
            let request_token = body.get("value").and_then(|v| v.as_str()).ok_or_else(|| {
                LlmError::ApiError("OIDC token response missing `value`".to_string())
            })?;
            return self.exchange_github_oidc(token_url, request_token).await;
        }

        Err(LlmError::Config(
            "GitHub Actions OIDC: set ACTIONS_ID_TOKEN_REQUEST_TOKEN or ACTIONS_ID_TOKEN_REQUEST_URL"
                .to_string(),
        ))
    }

    /// Exchange a GitHub OIDC JWT for an access token.
    async fn exchange_github_oidc(
        &self,
        token_url: &str,
        request_token: &str,
    ) -> LlmResult<OAuthToken> {
        let client = reqwest::Client::new();
        let resp = client
            .post(token_url)
            .header("Authorization", format!("Bearer {request_token}"))
            .form(&[(
                "grant_type",
                "urn:ietf:params:oauth:grant-type:token-exchange",
            )])
            .send()
            .await
            .map_err(|e| LlmError::Network(format!("token exchange failed: {e}")))?;

        let body: BTreeMap<String, serde_json::Value> = resp
            .json()
            .await
            .map_err(|e| LlmError::ApiError(format!("failed to parse token exchange: {e}")))?;

        let access_token = body
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LlmError::ApiError("token exchange missing access_token".to_string()))?;

        let expires_in = body.get("expires_in").and_then(|v| v.as_u64());
        let expires_at = expires_in.map(|secs| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                + secs
        });

        Ok(OAuthToken {
            access_token: access_token.to_string(),
            token_type: "Bearer".to_string(),
            expires_at,
        })
    }
}

/// Device-code authorization response.
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[allow(dead_code)]
    interval: u64,
}

/// Try to get a GCE identity token from the metadata server.
async fn get_gce_identity_token(audience: &str) -> LlmResult<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| LlmError::Config(format!("failed to build HTTP client: {e}")))?;

    let url = format!(
        "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/identity?audience={audience}"
    );
    let resp = client
        .get(&url)
        .header("Metadata-Flavor", "Google")
        .send()
        .await
        .map_err(|e| LlmError::Network(format!("GCE metadata request failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(LlmError::Network(format!(
            "GCE metadata returned {}",
            resp.status()
        )));
    }

    resp.text()
        .await
        .map_err(|e| LlmError::Network(format!("failed to read GCE metadata response: {e}")))
}

/// Fall back to `gcloud auth print-identity-token`.
fn get_gcloud_identity_token(audience: &str) -> LlmResult<String> {
    let output = std::process::Command::new("gcloud")
        .args([
            "auth",
            "print-identity-token",
            &format!("--audiences={audience}"),
        ])
        .output()
        .map_err(|e| {
            LlmError::Config(format!("failed to run gcloud CLI (is it installed?): {e}"))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(LlmError::ApiError(format!("gcloud CLI error: {stderr}")));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Resolve credentials from environment variables, with OAuth fallback.
///
/// Priority:
/// 1. Explicit `AuthSource::from_env()` (ANTHROPIC_API_KEY, etc.)
/// 2. `ANTHROPIC_AUTH_TOKEN` as bearer token
/// 3. OAuth resolver (if configured)
pub async fn resolve_credentials(
    oauth_resolver: Option<&OAuthResolver>,
) -> LlmResult<ResolvedCredential> {
    // Try env vars first.
    if let Ok(auth) = crate::auth::AuthSource::from_env() {
        return Ok(match auth {
            crate::auth::AuthSource::ApiKey(key) => ResolvedCredential::ApiKey(key),
            crate::auth::AuthSource::BearerToken(token)
            | crate::auth::AuthSource::ApiKeyAndBearer {
                bearer_token: token,
                ..
            } => ResolvedCredential::BearerToken(token),
            crate::auth::AuthSource::OpenAiBearer(token) => ResolvedCredential::BearerToken(token),
            crate::auth::AuthSource::None => {
                return Err(LlmError::MissingCredentials(
                    "No credentials configured".to_string(),
                ));
            }
        });
    }

    // Try explicit ANTHROPIC_AUTH_TOKEN.
    if let Ok(token) = std::env::var("ANTHROPIC_AUTH_TOKEN") {
        if !token.is_empty() {
            return Ok(ResolvedCredential::BearerToken(token));
        }
    }

    // Try OAuth resolver.
    if let Some(resolver) = oauth_resolver {
        let token = resolver.resolve().await?;
        return Ok(ResolvedCredential::OAuth(token));
    }

    Err(LlmError::MissingCredentials(
        "No credentials found: set ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN, \
         or configure an OAuth resolver"
            .to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_credential_header_format() {
        let api_key = ResolvedCredential::ApiKey("sk-test".to_string());
        let (name, value) = api_key.header_pair();
        assert_eq!(name, "x-api-key");
        assert_eq!(value, "sk-test");
    }

    #[test]
    fn bearer_header_format() {
        let bearer = ResolvedCredential::BearerToken("tok-abc".to_string());
        let (name, value) = bearer.header_pair();
        assert_eq!(name, "Authorization");
        assert_eq!(value, "Bearer tok-abc");
    }

    #[test]
    fn oauth_header_format() {
        let oauth = ResolvedCredential::OAuth(OAuthToken {
            access_token: "at-123".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: None,
        });
        let (name, value) = oauth.header_pair();
        assert_eq!(name, "Authorization");
        assert_eq!(value, "Bearer at-123");
    }

    #[test]
    fn resolve_credentials_fails_when_not_set() {
        // Use a mutex to serialize env var manipulation.
        use std::sync::Mutex;
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        let _guard = ENV_LOCK.lock().unwrap();

        // Save and clear env vars for this test.
        let saved_anthropic = std::env::var("ANTHROPIC_API_KEY").ok();
        let saved_openai = std::env::var("OPENAI_API_KEY").ok();
        let saved_auth_token = std::env::var("ANTHROPIC_AUTH_TOKEN").ok();
        std::env::remove_var("ANTHROPIC_API_KEY");
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("ANTHROPIC_AUTH_TOKEN");

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(resolve_credentials(None));
        assert!(result.is_err(), "expected error when no credentials set");

        // Restore env vars.
        if let Some(v) = saved_anthropic {
            std::env::set_var("ANTHROPIC_API_KEY", v);
        }
        if let Some(v) = saved_openai {
            std::env::set_var("OPENAI_API_KEY", v);
        }
        if let Some(v) = saved_auth_token {
            std::env::set_var("ANTHROPIC_AUTH_TOKEN", v);
        }
    }

    #[test]
    fn resolve_credentials_fallback_to_anthropic_auth_token() {
        use std::sync::Mutex;
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        let _guard = ENV_LOCK.lock().unwrap();

        // Clear all credential env vars first.
        let saved_anthropic = std::env::var("ANTHROPIC_API_KEY").ok();
        let saved_openai = std::env::var("OPENAI_API_KEY").ok();
        std::env::remove_var("ANTHROPIC_API_KEY");
        std::env::remove_var("OPENAI_API_KEY");

        // Temporarily set ANTHROPIC_AUTH_TOKEN.
        std::env::set_var("ANTHROPIC_AUTH_TOKEN", "tok-manual");
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(resolve_credentials(None));
        std::env::remove_var("ANTHROPIC_AUTH_TOKEN");
        if let Some(v) = saved_anthropic {
            std::env::set_var("ANTHROPIC_API_KEY", v);
        }
        if let Some(v) = saved_openai {
            std::env::set_var("OPENAI_API_KEY", v);
        }

        assert!(result.is_ok());
        let cred = result.unwrap();
        assert_eq!(cred.to_header(), "Bearer tok-manual");
    }
}
