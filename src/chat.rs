//! Chat 模块 —— 提供 `client.chat.completions()...` API
//!
//! 对应 OpenAI /v1/chat/completions 端点。

use crate::error::VllmError;
use crate::request;
use crate::types::{ChatCompletionResponse, MessageStream};
use reqwest::Client;
use serde_json::Value;

/// Chat API 入口
///
/// 访问方法: `client.chat.completions()`
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

    /// 进入 Chat Completions 构建器
    pub fn completions(&self) -> ChatCompletions {
        ChatCompletions::new(
            self.http.clone(),
            self.base_url.clone(),
            self.api_key.clone(),
        )
    }
}

/// Chat Completions 请求构建器
///
/// 使用 Builder 模式设置参数，最后调用 `.send()` 或 `.send_stream()` 执行请求。
pub struct ChatCompletions {
    http: Client,
    base_url: String,
    api_key: Option<String>,
}

impl ChatCompletions {
    pub(crate) fn new(http: Client, base_url: String, api_key: Option<String>) -> Self {
        Self {
            http,
            base_url,
            api_key,
        }
    }

    /// 创建一个新请求
    pub fn create(&self) -> ChatCompletionRequest {
        ChatCompletionRequest {
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

/// 单个 Chat Completion 请求
///
/// 支持的所有参数，通过链式调用设置。
pub struct ChatCompletionRequest {
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

impl Clone for ChatCompletionRequest {
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

impl ChatCompletionRequest {
    /// 设置模型名称
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// 设置消息列表（JSON 数组）
    ///
    /// 示例:
    /// ```ignore
    /// .messages(json!([
    ///     {"role": "system", "content": "You are helpful."},
    ///     {"role": "user", "content": "Hello!"}
    /// ]))
    /// ```
    pub fn messages(mut self, messages: Value) -> Self {
        self.messages = Some(messages);
        self
    }

    /// 设置 temperature (0.0 ~ 2.0)
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// 设置 max_tokens
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// 设置 top_p (0.0 ~ 1.0)
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

    /// 设置工具定义（JSON 数组）
    pub fn tools(mut self, tools: Value) -> Self {
        self.tools = Some(tools);
        self
    }

    /// 设置 tool_choice
    ///
    /// "auto" | "none" | "required" | {"type":"function","function":{"name":"..."}}
    pub fn tool_choice(mut self, tool_choice: Value) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// 设置额外参数（vLLM 扩展参数，如 `chat_template_kwargs`、`reasoning_effort`）
    ///
    /// 会展开合并到请求体中。
    pub fn extra(mut self, extra: Value) -> Self {
        self.extra = Some(extra);
        self
    }

    // -------- 内部方法 --------

    /// 构造 JSON 请求体
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

        macro_rules! insert_if_some {
            ($map:expr, $key:expr, $val:expr) => {
                if let Some(v) = $val {
                    $map.insert($key.into(), serde_json::json!(v));
                }
            };
        }
        insert_if_some!(obj, "temperature", self.temperature);
        insert_if_some!(obj, "max_tokens", self.max_tokens);
        insert_if_some!(obj, "top_p", self.top_p);
        insert_if_some!(obj, "top_k", self.top_k);
        if let Some(stop) = &self.stop {
            obj.insert("stop".into(), stop.clone());
        }
        if let Some(tools) = &self.tools {
            obj.insert("tools".into(), tools.clone());
        }
        if let Some(tc) = &self.tool_choice {
            obj.insert("tool_choice".into(), tc.clone());
        }
        // extra 参数展开合并
        if let Some(extra) = &self.extra {
            if let Some(extra_obj) = extra.as_object() {
                for (k, v) in extra_obj {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }

        Ok(body)
    }

    /// 执行请求，返回非流式响应
    pub async fn send(self) -> Result<ChatCompletionResponse, VllmError> {
        let body = self.build_body()?;
        let url = format!("{}/chat/completions", self.base_url);
        let raw = request::send_and_parse(request::with_auth(
            self.http.post(&url).json(&body),
            &self.api_key,
        ))
        .await?;
        ChatCompletionResponse::from_raw(raw)
    }

    /// 执行请求，返回流式响应
    pub async fn send_stream(mut self) -> Result<MessageStream, VllmError> {
        self.stream = true;

        // 提取 messages 文本用于本地 prompt token 估算
        let messages_text = self.messages.as_ref().map(extract_messages_text);

        let body = self.build_body()?;
        let url = format!("{}/chat/completions", self.base_url);
        let response = request::send_for_stream(request::with_auth(
            self.http.post(&url).json(&body),
            &self.api_key,
        ))
        .await?;

        Ok(MessageStream::with_context(response, messages_text))
    }
}

/// 从 messages JSON 数组中提取所有文本内容（用于本地 token 估算）
fn extract_messages_text(messages: &serde_json::Value) -> String {
    let mut text = String::new();
    if let Some(arr) = messages.as_array() {
        for msg in arr {
            if let Some(content) = msg.get("content") {
                match content {
                    serde_json::Value::String(s) => text.push_str(s),
                    serde_json::Value::Array(parts) => {
                        for part in parts {
                            if let Some(t) = part.get("text").and_then(|v| v.as_str()) {
                                text.push_str(t);
                            }
                        }
                    }
                    _ => {}
                }
            }
            text.push(' ');
        }
    }
    text
}
