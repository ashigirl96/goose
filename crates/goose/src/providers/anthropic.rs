use anyhow::Result;
use async_trait::async_trait;
use axum::http::HeaderMap;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

use super::base::{ConfigKey, Provider, ProviderMetadata, ProviderUsage};
use super::errors::ProviderError;
use super::formats::anthropic::{create_request, get_usage, response_to_message};
use super::utils::{emit_debug_trace, get_model};
use crate::message::Message;
use crate::model::ModelConfig;
use mcp_core::tool::Tool;

pub const ANTHROPIC_DEFAULT_MODEL: &str = "claude-3-5-sonnet-latest";
pub const ANTHROPIC_KNOWN_MODELS: &[&str] = &[
    "claude-3-5-sonnet-latest",
    "claude-3-5-haiku-latest",
    "claude-3-opus-latest",
    "claude-3-7-sonnet-20250219",
    "claude-3-7-sonnet-latest",
];

pub const ANTHROPIC_DOC_URL: &str = "https://docs.anthropic.com/en/docs/about-claude/models";

/// Default timeout for API requests in seconds
const DEFAULT_TIMEOUT_SECS: u64 = 600;
/// Default initial interval for retry (in milliseconds)
const DEFAULT_INITIAL_RETRY_INTERVAL_MS: u64 = 5000;
/// Default maximum number of retries
const DEFAULT_MAX_RETRIES: usize = 6;
/// Default retry backoff multiplier
const DEFAULT_BACKOFF_MULTIPLIER: f64 = 2.0;
/// Default maximum interval for retry (in milliseconds)
const DEFAULT_MAX_RETRY_INTERVAL_MS: u64 = 320_000;

/// Retry configuration for handling rate limit errors
#[derive(Debug, Clone)]
struct RetryConfig {
    /// Maximum number of retry attempts
    max_retries: usize,
    /// Initial interval between retries in milliseconds
    initial_interval_ms: u64,
    /// Multiplier for backoff (exponential)
    backoff_multiplier: f64,
    /// Maximum interval between retries in milliseconds
    max_interval_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            initial_interval_ms: DEFAULT_INITIAL_RETRY_INTERVAL_MS,
            backoff_multiplier: DEFAULT_BACKOFF_MULTIPLIER,
            max_interval_ms: DEFAULT_MAX_RETRY_INTERVAL_MS,
        }
    }
}

impl RetryConfig {
    /// Calculate the delay for a specific retry attempt (with jitter)
    fn delay_for_attempt(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(0);
        }

        // Calculate exponential backoff
        let exponent = (attempt - 1) as u32;
        let base_delay_ms = (self.initial_interval_ms as f64
            * self.backoff_multiplier.powi(exponent as i32)) as u64;

        // Apply max limit
        let capped_delay_ms = std::cmp::min(base_delay_ms, self.max_interval_ms);

        // Add jitter (+/-20% randomness) to avoid thundering herd problem
        let jitter_factor = 0.8 + (rand::random::<f64>() * 0.4); // Between 0.8 and 1.2
        let jittered_delay_ms = (capped_delay_ms as f64 * jitter_factor) as u64;

        Duration::from_millis(jittered_delay_ms)
    }
}

#[derive(serde::Serialize)]
pub struct AnthropicProvider {
    #[serde(skip)]
    client: Client,
    host: String,
    api_key: String,
    model: ModelConfig,
    #[serde(skip)]
    retry_config: RetryConfig,
}

impl Default for AnthropicProvider {
    fn default() -> Self {
        let model = ModelConfig::new(AnthropicProvider::metadata().default_model);
        AnthropicProvider::from_env(model).expect("Failed to initialize Anthropic provider")
    }
}

impl AnthropicProvider {
    pub fn from_env(model: ModelConfig) -> Result<Self> {
        let config = crate::config::Config::global();
        let api_key: String = config.get_secret("ANTHROPIC_API_KEY")?;
        let host: String = config
            .get_param("ANTHROPIC_HOST")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()?;

        // Load retry configuration from environment
        let retry_config = Self::load_retry_config(&config);

        Ok(Self {
            client,
            host,
            api_key,
            model,
            retry_config,
        })
    }

