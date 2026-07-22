//! 旧版 Completions 模块 —— `/v1/completions`
//!
//! OpenAI 旧版补全接口，vLLM 也兼容支持。

use crate::error::VllmError;
use crate::request;
use crate::types::{CompletionResponse, CompletionStream};
use reqwest::Client;
use serde_json::Value;

/// Legacy Completions 入口
///
/// 访问方法: `client.completions.create()`
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

    /// 创建一个 Legacy Completion 请求
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

/// Legacy Completion 请求构建器
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

impl Clone for CompletionRequest {
    fn clone(&self) -> Self {
        Self {
            http: self.http.clone(),
            base_url: self.base_url.clone(),
            api_key: self.api_key.clone(),
            model: self.model.clone(),
            prompt: self.prompt.clone(),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            top_p: self.top_p,
            top_k: self.top_k,
            stop: self.stop.clone(),
            stream: self.stream,
        }
    }
}

impl CompletionRequest {
    /// 设置模型名称
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// 设置 prompt（字符串或数组）
    pub fn prompt(mut self, prompt: impl Into<Value>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// 设置 max_tokens
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// 设置 temperature
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// 设置 top_p
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// 设置 top_k（vLLM 扩展参数）
    pub fn top_k(mut self, top_k: i32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// 设置 stop 序列
    pub fn stop(mut self, stop: Value) -> Self {
        self.stop = Some(stop);
        self
    }

    /// 启用流式模式
    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    // -------- 内部方法 --------

    /// 构造 JSON 请求体
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

        macro_rules! insert_if_some {
            ($map:expr, $key:expr, $val:expr) => {
                if let Some(v) = $val {
                    $map.insert($key.into(), serde_json::json!(v));
                }
            };
        }
        insert_if_some!(obj, "max_tokens", self.max_tokens);
        insert_if_some!(obj, "temperature", self.temperature);
        insert_if_some!(obj, "top_p", self.top_p);
        insert_if_some!(obj, "top_k", self.top_k);
        if let Some(stop) = &self.stop {
            obj.insert("stop".into(), stop.clone());
        }

        Ok(body)
    }

    /// 执行请求，返回非流式响应
    pub async fn send(self) -> Result<CompletionResponse, VllmError> {
        let body = self.build_body()?;
        let url = format!("{}/completions", self.base_url);
        let raw = request::send_and_parse(request::with_auth(
            self.http.post(&url).json(&body),
            &self.api_key,
        ))
        .await?;
        CompletionResponse::from_raw(raw)
    }

    /// 执行请求，返回流式响应
    pub async fn send_stream(mut self) -> Result<CompletionStream, VllmError> {
        self.stream = true;

        let body = self.build_body()?;
        let url = format!("{}/completions", self.base_url);
        let response = request::send_for_stream(request::with_auth(
            self.http.post(&url).json(&body),
            &self.api_key,
        ))
        .await?;

        Ok(CompletionStream::new(response))
    }
}
