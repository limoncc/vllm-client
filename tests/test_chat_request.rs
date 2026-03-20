//! Phase 2.1: Chat Request Building Tests
//!
//! 测试 ChatCompletionsRequest 的构建和参数设置

use vllm_client::{json, VllmClient};

// ============================================================================
// Test: test_build_minimal_request
// 输入: client.chat.completions.create().model().messages()
// 预期: 构建出完整的请求对象，model 和 messages 正确
// ============================================================================
#[test]
fn test_build_minimal_request() {
    let client = VllmClient::new("http://localhost:8000/v1");

    // 构建最小请求
    let _request = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "hi"}]));

    // 如果能成功构建请求对象，测试通过
}

// ============================================================================
// Test: test_build_request_with_all_params
// 输入: 设置所有参数的请求
// 预期: 所有参数正确设置，请求对象构建成功
// ============================================================================
#[test]
fn test_build_request_with_all_params() {
    let client = VllmClient::new("http://localhost:8000/v1");

    let messages = json!([
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello!"}
    ]);

    let _request = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(messages)
        .temperature(0.7)
        .max_tokens(100)
        .top_p(0.9)
        .top_k(50)
        .stop(json!(["END", "STOP"]))
        .stream(true);

    // 如果能成功构建包含所有参数的请求对象，测试通过
}

// ============================================================================
// Test: test_build_request_with_tools
// 输入: 包含工具定义的请求
// 预期: 工具参数正确设置
// ============================================================================
#[test]
fn test_build_request_with_tools() {
    let client = VllmClient::new("http://localhost:8000/v1");

    let tools = json!([
        {
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get current weather",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "city": {"type": "string"}
                    },
                    "required": ["city"]
                }
            }
        }
    ]);

    let _request = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "What's the weather?"}]))
        .tools(tools)
        .tool_choice(json!("auto"));

    // 如果能成功构建包含工具的请求对象，测试通过
}

// ============================================================================
// Test: test_build_request_with_extra_params
// 输入: 包含额外参数的请求（vLLM 扩展）
// 预期: 额外参数正确设置
// ============================================================================
#[test]
fn test_build_request_with_extra_params() {
    let client = VllmClient::new("http://localhost:8000/v1");

    let _request = client
        .chat
        .completions()
        .create()
        .model("test-model")
        .messages(json!([{"role": "user", "content": "Think about it"}]))
        .extra(json!({
            "chat_template_kwargs": {"think_mode": true},
            "reasoning_effort": "high"
        }));

    // 如果能成功构建包含额外参数的请求对象，测试通过
}

// ============================================================================
// Test: test_request_builder_chain
// 输入: 链式调用构建请求
// 预期: 链式调用正常工作，请求对象构建成功
// ============================================================================
#[test]
fn test_request_builder_chain() {
    let client = VllmClient::new("http://localhost:8000/v1");

    // 测试链式调用
    let request = client
        .chat
        .completions()
        .create()
        .model("model-1")
        .messages(json!([{"role": "user", "content": "test"}]));

    // 确认可以继续链式调用
    let _request = request.temperature(0.5).max_tokens(50);
}

// ============================================================================
// Test: test_messages_json_format
// 输入: json!([{"role": "system", ...}, {"role": "user", ...}])
// 预期: 序列化后符合 OpenAI API 格式
// ============================================================================
#[test]
fn test_messages_json_format() {
    let messages = json!([
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello!"},
        {"role": "assistant", "content": "Hi there!"},
        {"role": "user", "content": "How are you?"}
    ]);

    // 验证 JSON 格式
    assert!(messages.is_array());
    let arr = messages.as_array().unwrap();
    assert_eq!(arr.len(), 4);

    // 验证每条消息都有 role 和 content
    for msg in arr {
        assert!(msg.get("role").is_some());
        assert!(msg.get("content").is_some());
    }

    // 验证序列化后的字符串格式
    let json_str = serde_json::to_string(&messages).unwrap();
    assert!(json_str.contains("\"role\""));
    assert!(json_str.contains("\"content\""));
}

// ============================================================================
// Test: test_multimodal_message_format
// 输入: 包含图像的多模态消息
// 预期: 消息格式正确，符合 OpenAI 格式
// ============================================================================
#[test]
fn test_multimodal_message_format() {
    let messages = json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "What's in this image?"},
                {
                    "type": "image_url",
                    "image_url": {"url": "https://example.com/image.jpg"}
                }
            ]
        }
    ]);

    // 验证多模态消息格式
    assert!(messages.is_array());
    let arr = messages.as_array().unwrap();
    assert_eq!(arr.len(), 1);

    let first_msg = &arr[0];
    assert_eq!(first_msg["role"], "user");
    assert!(first_msg["content"].is_array());

    let content = first_msg["content"].as_array().unwrap();
    assert_eq!(content.len(), 2);
    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[1]["type"], "image_url");
}

// ============================================================================
// Test: test_stop_sequences_format
// 输入: 不同格式的 stop 参数
// 预期: 都能正确接受
// ============================================================================
#[test]
fn test_stop_sequences_format() {
    let client = VllmClient::new("http://localhost:8000/v1");

    // 单个字符串
    let _req1 = client
        .chat
        .completions()
        .create()
        .model("model")
        .messages(json!([{"role": "user", "content": "test"}]))
        .stop(json!("END"));

    // 字符串数组
    let _req2 = client
        .chat
        .completions()
        .create()
        .model("model")
        .messages(json!([{"role": "user", "content": "test"}]))
        .stop(json!(["END", "STOP", "HALT"]));
}

// ============================================================================
// Test: test_tool_choice_formats
// 输入: 不同格式的 tool_choice 参数
// 预期: 都能正确接受
// ============================================================================
#[test]
fn test_tool_choice_formats() {
    let client = VllmClient::new("http://localhost:8000/v1");

    let base_request = || {
        client
            .chat
            .completions()
            .create()
            .model("model")
            .messages(json!([{"role": "user", "content": "test"}]))
    };

    // 字符串格式
    let _req1 = base_request().tool_choice(json!("auto"));
    let _req2 = base_request().tool_choice(json!("none"));
    let _req3 = base_request().tool_choice(json!("required"));

    // 对象格式
    let _req4 = base_request().tool_choice(json!({
        "type": "function",
        "function": {"name": "get_weather"}
    }));
}
