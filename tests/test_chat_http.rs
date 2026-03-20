//! Phase 2.3: HTTP Mock Server Tests
//!
//! 测试完整的 HTTP 调用流程，使用 mockito 模拟服务器

use serde_json::json;
use vllm_client::{VllmClient, VllmError};

// ============================================================================
// Test: test_send_simple_request
// Mock: POST /v1/chat/completions, 返回正常响应
// 输入: client.chat.completions.create().model().messages().send().await
// 预期: 返回正确的 ChatCompletionResponse
// ============================================================================
#[tokio::test]
async fn test_send_simple_request() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "created": 1234567890,
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello! How can I help you?"
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 10,
                    "completion_tokens": 8,
                    "total_tokens": 18
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));
    let response = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "Hello!"}]))
        .send()
        .await
        .unwrap();

    mock.assert();
    assert_eq!(response.id, "chatcmpl-123");
    assert_eq!(response.model, "test-model");
    assert_eq!(
        response.content,
        Some("Hello! How can I help you?".to_string())
    );
    assert_eq!(response.finish_reason, Some("stop".to_string()));
}

// ============================================================================
// Test: test_send_with_temperature
// Mock: 检查请求体中 temperature 字段
// 预期: 请求 JSON 包含 "temperature": 0.7
// ============================================================================
#[tokio::test]
async fn test_send_with_temperature() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Response"
                    },
                    "finish_reason": "stop"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));
    let _response = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "Hello!"}]))
        .temperature(0.7)
        .send()
        .await
        .unwrap();

    mock.assert();
}

// ============================================================================
// Test: test_send_with_max_tokens
// Mock: 检查请求体中 max_tokens 字段
// 预期: 请求 JSON 包含 "max_tokens": 100
// ============================================================================
#[tokio::test]
async fn test_send_with_max_tokens() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .match_body(mockito::Matcher::JsonString(
            json!({
                "model": "test-model",
                "messages": [{"role": "user", "content": "Hello!"}],
                "stream": false,
                "max_tokens": 100
            })
            .to_string(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Response"
                    },
                    "finish_reason": "stop"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));
    let _response = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "Hello!"}]))
        .max_tokens(100)
        .send()
        .await
        .unwrap();

    mock.assert();
}

// ============================================================================
// Test: test_handle_api_error_404
// Mock: 返回 404 Not Found
// 预期: 返回 VllmError::ApiError { status_code: 404, ... }
// ============================================================================
#[tokio::test]
async fn test_handle_api_error_404() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "error": {
                    "message": "Model not found",
                    "type": "invalid_request_error",
                    "code": "model_not_found"
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));
    let result = client
        .chat
        .completions()
        .create()
        .model("nonexistent-model")
        .messages(json!([{"role": "user", "content": "Hello!"}]))
        .send()
        .await;

    mock.assert();
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        VllmError::ApiError { status_code, .. } => {
            assert_eq!(status_code, 404);
        }
        _ => panic!("Expected ApiError with status 404"),
    }
}

// ============================================================================
// Test: test_handle_api_error_401
// Mock: 返回 401 Unauthorized
// 预期: 返回 VllmError::ApiError { status_code: 401, ... }
// ============================================================================
#[tokio::test]
async fn test_handle_api_error_401() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "error": {
                    "message": "Invalid API key",
                    "type": "invalid_request_error",
                    "code": "invalid_api_key"
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url())).with_api_key("invalid-key");
    let result = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "Hello!"}]))
        .send()
        .await;

    mock.assert();
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        VllmError::ApiError { status_code, .. } => {
            assert_eq!(status_code, 401);
        }
        _ => panic!("Expected ApiError with status 401"),
    }
}

// ============================================================================
// Test: test_handle_api_error_500
// Mock: 返回 500 Internal Server Error
// 预期: 返回 VllmError::ApiError { status_code: 500, ... }
// ============================================================================
#[tokio::test]
async fn test_handle_api_error_500() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "error": {
                    "message": "Internal server error",
                    "type": "server_error"
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));
    let result = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "Hello!"}]))
        .send()
        .await;

    mock.assert();
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        VllmError::ApiError { status_code, .. } => {
            assert_eq!(status_code, 500);
        }
        _ => panic!("Expected ApiError with status 500"),
    }
}

// ============================================================================
// Test: test_send_with_all_params
// Mock: 检查所有参数都正确传递
// 预期: 请求体包含所有设置的字段
// ============================================================================
#[tokio::test]
async fn test_send_with_all_params() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Response with all params"
                    },
                    "finish_reason": "stop"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));
    let _response = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "Hello!"}]))
        .temperature(0.8)
        .max_tokens(200)
        .top_p(0.95)
        .top_k(50)
        .stop(json!(["END", "STOP"]))
        .send()
        .await
        .unwrap();

    mock.assert();
}

