//! Phase 1.2: 错误处理测试
//!
//! 测试 VllmError 的创建、转换和显示

use vllm_client::VllmError;

// ============================================================================
// Test: test_error_display
// 输入: VllmError::ApiError { status_code: 404, message: "Not found" }
// 预期: format!("{}", err) 包含 "404" 和 "Not found"
// ============================================================================
#[test]
fn test_error_display() {
    let err = VllmError::api(404, "Not found");
    let msg = format!("{}", err);

    assert!(
        msg.contains("404"),
        "Error message should contain status code"
    );
    assert!(
        msg.contains("Not found"),
        "Error message should contain error message"
    );
}

// ============================================================================
// Test: test_error_display_timeout
// 输入: VllmError::Timeout
// 预期: format!("{}", err) 包含 "timeout"
// ============================================================================
#[test]
fn test_error_display_timeout() {
    let err = VllmError::Timeout;
    let msg = format!("{}", err);

    assert!(msg.to_lowercase().contains("timeout"));
}

// ============================================================================
// Test: test_error_display_stream
// 输入: VllmError::Stream("connection lost")
// 预期: format!("{}", err) 包含 "connection lost"
// ============================================================================
#[test]
fn test_error_display_stream() {
    let err = VllmError::Stream("connection lost".to_string());
    let msg = format!("{}", err);

    assert!(msg.contains("connection lost"));
}

// ============================================================================
// Test: test_error_display_model_not_found
// 输入: VllmError::ModelNotFound("gpt-5")
// 预期: format!("{}", err) 包含 "Model not found" 和 "gpt-5"
// ============================================================================
#[test]
fn test_error_display_model_not_found() {
    let err = VllmError::ModelNotFound("gpt-5".to_string());
    let msg = format!("{}", err);

    assert!(msg.contains("Model not found"));
    assert!(msg.contains("gpt-5"));
}

// ============================================================================
// Test: test_error_from_json
// 输入: serde_json::Error 转换
// 预期: 自动转换为 VllmError::Json
// ============================================================================
#[test]
fn test_error_from_json() {
    let json_err: serde_json::Error =
        serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();

    let vllm_err: VllmError = json_err.into();

    match vllm_err {
        VllmError::Json(_) => (), // 通过
        _ => panic!("Expected VllmError::Json variant"),
    }
}

// ============================================================================
// Test: test_error_api_helper
// 输入: VllmError::api(500, "Internal Server Error")
// 预期: 创建 ApiError 变体，error_type 为 None
// ============================================================================
#[test]
fn test_error_api_helper() {
    let err = VllmError::api(500, "Internal Server Error");

    match err {
        VllmError::ApiError {
            status_code,
            message,
            error_type,
        } => {
            assert_eq!(status_code, 500);
            assert_eq!(message, "Internal Server Error");
            assert!(error_type.is_none());
        }
        _ => panic!("Expected VllmError::ApiError variant"),
    }
}

// ============================================================================
// Test: test_error_api_with_type_helper
// 输入: VllmError::api_with_type(401, "Unauthorized", "auth_error")
// 预期: 创建 ApiError 变体，error_type 为 Some("auth_error")
// ============================================================================
#[test]
fn test_error_api_with_type_helper() {
    let err = VllmError::api_with_type(401, "Unauthorized", "auth_error");

    match err {
        VllmError::ApiError {
            status_code,
            message,
            error_type,
        } => {
            assert_eq!(status_code, 401);
            assert_eq!(message, "Unauthorized");
            assert_eq!(error_type, Some("auth_error".to_string()));
        }
        _ => panic!("Expected VllmError::ApiError variant"),
    }
}

// ============================================================================
// Test: test_error_is_retryable_timeout
// 输入: VllmError::Timeout
// 预期: is_retryable() 返回 true
// ============================================================================
#[test]
fn test_error_is_retryable_timeout() {
    let err = VllmError::Timeout;
    assert!(err.is_retryable());
}

// ============================================================================
// Test: test_error_is_retryable_rate_limit
// 输入: VllmError::ApiError { status_code: 429, ... }
// 预期: is_retryable() 返回 true
// ============================================================================
#[test]
fn test_error_is_retryable_rate_limit() {
    let err = VllmError::api(429, "Rate limit exceeded");
    assert!(err.is_retryable());
}

// ============================================================================
// Test: test_error_is_retryable_server_error
// 输入: VllmError::ApiError { status_code: 500/502/503/504, ... }
// 预期: is_retryable() 返回 true
// ============================================================================
#[test]
fn test_error_is_retryable_server_error() {
    for status in [500, 502, 503, 504] {
        let err = VllmError::api(status, "Server error");
        assert!(err.is_retryable(), "Status {} should be retryable", status);
    }
}

// ============================================================================
// Test: test_error_is_retryable_client_error
// 输入: VllmError::ApiError { status_code: 400/401/404, ... }
// 预期: is_retryable() 返回 false
// ============================================================================
#[test]
fn test_error_is_retryable_client_error() {
    for status in [400, 401, 403, 404] {
        let err = VllmError::api(status, "Client error");
        assert!(
            !err.is_retryable(),
            "Status {} should not be retryable",
            status
        );
    }
}

// ============================================================================
// Test: test_error_is_retryable_other
// 输入: VllmError::Json / VllmError::Stream
// 预期: is_retryable() 返回 false
// ============================================================================
#[test]
fn test_error_is_retryable_other() {
    // Json error
    let json_err: VllmError = serde_json::from_str::<serde_json::Value>("bad")
        .unwrap_err()
        .into();
    assert!(!json_err.is_retryable());

    // Stream error
    let stream_err = VllmError::Stream("error".to_string());
    assert!(!stream_err.is_retryable());
}

// ============================================================================
// Test: test_error_missing_parameter
// 输入: VllmError::MissingParameter("model")
// 预期: format!("{}", err) 包含 "Missing required parameter" 和 "model"
// ============================================================================
#[test]
fn test_error_missing_parameter() {
    let err = VllmError::MissingParameter("model".to_string());
    let msg = format!("{}", err);

    assert!(msg.contains("Missing required parameter"));
    assert!(msg.contains("model"));
}

// ============================================================================
// Test: test_error_no_content
// 输入: VllmError::NoContent
// 预期: format!("{}", err) 包含 "No response content"
// ============================================================================
#[test]
fn test_error_no_content() {
    let err = VllmError::NoContent;
    let msg = format!("{}", err);

    assert!(msg.contains("No response content"));
}

// ============================================================================
// Test: test_error_invalid_response
// 输入: VllmError::InvalidResponse("missing choices")
// 预期: format!("{}", err) 包含 "Invalid response format" 和 "missing choices"
// ============================================================================
#[test]
fn test_error_invalid_response() {
    let err = VllmError::InvalidResponse("missing choices".to_string());
    let msg = format!("{}", err);

    assert!(msg.contains("Invalid response format"));
    assert!(msg.contains("missing choices"));
}

// ============================================================================
// Test: test_error_debug_impl
// 输入: VllmError::api(404, "Not found")
// 预期: debug format 包含必要信息
// ============================================================================
#[test]
fn test_error_debug_impl() {
    let err = VllmError::api(404, "Not found");
    let debug_msg = format!("{:?}", err);

    assert!(debug_msg.contains("ApiError"));
    assert!(debug_msg.contains("404"));
    assert!(debug_msg.contains("Not found"));
}
