//! Error types for vllm-client

use thiserror::Error;

/// VLLM Client 错误类型
#[derive(Debug, Error, Clone)]
pub enum VllmError {
    /// HTTP 请求错误
    #[error("HTTP request failed: {0}")]
    Http(String),

    /// JSON 序列化/反序列化错误
    #[error("JSON error: {0}")]
    Json(String),

    /// API 返回错误
    #[error("API error (status {status_code}): {message}")]
    ApiError {
        status_code: u16,
        message: String,
        error_type: Option<String>,
    },

    /// 流式响应错误
    #[error("Stream error: {0}")]
    Stream(String),

    /// 连接超时
    #[error("Connection timeout")]
    Timeout,

    /// 模型未找到
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// 缺少必需参数
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    /// 无响应内容
    #[error("No response content")]
    NoContent,

    /// 无效的响应格式
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    /// 其他错误
    #[error("{0}")]
    Other(String),
}

// 实现 From<reqwest::Error> for VllmError
impl From<reqwest::Error> for VllmError {
    fn from(err: reqwest::Error) -> Self {
        VllmError::Http(err.to_string())
    }
}

// 实现 From<serde_json::Error> for VllmError
impl From<serde_json::Error> for VllmError {
    fn from(err: serde_json::Error) -> Self {
        VllmError::Json(err.to_string())
    }
}

impl VllmError {
    /// 创建 API 错误
    pub fn api(status_code: u16, message: impl Into<String>) -> Self {
        Self::ApiError {
            status_code,
            message: message.into(),
            error_type: None,
        }
    }

    /// 创建 API 错误（别名，兼容旧代码）
    pub fn api_error(status_code: u16, message: impl Into<String>) -> Self {
        Self::api(status_code, message)
    }

    /// 创建带类型的 API 错误
    pub fn api_with_type(
        status_code: u16,
        message: impl Into<String>,
        error_type: impl Into<String>,
    ) -> Self {
        Self::ApiError {
            status_code,
            message: message.into(),
            error_type: Some(error_type.into()),
        }
    }

    /// 是否为可重试错误
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Timeout => true,
            Self::ApiError { status_code, .. } => {
                matches!(status_code, 429 | 500 | 502 | 503 | 504)
            }
            _ => false,
        }
    }
}

/// Result type alias for VllmError
pub type Result<T> = std::result::Result<T, VllmError>;
