//! 公共类型定义

use serde::{Deserialize, Serialize};

// ============================================================================
// Chat Completion Response
// ============================================================================

/// Chat Completion 响应
#[derive(Debug, Clone)]
pub struct ChatCompletionResponse {
    /// 原始 JSON 响应（保留所有字段）
    pub raw: serde_json::Value,

    /// 响应 ID
    pub id: String,

    /// 对象类型
    pub object: String,

    /// 模型名称
    pub model: String,

    /// 创建时间戳
    pub created: u64,

    /// 助手回复内容
    pub content: Option<String>,

    /// 思考/推理内容 (vLLM 推理模型)
    pub reasoning_content: Option<String>,

    /// 工具调用列表
    pub tool_calls: Vec<ToolCall>,

    /// 结束原因
    pub finish_reason: Option<String>,

    /// Token 使用统计
    pub usage: Option<Usage>,
}

impl ChatCompletionResponse {
    /// 是否有工具调用
    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }

    /// 获取第一个工具调用
    pub fn first_tool_call(&self) -> Option<&ToolCall> {
        self.tool_calls.first()
    }

    /// 获取 assistant 消息（用于追加到对话历史）
    pub fn assistant_message(&self) -> serde_json::Value {
        if self.has_tool_calls() {
            serde_json::json!({
                "role": "assistant",
                "content": self.content,
                "tool_calls": self.raw["choices"][0]["message"]["tool_calls"]
            })
        } else {
            serde_json::json!({
                "role": "assistant",
                "content": self.content
            })
        }
    }
}

// ============================================================================
// Tool Call
// ============================================================================

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// 调用 ID
    pub id: String,

    /// 函数名称
    pub name: String,

    /// 函数参数 (JSON 字符串)
    pub arguments: String,
}

impl ToolCall {
    /// 解析参数为 serde_json::Value
    pub fn parse_args(&self) -> Result<serde_json::Value, crate::VllmError> {
        Ok(serde_json::from_str(&self.arguments)?)
    }

    /// 解析参数为指定类型
    pub fn parse_args_as<T: for<'de> Deserialize<'de>>(&self) -> Result<T, crate::VllmError> {
        Ok(serde_json::from_str(&self.arguments)?)
    }

    /// 构造工具结果消息
    pub fn result<T: Serialize>(&self, content: T) -> serde_json::Value {
        serde_json::json!({
            "role": "tool",
            "tool_call_id": self.id,
            "content": serde_json::to_string(&content).unwrap_or_default()
        })
    }
}

// ============================================================================
// Usage
// ============================================================================

/// Token 使用统计
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Usage {
    /// 输入 token 数
    pub prompt_tokens: u64,

    /// 输出 token 数
    pub completion_tokens: u64,

    /// 总 token 数
    pub total_tokens: u64,
}

// ============================================================================
// Legacy Completion Response
// ============================================================================

/// Legacy Completion 响应
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    /// 原始 JSON 响应
    pub raw: serde_json::Value,

    /// 响应 ID
    pub id: String,

    /// 对象类型
    pub object: String,

    /// 模型名称
    pub model: String,

    /// 选择列表
    pub choices: Vec<CompletionChoice>,

    /// Token 使用统计
    pub usage: Option<Usage>,
}

