//! 公共类型定义入口
//!
//! 集中管理所有 API 响应类型、流式事件类型和辅助类型。

pub(crate) mod chat;
pub(crate) mod completion;
pub(crate) mod stream;
pub(crate) mod usage;

pub use chat::ChatCompletionResponse;
pub use completion::{CompletionChoice, CompletionResponse};
pub use stream::{
    CompletionStream, CompletionStreamEvent, MessageStream, StreamEvent,
};
pub use usage::Usage;

use serde::{Deserialize, Serialize};
use crate::VllmError;

// ============================================================================
// 工具调用（Tool Call）
// ============================================================================

/// 工具调用
///
/// 由模型发起，包含函数名称和参数。
/// 调用者执行函数后将结果通过 `result()` 方法构造 tool 消息返回给模型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// 调用 ID（用于关联 tool result）
    pub id: String,

    /// 函数名称
    pub name: String,

    /// 函数参数（JSON 字符串）
    pub arguments: String,
}

impl ToolCall {
    /// 解析参数为 serde_json::Value
    pub fn parse_args(&self) -> Result<serde_json::Value, VllmError> {
        Ok(serde_json::from_str(&self.arguments)?)
    }

    /// 解析参数为指定类型
    pub fn parse_args_as<T: for<'de> Deserialize<'de>>(&self) -> Result<T, VllmError> {
        Ok(serde_json::from_str(&self.arguments)?)
    }

    /// 构造工具结果消息（用于追加到对话历史）
    pub fn result<T: Serialize>(&self, content: T) -> serde_json::Value {
        serde_json::json!({
            "role": "tool",
            "tool_call_id": self.id,
            "content": serde_json::to_string(&content).unwrap_or_default()
        })
    }
}
