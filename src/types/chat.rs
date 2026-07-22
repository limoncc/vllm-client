//! Chat Completion 响应类型

use serde::Deserialize;

use super::ToolCall;
use super::Usage;
use crate::VllmError;

// ============================================================================
// Chat Completion Response
// ============================================================================

/// Chat Completion 响应
///
/// 封装了 OpenAI /v1/chat/completions 的响应数据。
/// 保留原始 JSON (`raw`) 以支持未来新增字段。
#[derive(Debug, Clone)]
pub struct ChatCompletionResponse {
    /// 原始 JSON 响应（保留所有字段，方便扩展）
    pub raw: serde_json::Value,

    /// 响应 ID
    pub id: String,

    /// 对象类型（通常为 "chat.completion"）
    pub object: String,

    /// 模型名称
    pub model: String,

    /// 创建时间戳
    pub created: u64,

    /// 助手回复内容
    pub content: Option<String>,

    /// 思考/推理内容（仅推理模型，如 DeepSeek-R1）
    pub reasoning_content: Option<String>,

    /// 工具调用列表
    pub tool_calls: Vec<ToolCall>,

    /// 结束原因（如 "stop"、"length"、"tool_calls"）
    pub finish_reason: Option<String>,

    /// Token 使用统计
    pub usage: Option<Usage>,
}

impl ChatCompletionResponse {
    /// 从原始 JSON 创建响应
    pub fn from_raw(raw: serde_json::Value) -> Result<Self, VllmError> {
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

impl<'de> Deserialize<'de> for ChatCompletionResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = serde_json::Value::deserialize(deserializer)?;
        Self::from_raw(raw).map_err(serde::de::Error::custom)
    }
}