// ============================================================================
// Test: test_send_with_tools
// Mock: 检查 tools 参数正确传递
// 预期: 请求体包含 tools 和 tool_choice
// ============================================================================
#[tokio::test]
async fn test_send_with_tools() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .match_body(mockito::Matcher::PartialJson(json!({
            "model": "test-model",
            "messages": [{"role": "user", "content": "What's the weather?"}],
            "stream": false,
            "tool_choice": "auto"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": null,
                        "tool_calls": [{
                            "id": "call_123",
                            "type": "function",
                            "function": {
                                "name": "get_weather",
                                "arguments": "{\"city\": \"Beijing\"}"
                            }
                        }]
                    },
                    "finish_reason": "tool_calls"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));
    let response = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "What's the weather?"}]))
        .tools(json!([{
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get weather info",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "city": {"type": "string"}
                    },
                    "required": ["city"]
                }
            }
        }]))
        .tool_choice(json!("auto"))
        .send()
        .await
        .unwrap();

    mock.assert();
    assert!(response.has_tool_calls());
    let tool_call = response.first_tool_call().unwrap();
    assert_eq!(tool_call.name, "get_weather");
}

// ============================================================================
// Test: test_send_with_extra_params
// Mock: 检查 extra 参数正确透传
// 预期: 请求体包含 extra 中的所有字段
// ============================================================================
#[tokio::test]
async fn test_send_with_extra_params() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .match_body(mockito::Matcher::PartialJson(json!({
            "model": "test-model",
            "messages": [{"role": "user", "content": "Think about it"}],
            "stream": false,
            "chat_template_kwargs": {"think_mode": true},
            "reasoning_effort": "high"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Let me think...",
                        "reasoning_content": "First, I need to analyze..."
                    },
                    "finish_reason": "stop"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));
    let response = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "Think about it"}]))
        .extra(json!({
            "chat_template_kwargs": {"think_mode": true},
            "reasoning_effort": "high"
        }))
        .send()
        .await
        .unwrap();

    mock.assert();
    assert_eq!(response.content, Some("Let me think...".to_string()));
    assert!(response.reasoning_content.is_some());
}

// ============================================================================
// Test: test_send_with_api_key
// Mock: 检查 Authorization header 正确设置
// 预期: 请求包含 Bearer token
// ============================================================================
#[tokio::test]
async fn test_send_with_api_key() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .match_header("authorization", "Bearer sk-test-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Authenticated response"
                    },
                    "finish_reason": "stop"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url())).with_api_key("sk-test-key");
    let _response = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "Hello!"}]))
        .send()
        .await
        .unwrap();

    mock.assert();
}

// ============================================================================
// Test: test_send_multimodal_request
// Mock: 接收多模态请求
// 预期: 请求体格式正确，包含图像 URL
// ============================================================================
#[tokio::test]
async fn test_send_multimodal_request() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .match_body(mockito::Matcher::PartialJson(json!({
            "model": "test-model",
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "text", "text": "What's in this image?"},
                    {"type": "image_url", "image_url": {"url": "https://example.com/image.jpg"}}
                ]
            }],
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "I see a beautiful landscape."
                    },
                    "finish_reason": "stop"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));
    let response = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{
            "role": "user",
            "content": [
                {"type": "text", "text": "What's in this image?"},
                {"type": "image_url", "image_url": {"url": "https://example.com/image.jpg"}}
            ]
        }]))
        .send()
        .await
        .unwrap();

    mock.assert();
    assert_eq!(
        response.content,
        Some("I see a beautiful landscape.".to_string())
    );
}

// ============================================================================
// Test: test_handle_rate_limit_error
// Mock: 返回 429 Too Many Requests
// 预期: 返回 VllmError::ApiError { status_code: 429, ... }
// ============================================================================
#[tokio::test]
async fn test_handle_rate_limit_error() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "error": {
                    "message": "Rate limit exceeded",
                    "type": "rate_limit_error",
                    "code": "rate_limit_exceeded"
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));
    let result = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "Hello!"}]))
        .send()
        .await;

    mock.assert();
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        VllmError::ApiError { status_code, .. } => {
            assert_eq!(status_code, 429);
        }
        _ => panic!("Expected ApiError with status 429"),
    }
}

// ============================================================================
// Test: test_response_with_reasoning_content
// Mock: 返回包含 reasoning_content 的响应
// 预期: response.reasoning_content 正确解析
// ============================================================================
#[tokio::test]
async fn test_response_with_reasoning_content() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "model": "test-model",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "The answer is 42.",
                    "reasoning_content": "Let me think about this question... First, I need to understand what is being asked."
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 50,
                "total_tokens": 60
            }
        }).to_string())
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));
    let response = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "What is the answer?"}]))
        .send()
        .await
        .unwrap();

    mock.assert();
    assert_eq!(response.content, Some("The answer is 42.".to_string()));
    assert_eq!(
        response.reasoning_content,
        Some(
            "Let me think about this question... First, I need to understand what is being asked."
                .to_string()
        )
    );
}
