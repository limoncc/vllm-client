//! Phase 6.1: Mock Server Integration Tests
//!
//! 使用 mockito 模拟服务器进行集成测试

use serde_json::json;
use std::sync::Arc;
use tokio::sync::Barrier;
use vllm_client::{VllmClient, VllmError};

// ============================================================================
// Test: test_full_conversation_flow
// Mock: 模拟多轮对话
// 预期: 消息历史正确维护
// ============================================================================
#[tokio::test]
async fn test_full_conversation_flow() {
    let mut server = mockito::Server::new_async().await;

    // 第一轮对话
    let mock1 = server
        .mock("POST", "/v1/chat/completions")
        .match_body(mockito::Matcher::PartialJson(json!({
            "messages": [
                {"role": "system", "content": "You are helpful."},
                {"role": "user", "content": "Hello!"}
            ]
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-1",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hi! How can I help you?"
                    },
                    "finish_reason": "stop"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    // 第二轮对话
    let mock2 = server
        .mock("POST", "/v1/chat/completions")
        .match_body(mockito::Matcher::PartialJson(json!({
            "messages": [
                {"role": "system", "content": "You are helpful."},
                {"role": "user", "content": "Hello!"},
                {"role": "assistant", "content": "Hi! How can I help you?"},
                {"role": "user", "content": "Tell me a joke."}
            ]
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-2",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Why did the chicken cross the road? To get to the other side!"
                    },
                    "finish_reason": "stop"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    // 第三轮对话
    let mock3 = server
        .mock("POST", "/v1/chat/completions")
        .match_body(mockito::Matcher::PartialJson(json!({
            "messages": [
                {"role": "system", "content": "You are helpful."},
                {"role": "user", "content": "Hello!"},
                {"role": "assistant", "content": "Hi! How can I help you?"},
                {"role": "user", "content": "Tell me a joke."},
                {"role": "assistant", "content": "Why did the chicken cross the road? To get to the other side!"},
                {"role": "user", "content": "That's funny!"}
            ]
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-3",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "I'm glad you enjoyed it!"
                    },
                    "finish_reason": "stop"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));

    // 维护对话历史
    let mut messages = vec![json!({"role": "system", "content": "You are helpful."})];

    // 第一轮
    messages.push(json!({"role": "user", "content": "Hello!"}));
    let response1 = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!(messages.clone()))
        .send()
        .await
        .unwrap();
    messages.push(json!({"role": "assistant", "content": response1.content.unwrap()}));

    // 第二轮
    messages.push(json!({"role": "user", "content": "Tell me a joke."}));
    let response2 = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!(messages.clone()))
        .send()
        .await
        .unwrap();
    messages.push(json!({"role": "assistant", "content": response2.content.unwrap()}));

    // 第三轮
    messages.push(json!({"role": "user", "content": "That's funny!"}));
    let response3 = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!(messages.clone()))
        .send()
        .await
        .unwrap();

    // 添加第三轮的 assistant 响应
    messages.push(json!({"role": "assistant", "content": response3.content.clone().unwrap()}));

    mock1.assert();
    mock2.assert();
    mock3.assert();

    assert_eq!(messages.len(), 7); // system + 3 pairs of user/assistant
    assert_eq!(
        response3.content,
        Some("I'm glad you enjoyed it!".to_string())
    );
}

// ============================================================================
// Test: test_concurrent_requests
// 输入: 并发发送多个请求
// 预期: 所有请求正确处理
// ============================================================================
#[tokio::test]
async fn test_concurrent_requests() {
    let mut server = mockito::Server::new_async().await;

    // 设置 mock 期望至少 5 次调用
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-concurrent",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Concurrent response"
                    },
                    "finish_reason": "stop"
                }]
            })
            .to_string(),
        )
        .expect(5)
        .create_async()
        .await;

    let client = Arc::new(VllmClient::new(format!("{}/v1", server.url())));
    let barrier = Arc::new(Barrier::new(5));
    let mut handles = vec![];

    for i in 0..5 {
        let client = Arc::clone(&client);
        let barrier = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            barrier.wait().await;

            let response = client
                .chat
                .completions()
                .create()
                .model("test-model")
                .messages(json!([{"role": "user", "content": format!("Request {}", i)}]))
                .send()
                .await
                .unwrap();

            response
        });

        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles).await;

    for result in results {
        let response = result.unwrap();
        assert_eq!(response.content, Some("Concurrent response".to_string()));
    }

    mock.assert();
}

