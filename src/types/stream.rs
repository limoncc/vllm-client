//! 流式响应事件类型
//!
//! 包括 Chat Completions 和 Legacy Completions 两种流的 SSE 解析。

use async_stream::stream;
use futures::StreamExt;
use std::collections::HashMap;

use super::usage::estimator::estimate_usage;
use super::ToolCall;
use super::Usage;
use crate::VllmError;

// ============================================================================
// 1. 状态管理：把散乱的变量封装起来
// ============================================================================

/// 流式解析过程中的状态容器
struct StreamState {
    /// 工具调用构建器（拼积木用的）
    tool_builders: HashMap<usize, ToolCallBuilder>,
    /// 统计产生的字符数（用于估算 Usage）
    completion_chars: usize,
    /// API 返回的官方 Usage（如果有）
    api_usage: Option<Usage>,
}

impl StreamState {
    fn new() -> Self {
        Self {
            tool_builders: HashMap::new(),
            completion_chars: 0,
            api_usage: None,
        }
    }

    /// 核心逻辑：处理一个解析好的 JSON 块，产出事件
    fn process_json(&mut self, chunk: serde_json::Value) -> Vec<StreamEvent> {
        let mut events = Vec::new();

        // 1. 处理 choices 内容
        if let Some(choices) = chunk.get("choices").and_then(|c| c.as_array()) {
            for choice in choices {
                // 利用辅助方法处理具体内容，代码更整洁
                events.extend(self.process_choice(choice));
            }
        }

        // 2. 处理 usage（通常是最后一个 chunk）
        if let Some(u) = chunk.get("usage") {
            self.api_usage = parse_usage_from_json(u);
        }

        events
    }

    /// 处理单个 choice 的 delta
    fn process_choice(&mut self, choice: &serde_json::Value) -> Vec<StreamEvent> {
        let mut events = Vec::new();
        let delta = &choice["delta"];

        // A. 普通文本
        if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
            if !content.is_empty() {
                self.completion_chars += content.len();
                events.push(StreamEvent::Content(content.to_string()));
            }
        }

        // B. 推理内容
        if let Some(r) = delta
            .get("reasoning")
            .and_then(|c| c.as_str())
            .or_else(|| delta.get("reasoning_content").and_then(|c| c.as_str()))
        {
            if !r.is_empty() {
                self.completion_chars += r.len();
                events.push(StreamEvent::Reasoning(r.to_string()));
            }
        }

        // C. 工具调用
        if let Some(tool_calls) = delta.get("tool_calls").and_then(|c| c.as_array()) {
            for tc in tool_calls {
                if let Some(event) = self.process_tool_call(tc) {
                    events.push(event);
                }
            }
        }

        events
    }

    /// 处理工具调用片段，如果拼完了就返回完成事件
    fn process_tool_call(&mut self, tc: &serde_json::Value) -> Option<StreamEvent> {
        let idx = tc.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
        let builder = self.tool_builders.entry(idx).or_default();

        // 更新积木
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

        // 检查是否拼完
        if builder.is_complete() {
            let tc = builder.build()?;
            self.tool_builders.remove(&idx); // 清理
            return Some(StreamEvent::ToolCallComplete(tc));
        }
        None
    }

    /// 结束时的收尾工作
    fn finalize(&mut self, messages_text: &Option<String>) -> Vec<StreamEvent> {
        let mut events = Vec::new();

        // 1. 强制完成未结束的工具调用（容错）
        for (_, builder) in self.tool_builders.drain() {
            if let Some(tc) = builder.build() {
                events.push(StreamEvent::ToolCallComplete(tc));
            }
        }

        // 2. 发送 Usage
        let usage = self
            .api_usage
            .take()
            .unwrap_or_else(|| estimate_usage(messages_text, self.completion_chars));
        events.push(StreamEvent::Usage(usage));

        events
    }
}

// ============================================================================
// 2. SSE 解析辅助：把字符串切割逻辑抽离出来
// ============================================================================

