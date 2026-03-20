//! Phase 2.2: Chat Response Parsing Tests
//!
//! 测试 ChatCompletionResponse 的解析和字段访问

use serde_json::json;
use vllm_client::{ChatCompletionResponse, ToolCall};

// ============================================================================
// Test: test_parse_simple_response
// 输入: Mock 响应 JSON
// 预期:
//   response.id == "chatcmpl-123"
//   response.content == Some("Hello!")
//   response.finish_reason == Some("stop")
//   response.has_tool_calls() == false
// ============================================================================
#[test]
fn test_parse_simple_response() {
    let raw = json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "model": "test-model",
        "created": 1234567890,
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "Hello!"},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
    });

    let response = ChatCompletionResponse::from_raw(raw).unwrap();

    assert_eq!(response.id, "chatcmpl-123");
    assert_eq!(response.object, "chat.completion");
    assert_eq!(response.model, "test-model");
    assert_eq!(response.content, Some("Hello!".to_string()));
    assert_eq!(response.finish_reason, Some("stop".to_string()));
    assert!(!response.has_tool_calls());
}

// ============================================================================
// Test: test_parse_response_with_reasoning
// 输入: 包含 reasoning_content 的响应
// 预期: response.reasoning_content == Some("思考内容...")
// ============================================================================
#[test]
fn test_parse_response_with_reasoning() {
    let raw = json!({
        "id": "chatcmpl-456",
        "object": "chat.completion",
        "model": "reasoning-model",
        "created": 1234567890,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Final answer",
                "reasoning_content": "Let me think about this step by step..."
            },
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 20, "completion_tokens": 10, "total_tokens": 30}
    });

    let response = ChatCompletionResponse::from_raw(raw).unwrap();

    assert_eq!(response.content, Some("Final answer".to_string()));
    assert_eq!(
        response.reasoning_content,
        Some("Let me think about this step by step...".to_string())
    );
}

// ============================================================================
// Test: test_parse_empty_content
// 输入: content 为 null 或缺失
// 预期: response.content == None, 程序不崩溃
// ============================================================================
#[test]
fn test_parse_empty_content() {
    // Test with null content
    let raw_with_null = json!({
        "id": "chatcmpl-789",
        "object": "chat.completion",
        "model": "test-model",
        "created": 1234567890,
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": null},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
    });

    let response = ChatCompletionResponse::from_raw(raw_with_null).unwrap();
    assert_eq!(response.content, None);

    // Test with missing content field
    let raw_without_content = json!({
        "id": "chatcmpl-790",
        "object": "chat.completion",
        "model": "test-model",
        "created": 1234567890,
        "choices": [{
            "index": 0,
            "message": {"role": "assistant"},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
    });

    let response = ChatCompletionResponse::from_raw(raw_without_content).unwrap();
    assert_eq!(response.content, None);
}

// ============================================================================
// Test: test_parse_usage
// 输入: 包含 usage 的响应
// 预期: usage.prompt_tokens, usage.completion_tokens 正确
// ============================================================================
#[test]
fn test_parse_usage() {
    let raw = json!({
        "id": "chatcmpl-usage",
        "object": "chat.completion",
        "model": "test-model",
        "created": 1234567890,
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "Response"},
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 100,
            "completion_tokens": 50,
            "total_tokens": 150
        }
    });

    let response = ChatCompletionResponse::from_raw(raw).unwrap();

    let usage = response.usage.as_ref().unwrap();
    assert_eq!(usage.prompt_tokens, 100);
    assert_eq!(usage.completion_tokens, 50);
    assert_eq!(usage.total_tokens, 150);
}

// ============================================================================
// Test: test_response_assistant_message
// 输入: 普通 assistant 响应
// 预期: response.assistant_message() 返回正确的 JSON
// ============================================================================
#[test]
fn test_response_assistant_message() {
    let raw = json!({
        "id": "chatcmpl-msg",
        "object": "chat.completion",
        "model": "test-model",
        "created": 1234567890,
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "Hello! How can I help?"},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 8, "total_tokens": 18}
    });

    let response = ChatCompletionResponse::from_raw(raw).unwrap();
    let msg = response.assistant_message();

    assert_eq!(msg["role"], "assistant");
    assert_eq!(msg["content"], "Hello! How can I help?");
    assert!(!msg.get("tool_calls").is_some());
}