    /// Loads retry configuration from environment variables or uses defaults.
    fn load_retry_config(config: &crate::config::Config) -> RetryConfig {
        let max_retries = config
            .get_param("ANTHROPIC_MAX_RETRIES")
            .ok()
            .and_then(|v: String| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_RETRIES);

        let initial_interval_ms = config
            .get_param("ANTHROPIC_INITIAL_RETRY_INTERVAL_MS")
            .ok()
            .and_then(|v: String| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_INITIAL_RETRY_INTERVAL_MS);

        let backoff_multiplier = config
            .get_param("ANTHROPIC_BACKOFF_MULTIPLIER")
            .ok()
            .and_then(|v: String| v.parse::<f64>().ok())
            .unwrap_or(DEFAULT_BACKOFF_MULTIPLIER);

        let max_interval_ms = config
            .get_param("ANTHROPIC_MAX_RETRY_INTERVAL_MS")
            .ok()
            .and_then(|v: String| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_MAX_RETRY_INTERVAL_MS);

        RetryConfig {
            max_retries,
            initial_interval_ms,
            backoff_multiplier,
            max_interval_ms,
        }
    }

    async fn post(&self, headers: HeaderMap, payload: Value) -> Result<Value, ProviderError> {
        let base_url = url::Url::parse(&self.host)
            .map_err(|e| ProviderError::RequestFailed(format!("Invalid base URL: {e}")))?;
        let url = base_url.join("v1/messages").map_err(|e| {
            ProviderError::RequestFailed(format!("Failed to construct endpoint URL: {e}"))
        })?;

        // Initialize retry counter
        let mut attempts = 0;
        let mut last_error = None;

        loop {
            // Check if we've exceeded max retries
            if attempts > 0 && attempts > self.retry_config.max_retries {
                let error_msg = format!(
                    "Exceeded maximum retry attempts ({}) for rate limiting (429)",
                    self.retry_config.max_retries
                );
                tracing::error!("{}", error_msg);
                return Err(last_error.unwrap_or(ProviderError::RateLimitExceeded(error_msg)));
            }

            // Make the request
            let response = self
                .client
                .post(url.clone())
                .headers(headers.clone())
                .json(&payload)
                .send()
                .await
                .map_err(|e| ProviderError::RequestFailed(e.to_string()))?;

            let status = response.status();
            let response_payload: Option<Value> = response.json().await.ok();

            // Handle 429 Too Many Requests or 503 Service Unavailable
            if status == StatusCode::TOO_MANY_REQUESTS || status == StatusCode::SERVICE_UNAVAILABLE {
                // Handle rate limit or server error with retry logic
                attempts += 1;

                // Try to parse response for more detailed error info
                let error_message = if status == StatusCode::TOO_MANY_REQUESTS {
                    format!("Rate limit exceeded (429). Retrying request.")
                } else {
                    format!("Server error (503). Retrying request.")
                };

                tracing::warn!(
                    "Rate limit or server error (status {}) (attempt {}/{}): {}. Retrying after backoff...",
                    status,
                    attempts,
                    self.retry_config.max_retries,
                    error_message
                );

                // Store the error in case we need to return it after max retries
                last_error = Some(ProviderError::RateLimitExceeded(error_message));

                // Calculate and apply the backoff delay
                let delay = self.retry_config.delay_for_attempt(attempts);
                tracing::info!("Backing off for {:?} before retry", delay);
                sleep(delay).await;
                continue;
            }

            // https://docs.anthropic.com/en/api/errors
            return match status {
                StatusCode::OK => response_payload.ok_or_else( || ProviderError::RequestFailed("Response body is not valid JSON".to_string()) ),
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                    Err(ProviderError::Authentication(format!("Authentication failed. Please ensure your API keys are valid and have the required permissions. \
                        Status: {}. Response: {:?}", status, response_payload)))
                }
                StatusCode::BAD_REQUEST => {
                    let mut error_msg = "Unknown error".to_string();
                    if let Some(payload) = &response_payload {
                        if let Some(error) = payload.get("error") {
                        tracing::debug!("Bad Request Error: {error:?}");
                        error_msg = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error").to_string();
                        if error_msg.to_lowercase().contains("too long") || error_msg.to_lowercase().contains("too many") {
                            return Err(ProviderError::ContextLengthExceeded(error_msg.to_string()));
                        }
                    }}
                    tracing::debug!(
                        "{}", format!("Provider request failed with status: {}. Payload: {:?}", status, response_payload)
                    );
                    Err(ProviderError::RequestFailed(format!("Request failed with status: {}. Message: {}", status, error_msg)))
                }
                StatusCode::INTERNAL_SERVER_ERROR => {
                    Err(ProviderError::ServerError(format!("{:?}", response_payload)))
                }
                _ => {
                    tracing::debug!(
                        "{}", format!("Provider request failed with status: {}. Payload: {:?}", status, response_payload)
                    );
                    Err(ProviderError::RequestFailed(format!("Request failed with status: {}", status)))
                }
            };
        }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn metadata() -> ProviderMetadata {
        ProviderMetadata::new(
            "anthropic",
            "Anthropic",
            "Claude and other models from Anthropic",
            ANTHROPIC_DEFAULT_MODEL,
            ANTHROPIC_KNOWN_MODELS.to_vec(),
            ANTHROPIC_DOC_URL,
            vec![
                ConfigKey::new("ANTHROPIC_API_KEY", true, true, None),
                ConfigKey::new(
                    "ANTHROPIC_HOST",
                    true,
                    false,
                    Some("https://api.anthropic.com"),
                ),
                ConfigKey::new(
                    "ANTHROPIC_MAX_RETRIES",
                    false,
                    false,
                    Some(&DEFAULT_MAX_RETRIES.to_string()),
                ),
                ConfigKey::new(
                    "ANTHROPIC_INITIAL_RETRY_INTERVAL_MS",
                    false,
                    false,
                    Some(&DEFAULT_INITIAL_RETRY_INTERVAL_MS.to_string()),
                ),
                ConfigKey::new(
                    "ANTHROPIC_BACKOFF_MULTIPLIER",
                    false,
                    false,
                    Some(&DEFAULT_BACKOFF_MULTIPLIER.to_string()),
                ),
                ConfigKey::new(
                    "ANTHROPIC_MAX_RETRY_INTERVAL_MS",
                    false,
                    false,
                    Some(&DEFAULT_MAX_RETRY_INTERVAL_MS.to_string()),
                ),
            ],
        )
    }