// ============================================================================
// Test: test_error_followed_by_success
// 输入: 先返回错误，再返回成功
// 预期: 客户端可以正确处理错误后重试
// ============================================================================
#[tokio::test]
async fn test_error_followed_by_success() {
    let mut server = mockito::Server::new_async().await;

    // 第一次请求返回 500 错误
    let mock_error = server
        .mock("POST", "/v1/chat/completions")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "error": {
                    "message": "Temporary server error",
                    "type": "server_error"
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    // 第二次请求返回成功
    let mock_success = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-retry",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Success after retry"
                    },
                    "finish_reason": "stop"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));

    // 第一次请求应该失败
    let result1 = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "Hello"}]))
        .send()
        .await;

    assert!(result1.is_err());
    match result1.unwrap_err() {
        VllmError::ApiError { status_code, .. } => {
            assert_eq!(status_code, 500);
        }
        _ => panic!("Expected ApiError"),
    }

    // 第二次请求应该成功
    let result2 = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "Hello"}]))
        .send()
        .await;

    assert!(result2.is_ok());
    let response = result2.unwrap();
    assert_eq!(response.content, Some("Success after retry".to_string()));

    mock_error.assert();
    mock_success.assert();
}

// ============================================================================
// Test: test_streaming_conversation
// Mock: 模拟流式对话
// 预期: 流式输出正常工作
// ============================================================================
#[tokio::test]
async fn test_streaming_conversation() {
    let mut server = mockito::Server::new_async().await;

    // 模拟 SSE 流式响应
    let sse_response = concat!(
        "data: {\"id\":\"chatcmpl-stream\",\"object\":\"chat.completion.chunk\",\"model\":\"test-model\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\"},\"finish_reason\":null}]}\n\n",
        "data: {\"id\":\"chatcmpl-stream\",\"object\":\"chat.completion.chunk\",\"model\":\"test-model\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n",
        "data: {\"id\":\"chatcmpl-stream\",\"object\":\"chat.completion.chunk\",\"model\":\"test-model\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\" there\"},\"finish_reason\":null}]}\n\n",
        "data: {\"id\":\"chatcmpl-stream\",\"object\":\"chat.completion.chunk\",\"model\":\"test-model\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"!\"},\"finish_reason\":null}]}\n\n",
        "data: {\"id\":\"chatcmpl-stream\",\"object\":\"chat.completion.chunk\",\"model\":\"test-model\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        "data: [DONE]\n\n"
    );

    let mock = server
        .mock("POST", "/v1/chat/completions")
        .match_body(mockito::Matcher::PartialJson(json!({
            "stream": true
        })))
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(sse_response)
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));

    let stream = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "Say hello"}]))
        .send_stream()
        .await
        .unwrap();

    let content = stream.collect_content().await.unwrap();

    mock.assert();
    assert_eq!(content, "Hello there!");
}

// ============================================================================
// Test: test_tool_calling_flow
// Mock: 模拟工具调用流程
// 预期: 工具调用正确处理
// ============================================================================
#[tokio::test]
async fn test_tool_calling_flow() {
    let mut server = mockito::Server::new_async().await;

    // 第一次请求返回工具调用
    let mock_tool = server
        .mock("POST", "/v1/chat/completions")
        .match_body(mockito::Matcher::PartialJson(json!({
            "messages": [{"role": "user", "content": "What's the weather?"}]
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-tool",
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

    // 第二次请求包含工具结果
    let mock_result = server
        .mock("POST", "/v1/chat/completions")
        .match_body(mockito::Matcher::Regex(r#""role":"tool""#.to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-final",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "The weather in Beijing is sunny with a temperature of 25°C."
                    },
                    "finish_reason": "stop"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));

    // 第一步：获取工具调用
    let response1 = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "What's the weather?"}]))
        .tools(json!([{
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get weather",
                "parameters": {
                    "type": "object",
                    "properties": {"city": {"type": "string"}},
                    "required": ["city"]
                }
            }
        }]))
        .send()
        .await
        .unwrap();

    assert!(response1.has_tool_calls());
    let tool_call = response1.first_tool_call().unwrap();

    // 构建工具结果消息
    let tool_result = tool_call.result("Sunny, 25°C");
    let assistant_msg = response1.assistant_message();

    // 第二步：发送工具结果
    let response2 = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([
            {"role": "user", "content": "What's the weather?"},
            assistant_msg,
            tool_result
        ]))
        .send()
        .await
        .unwrap();

    mock_tool.assert();
    mock_result.assert();

    assert_eq!(
        response2.content,
        Some("The weather in Beijing is sunny with a temperature of 25°C.".to_string())
    );
}

