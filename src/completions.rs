//! Legacy Completions API (/v1/completions)
//!
//! OpenAI 的旧版 API，vLLM 也支持

use crate::error::VllmError;
use crate::types::CompletionResponse;
use reqwest::Client;
use serde_json::Value;

/// Completions 模块入口
pub struct Completions {
    pub(crate) http: Client,
    pub(crate) base_url: String,
    pub(crate) api_key: Option<String>,
}

impl Completions {
    pub(crate) fn new(http: Client, base_url: String, api_key: Option<String>) -> Self {
        Self {
            http,
            base_url,
            api_key,
        }
    }

    /// Create a legacy completion request
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use vllm_client::VllmClient;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = VllmClient::new("http://localhost:8000/v1");
    /// let response = client.completions.create()
    ///     .model("model-name")
    ///     .prompt("Hello")
    ///     .max_tokens(100)
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create(&self) -> CompletionRequest {
        CompletionRequest {
            http: self.http.clone(),
            base_url: self.base_url.clone(),
            api_key: self.api_key.clone(),
            model: None,
            prompt: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop: None,
            stream: false,
        }
    }
}

/// Completion request builder
pub struct CompletionRequest {
    http: Client,
    base_url: String,
    api_key: Option<String>,
    model: Option<String>,
    prompt: Option<Value>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    top_k: Option<i32>,
    stop: Option<Value>,
    stream: bool,
}

impl CompletionRequest {
    /// Set the model name
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set prompt (text or array)
    pub fn prompt(mut self, prompt: impl Into<Value>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// Set max tokens
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set temperature
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set top_p
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set top_k (vLLM extension)
    pub fn top_k(mut self, top_k: i32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Set stop sequences
    pub fn stop(mut self, stop: Value) -> Self {
        self.stop = Some(stop);
        self
    }

    /// Enable streaming mode
    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// Build request body
    fn build_body(&self) -> Result<Value, VllmError> {
        let model = self
            .model
            .as_ref()
            .ok_or_else(|| VllmError::MissingParameter("model is required".into()))?;

        let prompt = self
            .prompt
            .as_ref()
            .ok_or_else(|| VllmError::MissingParameter("prompt is required".into()))?;

        let mut body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": self.stream,
        });

        let obj = body.as_object_mut().unwrap();

        if let Some(max_tokens) = self.max_tokens {
            obj.insert("max_tokens".into(), serde_json::json!(max_tokens));
        }
        if let Some(temperature) = self.temperature {
            obj.insert("temperature".into(), serde_json::json!(temperature));
        }
        if let Some(top_p) = self.top_p {
            obj.insert("top_p".into(), serde_json::json!(top_p));
        }
        if let Some(top_k) = self.top_k {
            obj.insert("top_k".into(), serde_json::json!(top_k));
        }
        if let Some(stop) = &self.stop {
            obj.insert("stop".into(), stop.clone());
        }

        Ok(body)
    }

    /// Send request
    pub async fn send(self) -> Result<CompletionResponse, VllmError> {
        let body = self.build_body()?;
        let url = format!("{}/completions", self.base_url);

        let mut request = self.http.post(&url).json(&body);

        if let Some(api_key) = &self.api_key {
            request = request.bearer_auth(api_key);
        }

        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(VllmError::api(status.as_u16(), error_text));
        }

        let raw: Value = response.json().await?;

        CompletionResponse::from_raw(raw)
    }
}
