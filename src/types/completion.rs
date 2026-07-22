//! Legacy Completion 响应类型（/v1/completions）

use serde::{Deserialize, Serialize};

use super::Usage;
use crate::VllmError;

/// Legacy Completion 响应
///
/// OpenAI 旧版 /v1/completions 接口的结构化响应。
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    /// 原始 JSON 响应
    pub raw: serde_json::Value,

    /// 响应 ID
    pub id: String,

    /// 对象类型（通常为 "text_completion"）
    pub object: String,

    /// 模型名称
    pub model: String,

    /// 选择列表
    pub choices: Vec<CompletionChoice>,

    /// Token 使用统计
    pub usage: Option<Usage>,
}

impl CompletionResponse {
    /// 从原始 JSON 解析
    pub fn from_raw(raw: serde_json::Value) -> Result<Self, VllmError> {
        let id = raw["id"]
            .as_str()
            .ok_or_else(|| VllmError::InvalidResponse("missing id".into()))?
            .to_string();
        let object = raw["object"]
            .as_str()
            .unwrap_or("text_completion")
            .to_string();
        let model = raw["model"]
            .as_str()
            .ok_or_else(|| VllmError::InvalidResponse("missing model".into()))?
            .to_string();
        let choices = raw["choices"]
            .as_array()
            .ok_or_else(|| VllmError::InvalidResponse("missing choices".into()))?
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

    /// 对数概率（可选）
    pub logprobs: Option<serde_json::Value>,

    /// 结束原因
    pub finish_reason: Option<String>,
}

impl CompletionChoice {
    /// 从原始 JSON 解析
    pub fn from_raw(raw: serde_json::Value) -> Result<Self, VllmError> {
        let index = raw["index"]
            .as_u64()
            .ok_or_else(|| VllmError::InvalidResponse("missing choice index".into()))?
            as u32;
        let text = raw["text"]
            .as_str()
            .ok_or_else(|| VllmError::InvalidResponse("missing choice text".into()))?
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
