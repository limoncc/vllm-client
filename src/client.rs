//! VLLM Client implementation

use crate::chat::Chat;
use crate::completions::Completions;
use reqwest::Client;
use std::time::Duration;

/// VLLM OpenAI-compatible client
///
/// # Example
///
/// ```rust
/// use vllm_client::VllmClient;
///
/// let client = VllmClient::new("http://localhost:8000/v1");
/// let client = VllmClient::new("http://localhost:8000/v1").with_api_key("sk-xxx");
/// ```
pub struct VllmClient {
    http: Client,
    base_url: String,
    api_key: Option<String>,
    /// Chat completions API
    pub chat: Chat,
    /// Legacy completions API
    pub completions: Completions,
}

impl VllmClient {
    /// Create a new client with base URL
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the vLLM server (e.g., "http://localhost:8000/v1")
    ///
    /// # Example
    ///
    /// ```rust
    /// use vllm_client::VllmClient;
    ///
    /// let client = VllmClient::new("http://localhost:8000/v1");
    /// ```
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        let base_url = base_url.trim_end_matches('/').to_string();

        let http = Client::new();
        let chat = Chat::new(http.clone(), base_url.clone(), None);
        let completions = Completions::new(http.clone(), base_url.clone(), None);

        Self {
            http,
            base_url,
            api_key: None,
            chat,
            completions,
        }
    }

    /// Set API key (builder pattern)
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key for authentication
    ///
    /// # Example
    ///
    /// ```rust
    /// use vllm_client::VllmClient;
    ///
    /// let client = VllmClient::new("http://localhost:8000/v1").with_api_key("sk-xxx");
    /// ```
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self.chat = Chat::new(
            self.http.clone(),
            self.base_url.clone(),
            self.api_key.clone(),
        );
        self.completions = Completions::new(
            self.http.clone(),
            self.base_url.clone(),
            self.api_key.clone(),
        );
        self
    }

    /// Set request timeout in seconds
    ///
    /// # Arguments
    ///
    /// * `secs` - Timeout in seconds
    ///
    /// # Example
    ///
    /// ```rust
    /// use vllm_client::VllmClient;
    ///
    /// let client = VllmClient::new("http://localhost:8000/v1").timeout_secs(60);
    /// ```
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(secs))
            .build()
            .unwrap_or_else(|_| Client::new());

        self.chat = Chat::new(http.clone(), self.base_url.clone(), self.api_key.clone());
        self.completions =
            Completions::new(http.clone(), self.base_url.clone(), self.api_key.clone());
        self.http = http;
        self
    }

    /// Get base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get API key
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    /// Create a builder for more configuration options
    ///
    /// # Example
    ///
    /// ```rust
    /// use vllm_client::VllmClient;
    ///
    /// let client = VllmClient::builder()
    ///     .base_url("http://localhost:8000/v1")
    ///     .api_key("sk-xxx")
    ///     .timeout_secs(120)
    ///     .build();
    /// ```
    pub fn builder() -> VllmClientBuilder {
        VllmClientBuilder::default()
    }
}

impl Default for VllmClient {
    fn default() -> Self {
        Self::new("http://localhost:8000/v1")
    }
}

/// Builder for VllmClient
///
/// # Example
///
/// ```rust
/// use vllm_client::VllmClient;
///
/// let client = VllmClient::builder()
///     .base_url("http://localhost:8000/v1")
///     .api_key("sk-xxx")
///     .timeout_secs(120)
///     .build();
/// ```
pub struct VllmClientBuilder {
    base_url: String,
    api_key: Option<String>,
    timeout_secs: Option<u64>,
}

impl Default for VllmClientBuilder {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8000/v1".to_string(),
            api_key: None,
            timeout_secs: None,
        }
    }
}

impl VllmClientBuilder {
    /// Set base URL
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set API key
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set timeout in seconds
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    /// Build the client
    pub fn build(self) -> VllmClient {
        let base_url = self.base_url.trim_end_matches('/').to_string();

        let http = if let Some(secs) = self.timeout_secs {
            Client::builder()
                .timeout(Duration::from_secs(secs))
                .build()
                .unwrap_or_else(|_| Client::new())
        } else {
            Client::new()
        };

        let chat = Chat::new(http.clone(), base_url.clone(), self.api_key.clone());
        let completions = Completions::new(http.clone(), base_url.clone(), self.api_key.clone());

        VllmClient {
            http,
            base_url,
            api_key: self.api_key,
            chat,
            completions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client() {
        let client = VllmClient::new("http://localhost:8000/v1");
        assert_eq!(client.base_url(), "http://localhost:8000/v1");
    }

    #[test]
    fn test_new_client_trailing_slash() {
        let client = VllmClient::new("http://localhost:8000/v1/");
        assert_eq!(client.base_url(), "http://localhost:8000/v1");
    }

    #[test]
    fn test_api_key() {
        let client = VllmClient::new("http://localhost:8000/v1").with_api_key("sk-test");
        assert_eq!(client.api_key(), Some("sk-test"));
    }

    #[test]
    fn test_builder() {
        let client = VllmClient::builder()
            .base_url("http://localhost:8000/v1")
            .api_key("sk-test")
            .build();

        assert_eq!(client.base_url(), "http://localhost:8000/v1");
        assert_eq!(client.api_key(), Some("sk-test"));
    }

    #[test]
    fn test_default() {
        let client = VllmClient::default();
        assert!(client.base_url().contains("localhost"));
    }
}