    fn get_model_config(&self) -> ModelConfig {
        self.model.clone()
    }

    #[tracing::instrument(
        skip(self, system, messages, tools),
        fields(model_config, input, output, input_tokens, output_tokens, total_tokens)
    )]
    async fn complete(
        &self,
        system: &str,
        messages: &[Message],
        tools: &[Tool],
    ) -> Result<(Message, ProviderUsage), ProviderError> {
        let payload = create_request(&self.model, system, messages, tools)?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-api-key", self.api_key.parse().unwrap());
        headers.insert("anthropic-version", "2023-06-01".parse().unwrap());

        let is_thinking_enabled = std::env::var("CLAUDE_THINKING_ENABLED").is_ok();
        if self.model.model_name.starts_with("claude-3-7-sonnet-") && is_thinking_enabled {
            // https://docs.anthropic.com/en/docs/build-with-claude/extended-thinking#extended-output-capabilities-beta
            headers.insert("anthropic-beta", "output-128k-2025-02-19".parse().unwrap());
        }

        if self.model.model_name.starts_with("claude-3-7-sonnet-") {
            // https://docs.anthropic.com/en/docs/build-with-claude/tool-use/token-efficient-tool-use
            headers.insert(
                "anthropic-beta",
                "token-efficient-tools-2025-02-19".parse().unwrap(),
            );
        }

        // Make request
        let response = self.post(headers, payload.clone()).await?;

        // Parse response
        let message = response_to_message(response.clone())?;
        let usage = get_usage(&response)?;

        let model = get_model(&response);
        emit_debug_trace(&self.model, &payload, &response, &usage);
        Ok((message, ProviderUsage::new(model, usage)))
    }
}
