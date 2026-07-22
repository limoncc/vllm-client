//! Usage 类型与本地 token 估算

use serde::{Deserialize, Serialize};

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

/// 本地 token 估算（当 API 未返回 usage 时使用）
pub(crate) mod estimator {
    use super::Usage;

    /// 根据输入文本字符数和输出字符数粗略估算 token 用量
    ///
    /// 中英文混合场景约 2 字符 ≈ 1 token，仅作参考。
    pub fn estimate_usage(messages_text: &Option<String>, completion_chars: usize) -> Usage {
        let prompt_tokens = messages_text
            .as_ref()
            .map(|t| (t.len() / 2) as u64)
            .unwrap_or(0);
        let completion_tokens = completion_chars as u64;
        Usage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}