impl CompletionResponse {
    /// Parse from raw JSON
    pub fn from_raw(raw: serde_json::Value) -> Result<Self, crate::VllmError> {
        let id = raw["id"]
            .as_str()
            .ok_or_else(|| crate::VllmError::InvalidResponse("missing id".into()))?
            .to_string();

        let object = raw["object"]
            .as_str()
            .unwrap_or("text_completion")
            .to_string();

        let model = raw["model"]
            .as_str()
            .ok_or_else(|| crate::VllmError::InvalidResponse("missing model".into()))?
            .to_string();

        let choices = raw["choices"]
            .as_array()
            .ok_or_else(|| crate::VllmError::InvalidResponse("missing choices".into()))?
            .iter()
            .map(|c| CompletionChoice::from_raw(c.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        let usage = raw.get("usage").and_then(|u| {
            Some(Usage {
                prompt_tokens: u["prompt_tokens"].as_u64()?,
                completion_tokens: u["completion_tokens"].as_u64()?,
                total_tokens: u["total_tokens"].as_u64()?,
            })
        });

        Ok(Self {
            raw,
            id,
            object,
            model,
            choices,
            usage,
        })
    }
}

/// Legacy Completion 选择项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChoice {
    /// 索引
    pub index: u32,

    /// 文本内容
    pub text: String,

    /// Log probabilities (optional)
    pub logprobs: Option<serde_json::Value>,

    /// 结束原因
    pub finish_reason: Option<String>,
}

impl CompletionChoice {
    /// Parse from raw JSON
    pub fn from_raw(raw: serde_json::Value) -> Result<Self, crate::VllmError> {
        let index = raw["index"]
            .as_u64()
            .ok_or_else(|| crate::VllmError::InvalidResponse("missing choice index".into()))?
            as u32;

        let text = raw["text"]
            .as_str()
            .ok_or_else(|| crate::VllmError::InvalidResponse("missing choice text".into()))?
            .to_string();

        let logprobs = raw.get("logprobs").cloned();
        let finish_reason = raw["finish_reason"].as_str().map(|s| s.to_string());

        Ok(Self {
            index,
            text,
            logprobs,
            finish_reason,
        })
    }
}

// ============================================================================
// Stream Events
// ============================================================================

/// 流式响应事件
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// 文本内容增量
    Content(String),

    /// 思考/推理内容增量
    Reasoning(String),

    /// 工具调用增量
    ToolCallDelta {
        index: usize,
        id: Option<String>,
        name: Option<String>,
        arguments: Option<String>,
    },

    /// 工具调用完成
    ToolCallComplete(ToolCall),

    /// Token 使用统计（流结束时）
    Usage(Usage),

    /// 流结束
    Done,

    /// 错误
    Error(crate::VllmError),
}

/// 流式响应对象
pub struct MessageStream {
    inner: futures::stream::BoxStream<'static, StreamEvent>,
}

