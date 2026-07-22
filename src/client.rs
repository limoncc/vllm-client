//! VLLM Client 实现
//!
//! 提供 `VllmClient` 主结构体，支持 `new()`、`with_api_key()`、`timeout_secs()` 和 `builder()` 模式。

use crate::chat::Chat;
use crate::completions::Completions;
use reqwest::Client;
use std::time::Duration;

/// VLLM OpenAI 兼容客户端
///
/// # 基本用法
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

    /// Chat Completions API（`client.chat.completions().create().model(...).send()`）
    pub chat: Chat,

    /// Legacy Completions API（`client.completions.create().model(...).send()`）
    pub completions: Completions,
}

impl VllmClient {
    /// 创建客户端
    ///
    /// `base_url` 是 vLLM 服务的地址（如 `"http://localhost:8000/v1"`）。
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        let http = Client::new();

        Self {
            chat: Chat::new(http.clone(), base_url.clone(), None),
            completions: Completions::new(http.clone(), base_url.clone(), None),
            http,
            base_url,
            api_key: None,
        }
    }

    /// 设置 API Key（Bearer Token）
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self.chat = Chat::new(self.http.clone(), self.base_url.clone(), self.api_key.clone());
        self.completions = Completions::new(
            self.http.clone(),
            self.base_url.clone(),
            self.api_key.clone(),
        );
        self
    }

    /// 设置请求超时（秒）
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(secs))
            .build()
            .unwrap_or_else(|_| Client::new());

        self.http = http;
        self.chat = Chat::new(self.http.clone(), self.base_url.clone(), self.api_key.clone());
        self.completions = Completions::new(
            self.http.clone(),
            self.base_url.clone(),
            self.api_key.clone(),
        );
        self
    }

    /// 获取 base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// 获取 API Key
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    /// 使用构建器模式创建客户端
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

// ============================================================================
// Builder
// ============================================================================

/// VllmClient 构建器
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
    /// 设置 base URL
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// 设置 API Key
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// 设置超时时间（秒）
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    /// 构建 VllmClient
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

        VllmClient {
            chat: Chat::new(http.clone(), base_url.clone(), self.api_key.clone()),
            completions: Completions::new(http.clone(), base_url.clone(), self.api_key.clone()),
            http,
            base_url,
            api_key: self.api_key,
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
