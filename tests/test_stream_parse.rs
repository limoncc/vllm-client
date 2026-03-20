//! Phase 3.1: SSE Parsing Tests
//!
//! 测试 SSE (Server-Sent Events) 格式的解析

use serde_json::json;

// ============================================================================
// Test: test_sse_format_basic
// 输入: "data: {...}\n\n" 格式
// 预期: 理解 SSE 格式的基本结构
// ============================================================================
#[test]
fn test_sse_format_basic() {
    // SSE 格式的基本结构
    let sse_data = "data: {\"test\": \"value\"}\n\n";

    // 验证格式
    assert!(sse_data.starts_with("data: "));
    assert!(sse_data.ends_with("\n\n"));

    // 提取 JSON 部分
    let json_str = sse_data.strip_prefix("data: ").unwrap();
    let json_str = json_str.strip_suffix("\n\n").unwrap();

    // 解析 JSON
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert_eq!(parsed["test"], "value");
}

// ============================================================================
// Test: test_parse_stream_chunk_json
// 输入: 模拟 SSE chunk 的 JSON 数据
// 预期: 正确解析为 JSON 结构
// ============================================================================
#[test]
fn test_parse_stream_chunk_json() {
    // 模拟一个 SSE chunk 的 JSON
    let chunk = json!({
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "test-model",
        "choices": [{
            "index": 0,
            "delta": {
                "content": "Hello"
            },
            "finish_reason": null
        }]
    });

    // 验证结构
    assert_eq!(chunk["id"], "chatcmpl-123");
    assert_eq!(chunk["object"], "chat.completion.chunk");

    // 提取 delta content
    let delta = &chunk["choices"][0]["delta"];
    assert_eq!(delta["content"], "Hello");
}

// ============================================================================
// Test: test_parse_stream_done_format
// 输入: "data: [DONE]\n\n"
// 预期: 正确识别流结束标记
// ============================================================================
#[test]
fn test_parse_stream_done_format() {
    let done_marker = "data: [DONE]\n\n";

    // 提取数据部分
    if let Some(data) = done_marker.strip_prefix("data: ") {
        let data = data.strip_suffix("\n\n").unwrap();
        assert_eq!(data, "[DONE]");
    }
}

// ============================================================================
// Test: test_parse_multiline_sse_data
// 输入: 多个 SSE 事件
// 预期: 正确分割每个事件
// ============================================================================
#[test]
fn test_parse_multiline_sse_data() {
    let sse_stream = "data: {\"chunk\": 1}\n\ndata: {\"chunk\": 2}\n\ndata: [DONE]\n\n";

    // 模拟解析逻辑：按 "\n\n" 分割
    let events: Vec<&str> = sse_stream.split("\n\n").filter(|s| !s.is_empty()).collect();

    assert_eq!(events.len(), 3);
    assert_eq!(events[0], "data: {\"chunk\": 1}");
    assert_eq!(events[1], "data: {\"chunk\": 2}");
    assert_eq!(events[2], "data: [DONE]");

    // 提取每个事件的 JSON 数据
    for event in &events[..2] {
        if let Some(json_str) = event.strip_prefix("data: ") {
            let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
            assert!(parsed["chunk"].is_number());
        }
    }
}

// ============================================================================
// Test: test_delta_content_variants
// 输入: 不同类型的 delta content
// 预期: 能正确识别不同类型的内容
// ============================================================================
#[test]
fn test_delta_content_variants() {
    // 普通 content
    let delta_content = json!({
        "content": "Hello, world!"
    });
    assert!(delta_content.get("content").is_some());
    assert_eq!(delta_content["content"].as_str().unwrap(), "Hello, world!");

    // reasoning content (字段名: reasoning)
    let delta_reasoning = json!({
        "reasoning": "Thinking..."
    });
    assert!(delta_reasoning.get("reasoning").is_some());
    assert_eq!(
        delta_reasoning["reasoning"].as_str().unwrap(),
        "Thinking..."
    );

    // reasoning_content (字段名: reasoning_content)
    let delta_reasoning_content = json!({
        "reasoning_content": "Also thinking..."
    });
    assert!(delta_reasoning_content.get("reasoning_content").is_some());

    // tool calls
    let delta_tool_call = json!({
        "tool_calls": [{
            "id": "call_123",
            "function": {
                "name": "get_weather",
                "arguments": "{\"city\": \"Beijing\"}"
            }
        }]
    });
    assert!(delta_tool_call.get("tool_calls").is_some());
    assert!(delta_tool_call["tool_calls"].is_array());
}

// ============================================================================
// Test: test_empty_content_handling
// 输入: 空字符串的 content
// 预期: 正确处理空内容
// ============================================================================
#[test]
fn test_empty_content_handling() {
    let delta_empty_content = json!({
        "content": "",
        "role": "assistant"
    });

    let content = delta_empty_content["content"].as_str().unwrap();
    assert!(content.is_empty());

    // 空字符串应该被跳过，不应该触发 Content 事件
    // 这是 MessageStream 的行为
}