/// 从缓冲区中尝试提取一行 SSE 数据
/// 返回 (行内容, 消耗的字节数)
fn try_extract_line(buffer: &str) -> Option<(String, usize)> {
    // 优先找标准 SSE 分隔符 \n\n
    if let Some(pos) = buffer.find("\n\n") {
        let line = buffer[..pos].to_string();
        return Some((line, pos + 2));
    }
    // 兼容非标准格式
    if let Some(pos) = buffer.find('\n') {
        let line = buffer[..pos].to_string();
        if line.starts_with("data: ") {
            return Some((line, pos + 1));
        }
    }
    None
}

// 辅助函数：解析 Usage 的 JSON
fn parse_usage_from_json(u: &serde_json::Value) -> Option<Usage> {
    let prompt = u.get("prompt_tokens")?.as_u64()?;
    let completion = u.get("completion_tokens")?.as_u64()?;
    let total = u.get("total_tokens")?.as_u64()?;
    Some(Usage {
        prompt_tokens: prompt,
        completion_tokens: completion,
        total_tokens: total,
    })
}
// ============================================================================
// StreamEvent — Chat Completions
// ============================================================================

/// 流式响应事件（Chat Completions）
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// 文本内容增量
    Content(String),

    /// 思考/推理内容增量（仅推理模型）
    Reasoning(String),

    /// 工具调用增量（逐步汇集成完整 ToolCall）
    ToolCallDelta {
        index: usize,
        id: Option<String>,
        name: Option<String>,
        arguments: Option<String>,
    },

    /// 工具调用完成
    ToolCallComplete(ToolCall),

    /// Token 使用统计（流结束时发出）
    Usage(Usage),

    /// 流结束
    Done,

    /// 错误
    Error(VllmError),
}

/// Chat Completions 流式响应
///
/// 封装 SSE 字节流并产出结构化的 `StreamEvent`。
pub struct MessageStream {
    inner: futures::stream::BoxStream<'static, StreamEvent>,
}

impl MessageStream {
    /// 从 HTTP Response 创建流
    pub fn new(response: reqwest::Response) -> Self {
        Self::with_context(response, None)
    }

    /// 从 Response 创建流，附带消息文本用于 prompt token 估算
    pub fn with_context(response: reqwest::Response, messages_text: Option<String>) -> Self {
        let stream = stream! {
            let mut byte_stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut state = StreamState::new(); // 状态统一管理

            // 主循环：只负责 IO 和 流程控制
            while let Some(chunk) = byte_stream.next().await {
                // 1. 错误处理
                let bytes = match chunk {
                    Ok(b) => b,
                    Err(e) => {
                        yield StreamEvent::Error(VllmError::Http(e.to_string()));
                        continue;
                    }
                };

                // 2. 填充缓冲区
                buffer.push_str(&String::from_utf8_lossy(&bytes));

                // 3. 解析循环
                while let Some((line, consumed)) = try_extract_line(&buffer) {
                    buffer = buffer[consumed..].to_string(); // 修剪缓冲区

                    if line.is_empty() { continue; }

                    // 4. 处理 SSE 数据头
                    let data = match line.strip_prefix("data: ").or_else(|| line.strip_prefix("data:")) {
                        Some(d) => d,
                        None => continue,
                    };

                    // 5. 结束信号
                    if data == "[DONE]" {
                        for event in state.finalize(&messages_text) {
                            yield event;
                        }
                        yield StreamEvent::Done;
                        return;
                    }

                    // 6. JSON 解析
                    let chunk: serde_json::Value = match serde_json::from_str(data) {
                        Ok(c) => c,
                        Err(e) => {
                            yield StreamEvent::Error(VllmError::Json(e.to_string()));
                            continue;
                        }
                    };

                    // 7. 业务处理（交给状态机去算）
                    for event in state.process_json(chunk) {
                        yield event;
                    }
                }
            }

            // [兜底] 流自然结束时（未收到 [DONE] 信号），发出 Usage 和 Done
            for event in state.finalize(&messages_text) {
                yield event;
            }
            yield StreamEvent::Done;
        };

        Self {
            inner: stream.boxed(),
        }
    }

    /// 从 BoxStream 创建（用于测试）
    pub fn from_stream(stream: futures::stream::BoxStream<'static, StreamEvent>) -> Self {
        Self { inner: stream }
    }