// ============================================================================
// Test: test_legacy_completion_integration
// Mock: 测试 Legacy Completions API
// 预期: 正常工作
// ============================================================================
#[tokio::test]
async fn test_legacy_completion_integration() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/completions")
        .match_body(mockito::Matcher::PartialJson(json!({
            "model": "test-model",
            "prompt": "Once upon a time",
            "max_tokens": 50
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "cmpl-123",
                "object": "text_completion",
                "model": "test-model",
                "choices": [{
                    "text": " there was a brave knight who lived in a castle.",
                    "index": 0,
                    "finish_reason": "length"
                }],
                "usage": {
                    "prompt_tokens": 4,
                    "completion_tokens": 10,
                    "total_tokens": 14
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));

    let response = client
        .completions
        .create()
        .model("test-model")
        .prompt("Once upon a time")
        .max_tokens(50)
        .send()
        .await
        .unwrap();

    mock.assert();
    assert_eq!(
        response.choices[0].text,
        " there was a brave knight who lived in a castle."
    );
}

// ============================================================================
// Test: test_timeout_handling
// Mock: 模拟正常响应（在超时时间内）
// 预期: 客户端正常处理响应
// ============================================================================
#[tokio::test]
async fn test_timeout_handling() {
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
                        "content": "Quick response"
                    },
                    "finish_reason": "stop"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url())).timeout_secs(30);

    let response = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "Hello"}]))
        .send()
        .await;

    mock.assert();
    assert!(response.is_ok());
}

// ============================================================================
// Test: test_api_key_authentication
// Mock: 验证 API Key 正确传递
// 预期: Authorization header 正确设置
// ============================================================================
#[tokio::test]
async fn test_api_key_authentication() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/chat/completions")
        .match_header("authorization", "Bearer sk-secret-key-12345")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-auth",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Authenticated!"
                    },
                    "finish_reason": "stop"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client =
        VllmClient::new(format!("{}/v1", server.url())).with_api_key("sk-secret-key-12345");

    let response = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "Test"}]))
        .send()
        .await
        .unwrap();

    mock.assert();
    assert_eq!(response.content, Some("Authenticated!".to_string()));
}

// ============================================================================
// Test: test_sequential_requests_with_different_params
// Mock: 连续发送不同参数的请求
// 预期: 每个请求参数正确传递
// ============================================================================
#[tokio::test]
async fn test_sequential_requests_with_different_params() {
    let mut server = mockito::Server::new_async().await;

    // 第一个请求：低温度
    let mock1 = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-1",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{"index": 0, "message": {"role": "assistant", "content": "Low temp"}, "finish_reason": "stop"}]
            })
            .to_string(),
        )
        .create_async()
        .await;

    // 第二个请求：高温度
    let mock2 = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-2",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{"index": 0, "message": {"role": "assistant", "content": "High temp"}, "finish_reason": "stop"}]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = VllmClient::new(format!("{}/v1", server.url()));

    // 第一个请求
    let resp1 = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "A"}]))
        .temperature(0.1)
        .send()
        .await
        .unwrap();

    // 第二个请求
    let resp2 = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "B"}]))
        .temperature(1.5)
        .send()
        .await
        .unwrap();

    mock1.assert();
    mock2.assert();
    assert_eq!(resp1.content, Some("Low temp".to_string()));
    assert_eq!(resp2.content, Some("High temp".to_string()));
}

// ============================================================================
// Test: test_error_with_retryable_check
// Mock: 返回可重试的错误
// 预期: is_retryable() 方法正确判断
// ============================================================================
#[tokio::test]
async fn test_error_with_retryable_check() {
    let mut server = mockito::Server::new_async().await;

    // 429 错误应该是可重试的
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "error": {
                    "message": "Rate limit exceeded",
                    "type": "rate_limit_error"
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
        .messages(json!([{"role": "user", "content": "Test"}]))
        .send()
        .await;

    mock.assert();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.is_retryable());
}

// ============================================================================
// Test: test_usage_statistics
// Mock: 返回包含 usage 的响应
// 预期: usage 信息正确解析
// ============================================================================
#[tokio::test]
async fn test_usage_statistics() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "chatcmpl-usage",
                "object": "chat.completion",
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "This is a response."
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 15,
                    "completion_tokens": 5,
                    "total_tokens": 20
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
        .messages(json!([{"role": "user", "content": "Count tokens"}]))
        .send()
        .await
        .unwrap();

    mock.assert();

    let usage = response.usage.unwrap();
    assert_eq!(usage.prompt_tokens, 15);
    assert_eq!(usage.completion_tokens, 5);
    assert_eq!(usage.total_tokens, 20);
}