// ============================================================================
// Test: test_null_content_handling
// 输入: content 为 null
// 预期: 正确处理 null
// ============================================================================
#[test]
fn test_null_content_handling() {
    let delta_null_content = json!({
        "content": null,
        "role": "assistant"
    });

    // null 应该返回 None，而不是空字符串
    let content = delta_null_content.get("content").and_then(|c| c.as_str());
    assert!(content.is_none());
}

// ============================================================================
// Test: test_sse_with_usage_information
// 输入: 包含 usage 的 SSE chunk
// 预期: 正确提取 usage 信息
// ============================================================================
#[test]
fn test_sse_with_usage_information() {
    let chunk_with_usage = json!({
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "model": "test-model",
        "choices": [{
            "index": 0,
            "delta": {},
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 20,
            "total_tokens": 30
        }
    });

    // 提取 usage
    if let Some(usage) = chunk_with_usage.get("usage") {
        assert_eq!(usage["prompt_tokens"], 10);
        assert_eq!(usage["completion_tokens"], 20);
        assert_eq!(usage["total_tokens"], 30);
    }
}

// ============================================================================
// Test: test_finish_reason_variants
// 输入: 不同的 finish_reason
// 预期: 正确识别不同的结束原因
// ============================================================================
#[test]
fn test_finish_reason_variants() {
    let reasons = vec!["stop", "length", "tool_calls", "content_filter"];

    for reason in reasons {
        let delta = json!({
            "finish_reason": reason
        });

        let finish_reason = delta.get("finish_reason").and_then(|r| r.as_str());
        assert_eq!(finish_reason, Some(reason));
    }
}

// ============================================================================
// Test: test_invalid_json_in_sse
// 输入: SSE 中包含无效的 JSON
// 预期: 解析失败时应该优雅处理
// ============================================================================
#[test]
fn test_invalid_json_in_sse() {
    let invalid_sse = "data: {invalid json}\n\n";

    // 提取数据部分
    if let Some(data) = invalid_sse.strip_prefix("data: ") {
        let data = data.strip_suffix("\n\n").unwrap();

        // 尝试解析应该失败
        let result: Result<serde_json::Value, _> = serde_json::from_str(data);
        assert!(result.is_err());
    }
}

// ============================================================================
// Test: test_tool_call_delta_parsing
// 输入: 工具调用的增量数据
// 预期: 正确解析工具调用的各个部分
// ============================================================================
#[test]
fn test_tool_call_delta_parsing() {
    // 第一个 chunk：工具调用 ID 和名称
    let delta1 = json!({
        "tool_calls": [{
            "index": 0,
            "id": "call_abc123",
            "type": "function",
            "function": {
                "name": "get_weather",
                "arguments": ""
            }
        }]
    });

    let tool_calls = delta1["tool_calls"].as_array().unwrap();
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0]["id"], "call_abc123");
    assert_eq!(tool_calls[0]["function"]["name"], "get_weather");

    // 第二个 chunk：参数增量
    let delta2 = json!({
        "tool_calls": [{
            "index": 0,
            "function": {
                "arguments": "{\"city\":"
            }
        }]
    });

    let args = delta2["tool_calls"][0]["function"]["arguments"]
        .as_str()
        .unwrap();
    assert_eq!(args, "{\"city\":");

    // 第三个 chunk：参数增量继续
    let delta3 = json!({
        "tool_calls": [{
            "index": 0,
            "function": {
                "arguments": " \"Beijing\"}"
            }
        }]
    });

    let args = delta3["tool_calls"][0]["function"]["arguments"]
        .as_str()
        .unwrap();
    assert_eq!(args, " \"Beijing\"}");
}

// ============================================================================
// Test: test_sse_buffer_handling
// 输入: 模拟分块的 SSE 数据（模拟网络延迟）
// 预期: 正确处理分块到达的数据
// ============================================================================
#[test]
fn test_sse_buffer_handling() {
    // 模拟数据分块到达
    let chunk1 = "data: {\"test\":";
    let chunk2 = " \"value";
    let chunk3 = "\"}\n\n";

    // 模拟 buffer 累积
    let mut buffer = String::new();
    buffer.push_str(chunk1);
    buffer.push_str(chunk2);
    buffer.push_str(chunk3);

    // 检查是否可以完整解析
    assert!(buffer.contains("\n\n"));

    // 提取完整事件
    if let Some(pos) = buffer.find("\n\n") {
        let event = &buffer[..pos];
        assert!(event.starts_with("data: "));

        // 解析 JSON
        let json_str = event.strip_prefix("data: ").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(parsed["test"], "value");
    }
}

// ============================================================================
// Test: test_role_in_first_chunk
// 输入: 第一个 chunk 包含 role 字段
// 预期: 正确识别 assistant 角色
// ============================================================================
#[test]
fn test_role_in_first_chunk() {
    let first_chunk = json!({
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "model": "test-model",
        "choices": [{
            "index": 0,
            "delta": {
                "role": "assistant",
                "content": ""
            },
            "finish_reason": null
        }]
    });

    let delta = &first_chunk["choices"][0]["delta"];
    assert_eq!(delta["role"], "assistant");

    // 第一块通常 content 为空
    let content = delta.get("content").and_then(|c| c.as_str()).unwrap_or("");
    assert!(content.is_empty());
}
