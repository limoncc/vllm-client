//! HTTP 请求公共逻辑
//!
//! 封装 API 请求发送、错误处理等通用操作。

use crate::error::VllmError;
use reqwest::RequestBuilder;

/// 为 HTTP GET/POST 加上 Bearer Auth 头部（如果配置了 API key）
pub fn with_auth(request: RequestBuilder, api_key: &Option<String>) -> RequestBuilder {
    if let Some(key) = api_key {
        request.bearer_auth(key)
    } else {
        request
    }
}

/// 发送请求并检查 HTTP 状态码，返回原始 JSON
///
/// 成功时返回 `serde_json::Value`，失败时解析出 API 错误信息并返回 `VllmError`。
pub async fn send_and_parse(request: RequestBuilder) -> Result<serde_json::Value, VllmError> {
    let response = request.send().await?;
    check_status(response).await
}

/// 发送请求并返回原始的 `reqwest::Response`（用于流式消费）
pub async fn send_for_stream(request: RequestBuilder) -> Result<reqwest::Response, VllmError> {
    let response = request.send().await?;
    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(VllmError::api(status.as_u16(), error_text));
    }
    Ok(response)
}

/// 检查 HTTP 状态码，错误时返回 VllmError
async fn check_status(response: reqwest::Response) -> Result<serde_json::Value, VllmError> {
    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(VllmError::api(status.as_u16(), error_text));
    }
    Ok(response.json().await?)
}
