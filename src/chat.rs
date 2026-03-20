//! Chat module - provides `client.chat.completions().create()` API

use crate::error::VllmError;
use crate::types::{ChatCompletionResponse, MessageStream};
use reqwest::Client;
use serde_json::Value;

/// Chat API module
///
/// Provides access to chat completions API.
pub struct Chat {
    http: Client,
    base_url: String,
    api_key: Option<String>,
}

impl Chat {
    pub(crate) fn new(http: Client, base_url: String, api_key: Option<String>) -> Self {
        Self {
            http,
            base_url,
            api_key,
        }
    }

    /// Access completions API
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use vllm_client::{VllmClient, json};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = VllmClient::new("http://localhost:8000/v1");
    /// let response = client.chat.completions().create()
    ///     .model("Qwen/Qwen2.5-72B-Instruct")
    ///     .messages(json!([{"role": "user", "content": "Hello!"}]))
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn completions(&self) -> Completions {
        Completions::new(
            self.http.clone(),
            self.base_url.clone(),
            self.api_key.clone(),
        )
    }
}

/// Completions API (chat.completions)
pub struct Completions {
    http: Client,
    base_url: String,
    api_key: Option<String>,
}

impl Completions {
    pub(crate) fn new(http: Client, base_url: String, api_key: Option<String>) -> Self {
        Self {
            http,
            base_url,
            api_key,
        }
    }

    /// Create a new chat completion request
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use vllm_client::{VllmClient, json};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = VllmClient::new("http://localhost:8000/v1");
    /// let response = client.chat.completions().create()
    ///     .model("Qwen/Qwen2.5-72B-Instruct")
    ///     .messages(json!([
    ///         {"role": "user", "content": "Hello!"}
    ///     ]))
    ///     .temperature(0.7)
    ///     .max_tokens(1024)
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create(&self) -> ChatCompletionsRequest {
        ChatCompletionsRequest {
            http: self.http.clone(),
            base_url: self.base_url.clone(),
            api_key: self.api_key.clone(),
            model: None,
            messages: None,
            temperature: None,
            max_tokens: None,
            top_p: None,
            top_k: None,
            stop: None,
            stream: false,
            tools: None,
            tool_choice: None,
            extra: None,
        }
    }
}

/// Chat completion request builder
///
/// Build a request using the builder pattern, then call `.send()` or `.send_stream()`.
pub struct ChatCompletionsRequest {
    http: Client,
    base_url: String,
    api_key: Option<String>,
    model: Option<String>,
    messages: Option<Value>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    top_p: Option<f32>,
    top_k: Option<i32>,
    stop: Option<Value>,
    stream: bool,
    tools: Option<Value>,
    tool_choice: Option<Value>,
    extra: Option<Value>,
}

impl ChatCompletionsRequest {
    /// Set the model name
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set messages (JSON format)
    ///
    /// # Example
    ///
    /// ```ignore
    /// .messages(json!([
    ///     {"role": "system", "content": "You are a helpful assistant."},
    ///     {"role": "user", "content": "Hello!"}
    /// ]))
    /// ```
    pub fn messages(mut self, messages: Value) -> Self {
        self.messages = Some(messages);
        self
    }

    /// Set temperature (0.0 - 2.0)
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set max tokens
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set top_p (0.0 - 1.0)
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
    ///
    /// # Example
    ///
    /// ```ignore
    /// .stop(json!(["END", "STOP"]))
    /// // or single string
    /// .stop(json!("END"))
    /// ```
    pub fn stop(mut self, stop: Value) -> Self {
        self.stop = Some(stop);
        self
    }

    /// Enable streaming mode
    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// Set tools (JSON format)
    ///
    /// # Example
    ///
    /// ```ignore
    /// .tools(json!([
    ///     {
    ///         "type": "function",
    ///         "function": {
    ///             "name": "get_weather",
    ///             "description": "Get weather info",
    ///             "parameters": {
    ///                 "type": "object",
    ///                 "properties": {
    ///                     "city": {"type": "string"}
    ///                 },
    ///                 "required": ["city"]
    ///             }
    ///         }
    ///     }
    /// ]))
    /// ```
    pub fn tools(mut self, tools: Value) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set tool choice
    ///
    /// # Example
    ///
    /// ```ignore
    /// .tool_choice(json!("auto"))           // "auto" | "none" | "required"
    /// .tool_choice(json!({"type": "function", "function": {"name": "get_weather"}}))
    /// ```
    pub fn tool_choice(mut self, tool_choice: Value) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Set extra parameters (vLLM extensions)
    ///
    /// # Example
    ///
    /// ```ignore
    /// .extra(json!({
    ///     "chat_template_kwargs": {"think_mode": true},
    ///     "reasoning_effort": "high"
    /// }))
    /// ```
    pub fn extra(mut self, extra: Value) -> Self {
        self.extra = Some(extra);
        self
    }

    /// Build request body as JSON
    fn build_body(&self) -> Result<Value, VllmError> {
        let model = self
            .model
            .as_ref()
            .ok_or_else(|| VllmError::MissingParameter("model is required".into()))?;

        let messages = self
            .messages
            .as_ref()
            .ok_or_else(|| VllmError::MissingParameter("messages is required".into()))?;

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": self.stream,
        });

        let obj = body.as_object_mut().unwrap();

        if let Some(temperature) = self.temperature {
            obj.insert("temperature".into(), serde_json::json!(temperature));
        }
        if let Some(max_tokens) = self.max_tokens {
            obj.insert("max_tokens".into(), serde_json::json!(max_tokens));
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
        if let Some(tools) = &self.tools {
            obj.insert("tools".into(), tools.clone());
        }
        if let Some(tool_choice) = &self.tool_choice {
            obj.insert("tool_choice".into(), tool_choice.clone());
        }
        if let Some(extra) = &self.extra {
            if let Some(extra_obj) = extra.as_object() {
                for (key, value) in extra_obj {
                    obj.insert(key.clone(), value.clone());
                }
            }
        }

        Ok(body)
    }

    /// Send request and get response
    pub async fn send(self) -> Result<ChatCompletionResponse, VllmError> {
        let body = self.build_body()?;
        let url = format!("{}/chat/completions", self.base_url);

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

        ChatCompletionResponse::from_raw(raw)
    }

    /// Send request and get streaming response
    pub async fn send_stream(self) -> Result<MessageStream, VllmError> {
        let mut request = self.clone();
        request.stream = true;

        let body = request.build_body()?;
        let url = format!("{}/chat/completions", request.base_url);

        let mut req = request.http.post(&url).json(&body);

        if let Some(api_key) = &request.api_key {
            req = req.bearer_auth(api_key);
        }

        let response = req.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(VllmError::api(status.as_u16(), error_text));
        }

        Ok(MessageStream::new(response))
    }
}

impl Clone for ChatCompletionsRequest {
    fn clone(&self) -> Self {
        Self {
            http: self.http.clone(),
            base_url: self.base_url.clone(),
            api_key: self.api_key.clone(),
            model: self.model.clone(),
            messages: self.messages.clone(),
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            top_p: self.top_p,
            top_k: self.top_k,
            stop: self.stop.clone(),
            stream: self.stream,
            tools: self.tools.clone(),
            tool_choice: self.tool_choice.clone(),
            extra: self.extra.clone(),
        }
    }
}