// ============================================================================
// Test: test_response_raw_preserved
// 输入: 任意响应
// 预期: response.raw 保留原始 JSON，可访问任意字段
// ============================================================================
#[test]
fn test_response_raw_preserved() {
    let raw = json!({
        "id": "chatcmpl-raw",
        "object": "chat.completion",
        "model": "test-model",
        "created": 1234567890,
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "Test"},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15},
        "custom_field": "custom_value"
    });

    let response = ChatCompletionResponse::from_raw(raw.clone()).unwrap();

    // 原始 JSON 被保留
    assert_eq!(response.raw, raw);

    // 可以访问自定义字段
    assert_eq!(response.raw["custom_field"], "custom_value");
}

// ============================================================================
// Test: test_parse_response_with_tool_calls
// 输入: 包含 tool_calls 的响应
// 预期:
//   response.has_tool_calls() == true
//   response.tool_calls.len() == 1
//   call.id == "call_123"
//   call.name == "get_weather"
//   call.arguments == "{\"city\": \"Beijing\"}"
// ============================================================================
#[test]
fn test_parse_response_with_tool_calls() {
    let raw = json!({
        "id": "chatcmpl-tool",
        "object": "chat.completion",
        "model": "test-model",
        "created": 1234567890,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
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
        }],
        "usage": {"prompt_tokens": 20, "completion_tokens": 10, "total_tokens": 30}
    });

    let response = ChatCompletionResponse::from_raw(raw).unwrap();

    assert!(response.has_tool_calls());
    assert_eq!(response.tool_calls.len(), 1);

    let tool_call = &response.tool_calls[0];
    assert_eq!(tool_call.id, "call_123");
    assert_eq!(tool_call.name, "get_weather");
    assert_eq!(tool_call.arguments, "{\"city\": \"Beijing\"}");
}

// ============================================================================
// Test: test_parse_multiple_tool_calls
// 输入: 包含多个 tool_calls 的响应
// 预期: response.tool_calls.len() == 2, 所有调用正确解析
// ============================================================================
#[test]
fn test_parse_multiple_tool_calls() {
    let raw = json!({
        "id": "chatcmpl-multi-tool",
        "object": "chat.completion",
        "model": "test-model",
        "created": 1234567890,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "tool_calls": [
                    {
                        "id": "call_1",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"city\": \"Beijing\"}"
                        }
                    },
                    {
                        "id": "call_2",
                        "type": "function",
                        "function": {
                            "name": "get_time",
                            "arguments": "{\"timezone\": \"Asia/Shanghai\"}"
                        }
                    }
                ]
            },
            "finish_reason": "tool_calls"
        }],
        "usage": {"prompt_tokens": 30, "completion_tokens": 20, "total_tokens": 50}
    });

    let response = ChatCompletionResponse::from_raw(raw).unwrap();

    assert!(response.has_tool_calls());
    assert_eq!(response.tool_calls.len(), 2);

    assert_eq!(response.tool_calls[0].id, "call_1");
    assert_eq!(response.tool_calls[0].name, "get_weather");

    assert_eq!(response.tool_calls[1].id, "call_2");
    assert_eq!(response.tool_calls[1].name, "get_time");
}

// ============================================================================
// Test: test_tool_call_parse_args
// 输入: ToolCall { arguments: "{\"a\": 1}" }
// 预期: call.parse_args() 返回 json!({"a": 1})
// ============================================================================
#[test]
fn test_tool_call_parse_args() {
    let tool_call = ToolCall {
        id: "call_test".to_string(),
        name: "test_func".to_string(),
        arguments: "{\"a\": 1, \"b\": \"hello\"}".to_string(),
    };

    let args = tool_call.parse_args().unwrap();
    assert_eq!(args["a"], 1);
    assert_eq!(args["b"], "hello");
}

// ============================================================================
// Test: test_tool_call_parse_args_as_struct
// 输入: ToolCall { arguments: "{\"city\": \"Beijing\"}" }
// 预期: call.parse_args_as::<WeatherArgs>() 返回正确结构体
// ============================================================================
#[test]
fn test_tool_call_parse_args_as_struct() {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct WeatherArgs {
        city: String,
    }

    let tool_call = ToolCall {
        id: "call_weather".to_string(),
        name: "get_weather".to_string(),
        arguments: "{\"city\": \"Beijing\"}".to_string(),
    };

    let args: WeatherArgs = tool_call.parse_args_as().unwrap();
    assert_eq!(args.city, "Beijing");
}