    /// 获取下一个事件
    pub async fn next(&mut self) -> Option<StreamEvent> {
        self.inner.next().await
    }

    /// 收集所有文本内容为字符串
    pub async fn collect_content(self) -> Result<String, VllmError> {
        let mut content = String::new();
        let mut s = self.inner;
        while let Some(event) = s.next().await {
            match event {
                StreamEvent::Content(delta) => content.push_str(&delta),
                StreamEvent::Error(e) => return Err(e),
                StreamEvent::Done => break,
                _ => {}
            }
        }
        Ok(content)
    }

    /// 转换为原始 Stream
    pub fn into_stream(self) -> futures::stream::BoxStream<'static, StreamEvent> {
        self.inner
    }
}

// ============================================================================
// CompletionStream — Legacy Completions
// ============================================================================

/// Legacy Completions 流式事件
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
    Error(VllmError),
}

/// Legacy Completions 流式响应
pub struct CompletionStream {
    inner: futures::stream::BoxStream<'static, CompletionStreamEvent>,
}

impl CompletionStream {
    /// 从 HTTP Response 创建流
    pub fn new(response: reqwest::Response) -> Self {
        let stream = stream! {
            let mut byte_stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk) = byte_stream.next().await {
                let Ok(bytes) = chunk else {
                    yield CompletionStreamEvent::Error(VllmError::Http(chunk.unwrap_err().to_string()));
                    continue;
                };
                buffer.push_str(&String::from_utf8_lossy(&bytes));

                while let Some(pos) = buffer.find("\n\n") {
                    let line = buffer[..pos].to_string();
                    buffer = buffer[pos + 2..].to_string();

                    let Some(data) = line.strip_prefix("data: ") else { continue; };
                    if data == "[DONE]" {
                        yield CompletionStreamEvent::Done;
                        return;
                    }

                    let chunk: serde_json::Value = match serde_json::from_str(data) {
                        Ok(c) => c,
                        Err(e) => { yield CompletionStreamEvent::Error(VllmError::Json(e.to_string())); continue; }
                    };

                    if let Some(choices) = chunk.get("choices").and_then(|c| c.as_array()) {
                        for choice in choices {
                            if let Some(text) = choice.get("text").and_then(|t| t.as_str()) {
                                if !text.is_empty() {
                                    yield CompletionStreamEvent::Text(text.to_string());
                                }
                            }
                            if let Some(reason) = choice.get("finish_reason").and_then(|r| r.as_str()) {
                                if reason != "null" {
                                    yield CompletionStreamEvent::FinishReason(reason.to_string());
                                }
                            }
                        }
                    }

                    if let Some(u) = chunk.get("usage") {
                        if let (Some(prompt), Some(completion), Some(total)) = (
                            u.get("prompt_tokens").and_then(|v| v.as_u64()),
                            u.get("completion_tokens").and_then(|v| v.as_u64()),
                            u.get("total_tokens").and_then(|v| v.as_u64()),
                        ) {
                            yield CompletionStreamEvent::Usage(Usage {
                                prompt_tokens: prompt,
                                completion_tokens: completion,
                                total_tokens: total,
                            });
                        }
                    }
                }
            }
        };

        Self {
            inner: stream.boxed(),
        }
    }

    /// 获取下一个事件
    pub async fn next(&mut self) -> Option<CompletionStreamEvent> {
        self.inner.next().await
    }

    /// 收集所有文本
    pub async fn collect_text(self) -> Result<String, VllmError> {
        let mut text = String::new();
        let mut s = self.inner;
        while let Some(event) = s.next().await {
            match event {
                CompletionStreamEvent::Text(delta) => text.push_str(&delta),
                CompletionStreamEvent::Error(e) => return Err(e),
                CompletionStreamEvent::Done => break,
                _ => {}
            }
        }
        Ok(text)
    }

    /// 转换为原始 Stream
    pub fn into_stream(self) -> futures::stream::BoxStream<'static, CompletionStreamEvent> {
        self.inner
    }
}

// ============================================================================
// 工具调用构造器（流式解析内部辅助）
// ============================================================================

/// 流式累积工具调用片段的内部辅助结构
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