impl MessageStream {
    /// 从 Response 创建流
    pub fn new(response: reqwest::Response) -> Self {
        use async_stream::stream;
        use futures::StreamExt;

        let stream = stream! {
            let mut byte_stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut tool_calls_builders: std::collections::HashMap<usize, ToolCallBuilder> = std::collections::HashMap::new();

            while let Some(chunk) = byte_stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&text);

                        // 解析 SSE 格式: "data: {...}\n\n"
                        while let Some(pos) = buffer.find("\n\n") {
                            let line = buffer[..pos].to_string();
                            buffer = buffer[pos + 2..].to_string();

                            if let Some(data) = line.strip_prefix("data: ") {
                                if data == "[DONE]" {
                                    yield StreamEvent::Done;
                                    return;
                                }

                                match serde_json::from_str::<serde_json::Value>(data) {
                                    Ok(chunk) => {
                                        // 解析 chunk
                                        if let Some(choices) = chunk.get("choices").and_then(|c| c.as_array()) {
                                            for choice in choices {
                                                let delta = &choice["delta"];

                                                // 文本内容
                                                if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                                    if !content.is_empty() {
                                                        yield StreamEvent::Content(content.to_string());
                                                    }
                                                }

                                                // 推理内容 - 支持 "reasoning" 和 "reasoning_content" 两种字段名
                                                let reasoning = delta.get("reasoning").and_then(|c| c.as_str())
                                                    .or_else(|| delta.get("reasoning_content").and_then(|c| c.as_str()));

                                                if let Some(reasoning) = reasoning {
                                                    if !reasoning.is_empty() {
                                                        yield StreamEvent::Reasoning(reasoning.to_string());
                                                    }
                                                }

                                                // 工具调用增量
                                                if let Some(tool_calls) = delta.get("tool_calls").and_then(|c| c.as_array()) {
                                                    for tc in tool_calls {
                                                        let index = tc.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                                                        let builder = tool_calls_builders.entry(index).or_default();

                                                        if let Some(id) = tc.get("id").and_then(|i| i.as_str()) {
                                                            builder.id = Some(id.to_string());
                                                        }
                                                        if let Some(func) = tc.get("function") {
                                                            if let Some(name) = func.get("name").and_then(|n| n.as_str()) {
                                                                builder.name = Some(name.to_string());
                                                            }
                                                            if let Some(args) = func.get("arguments").and_then(|a| a.as_str()) {
                                                                builder.arguments.push_str(args);
                                                            }
                                                        }

                                                        // 检查是否完成
                                                        if builder.is_complete() {
                                                            if let Some(tool_call) = builder.build() {
                                                                yield StreamEvent::ToolCallComplete(tool_call);
                                                            }
                                                            tool_calls_builders.remove(&index);
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // Usage
                                        if let Some(usage) = chunk.get("usage") {
                                            if let (Some(prompt), Some(completion), Some(total)) = (
                                                usage.get("prompt_tokens").and_then(|v| v.as_u64()),
                                                usage.get("completion_tokens").and_then(|v| v.as_u64()),
                                                usage.get("total_tokens").and_then(|v| v.as_u64()),
                                            ) {
                                                yield StreamEvent::Usage(Usage {
                                                    prompt_tokens: prompt,
                                                    completion_tokens: completion,
                                                    total_tokens: total,
                                                });
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        yield StreamEvent::Error(crate::VllmError::Json(e.to_string()));
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield StreamEvent::Error(crate::VllmError::Http(e.to_string()));
                    }
                }
            }

            // 处理剩余的未完成工具调用
            for (_, builder) in tool_calls_builders {
                if let Some(tool_call) = builder.build() {
                    yield StreamEvent::ToolCallComplete(tool_call);
                }
            }

            yield StreamEvent::Done;
        };

        Self {
            inner: stream.boxed(),
        }
    }

    /// 从 BoxStream 创建流（用于测试）
    pub fn from_stream(stream: futures::stream::BoxStream<'static, StreamEvent>) -> Self {
        Self { inner: stream }
    }

    /// 获取下一个事件
    pub async fn next(&mut self) -> Option<StreamEvent> {
        use futures::StreamExt;
        self.inner.next().await
    }

    /// 收集所有内容为字符串
    pub async fn collect_content(self) -> Result<String, crate::VllmError> {
        use futures::StreamExt;

        let mut content = String::new();
        let mut stream = self.inner;

        while let Some(event) = stream.next().await {
            match event {
                StreamEvent::Content(delta) => content.push_str(&delta),
                StreamEvent::Error(e) => return Err(e),
                StreamEvent::Done => break,
                _ => {}
            }
        }

        Ok(content)
    }

    /// 转换为 Stream
    pub fn into_stream(self) -> futures::stream::BoxStream<'static, StreamEvent> {
        self.inner
    }
}

// ============================================================================
// Completion Stream (Legacy Completions)
// ============================================================================

/// Completions 流式响应对象
pub struct CompletionStream {
    inner: futures::stream::BoxStream<'static, CompletionStreamEvent>,
}

impl CompletionStream {
    /// 从 Response 创建流
    pub fn new(response: reqwest::Response) -> Self {
        use async_stream::stream;
        use futures::StreamExt;

        let stream = stream! {
            let mut byte_stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk) = byte_stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&text);

                        // 解析 SSE 格式: "data: {...}\n\n"
                        while let Some(pos) = buffer.find("\n\n") {
                            let line = buffer[..pos].to_string();
                            buffer = buffer[pos + 2..].to_string();

                            if let Some(data) = line.strip_prefix("data: ") {
                                if data == "[DONE]" {
                                    yield CompletionStreamEvent::Done;
                                    return;
                                }

                                match serde_json::from_str::<serde_json::Value>(data) {
                                    Ok(chunk) => {
                                        // 解析 chunk - completions 格式
                                        if let Some(choices) = chunk.get("choices").and_then(|c| c.as_array()) {
                                            for choice in choices {
                                                // 文本内容 (completions 使用 "text" 而不是 "delta.content")
                                                if let Some(text) = choice.get("text").and_then(|t| t.as_str()) {
                                                    if !text.is_empty() {
                                                        yield CompletionStreamEvent::Text(text.to_string());
                                                    }
                                                }

                                                // finish_reason
                                                if let Some(reason) = choice.get("finish_reason").and_then(|r| r.as_str()) {
                                                    if reason != "null" {
                                                        yield CompletionStreamEvent::FinishReason(reason.to_string());
                                                    }
                                                }
                                            }
                                        }

                                        // Usage
                                        if let Some(usage) = chunk.get("usage") {
                                            if let (Some(prompt), Some(completion), Some(total)) = (
                                                usage.get("prompt_tokens").and_then(|v| v.as_u64()),
                                                usage.get("completion_tokens").and_then(|v| v.as_u64()),
                                                usage.get("total_tokens").and_then(|v| v.as_u64()),
                                            ) {
                                                yield CompletionStreamEvent::Usage(Usage {
                                                    prompt_tokens: prompt,
                                                    completion_tokens: completion,
                                                    total_tokens: total,
                                                });
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        yield CompletionStreamEvent::Error(crate::VllmError::Json(e.to_string()));
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield CompletionStreamEvent::Error(crate::VllmError::Http(e.to_string()));
                    }
                }
            }
        };

        Self {
            inner: stream.boxed()
        }
    }

    /// 获取下一个事件
    pub async fn next(&mut self) -> Option<CompletionStreamEvent> {
        use futures::StreamExt;
        self.inner.next().await
    }

    /// 收集所有文本为字符串
    pub async fn collect_text(self) -> Result<String, crate::VllmError> {
        use futures::StreamExt;

        let mut text = String::new();
        let mut stream = self.inner;

        while let Some(event) = stream.next().await {
            match event {
                CompletionStreamEvent::Text(delta) => text.push_str(&delta),
                CompletionStreamEvent::Error(e) => return Err(e),
                CompletionStreamEvent::Done => break,
                _ => {}
            }
        }

        Ok(text)
    }

    /// 转换为 Stream
    pub fn into_stream(self) -> futures::stream::BoxStream<'static, CompletionStreamEvent> {
        self.inner
    }
}

/// Completions 流式事件
#[derive(Debug, Clone)]
pub enum CompletionStreamEvent {
    /// 文本增量
    Text(String),

    /// 结束原因
    FinishReason(String),

    /// Token 使用统计
    Usage(Usage),

    /// 流结束
    Done,

    /// 错误
    Error(crate::VllmError),
}

// ============================================================================
// Helper implementations
// ============================================================================

impl ChatCompletionResponse {
    /// 从原始 JSON 创建响应
    pub fn from_raw(raw: serde_json::Value) -> Result<Self, crate::VllmError> {
        let id = raw["id"].as_str().unwrap_or_default().to_string();

        let object = raw["object"]
            .as_str()
            .unwrap_or("chat.completion")
            .to_string();

        let model = raw["model"].as_str().unwrap_or_default().to_string();

        let created = raw["created"].as_u64().unwrap_or(0);

        let message = &raw["choices"][0]["message"];

        let content = message["content"].as_str().map(String::from);

        let reasoning_content = message["reasoning_content"].as_str().map(String::from);

        let tool_calls: Vec<ToolCall> = if let Some(calls) = message["tool_calls"].as_array() {
            calls
                .iter()
                .filter_map(|call| {
                    let id = call["id"].as_str()?.to_string();
                    let name = call["function"]["name"].as_str()?.to_string();
                    let arguments = call["function"]["arguments"].as_str()?.to_string();
                    Some(ToolCall {
                        id,
                        name,
                        arguments,
                    })
                })
                .collect()
        } else {
            Vec::new()
        };

        let finish_reason = raw["choices"][0]["finish_reason"]
            .as_str()
            .map(String::from);

        let usage = raw.get("usage").and_then(|u| {
            Some(Usage {
                prompt_tokens: u["prompt_tokens"].as_u64()?,
                completion_tokens: u["completion_tokens"].as_u64()?,
                total_tokens: u["total_tokens"].as_u64()?,
            })
        });

        Ok(ChatCompletionResponse {
            raw,
            id,
            object,
            model,
            created,
            content,
            reasoning_content,
            tool_calls,
            finish_reason,
            usage,
        })
    }
}

impl<'de> Deserialize<'de> for ChatCompletionResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = serde_json::Value::deserialize(deserializer)?;
        Self::from_raw(raw).map_err(serde::de::Error::custom)
    }
}

/// 用于构建增量工具调用的辅助结构
#[derive(Default)]
struct ToolCallBuilder {
    id: Option<String>,
    name: Option<String>,
    arguments: String,
}

impl ToolCallBuilder {
    fn is_complete(&self) -> bool {
        self.id.is_some() && self.name.is_some()
    }

    fn build(&self) -> Option<ToolCall> {
        Some(ToolCall {
            id: self.id.clone()?,
            name: self.name.clone()?,
            arguments: self.arguments.clone(),
        })
    }
}