// ============================================================================
// Test: test_tool_call_parse_invalid_args
// 输入: ToolCall { arguments: "invalid json" }
// 预期: parse_args() 返回错误
// ============================================================================
#[test]
fn test_tool_call_parse_invalid_args() {
    let tool_call = ToolCall {
        id: "call_invalid".to_string(),
        name: "test_func".to_string(),
        arguments: "invalid json".to_string(),
    };

    let result = tool_call.parse_args();
    assert!(result.is_err());
}

// ============================================================================
// Test: test_assistant_message_with_tool_calls
// 输入: 包含 tool_calls 的响应
// 预期: response.assistant_message() 包含 tool_calls 字段
// ============================================================================
#[test]
fn test_assistant_message_with_tool_calls() {
    let raw = json!({
        "id": "chatcmpl-tool-msg",
        "object": "chat.completion",
        "model": "test-model",
        "created": 1234567890,
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
        }],
        "usage": {"prompt_tokens": 20, "completion_tokens": 10, "total_tokens": 30}
    });

    let response = ChatCompletionResponse::from_raw(raw).unwrap();
    let msg = response.assistant_message();

    assert_eq!(msg["role"], "assistant");
    assert!(msg.get("tool_calls").is_some());
    assert!(msg["tool_calls"].is_array());
}

// ============================================================================
// Test: test_first_tool_call
// 输入: 包含 tool_calls 的响应
// 预期: response.first_tool_call() 返回第一个工具调用
// ============================================================================
#[test]
fn test_first_tool_call() {
    let raw = json!({
        "id": "chatcmpl-first",
        "object": "chat.completion",
        "model": "test-model",
        "created": 1234567890,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "tool_calls": [{
                    "id": "call_first",
                    "type": "function",
                    "function": {
                        "name": "first_func",
                        "arguments": "{}"
                    }
                }]
            },
            "finish_reason": "tool_calls"
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
    });

    let response = ChatCompletionResponse::from_raw(raw).unwrap();

    let first_call = response.first_tool_call().unwrap();
    assert_eq!(first_call.id, "call_first");
    assert_eq!(first_call.name, "first_func");
}

// ============================================================================
// Test: test_first_tool_call_empty
// 输入: 不包含 tool_calls 的响应
// 预期: response.first_tool_call() 返回 None
// ============================================================================
#[test]
fn test_first_tool_call_empty() {
    let raw = json!({
        "id": "chatcmpl-no-tool",
        "object": "chat.completion",
        "model": "test-model",
        "created": 1234567890,
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "Hello!"},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
    });

    let response = ChatCompletionResponse::from_raw(raw).unwrap();

    assert!(!response.has_tool_calls());
    assert!(response.first_tool_call().is_none());
}

// ============================================================================
// Test: test_tool_call_result_message
// 输入: call.result(json!({"temp": 25}))
// 预期: 返回正确的工具结果消息
// ============================================================================
#[test]
fn test_tool_call_result_message() {
    let tool_call = ToolCall {
        id: "call_123".to_string(),
        name: "get_weather".to_string(),
        arguments: "{\"city\": \"Beijing\"}".to_string(),
    };

    let result_msg = tool_call.result(json!({"temp": 25, "unit": "celsius"}));

    assert_eq!(result_msg["role"], "tool");
    assert_eq!(result_msg["tool_call_id"], "call_123");
    assert!(result_msg["content"].is_string());

    let content = result_msg["content"].as_str().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    assert_eq!(parsed["temp"], 25);
}

// ============================================================================
// Test: test_parse_response_finish_reasons
// 输入: 不同 finish_reason 的响应
// 预期: 都能正确解析
// ============================================================================
#[test]
fn test_parse_response_finish_reasons() {
    let finish_reasons = vec!["stop", "length", "tool_calls", "content_filter"];

    for reason in finish_reasons {
        let raw = json!({
            "id": "chatcmpl-finish",
            "object": "chat.completion",
            "model": "test-model",
            "created": 1234567890,
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": "Test"},
                "finish_reason": reason
            }],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
        });

        let response = ChatCompletionResponse::from_raw(raw).unwrap();
        assert_eq!(response.finish_reason, Some(reason.to_string()));
    }
}
