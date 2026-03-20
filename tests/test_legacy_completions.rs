//! Phase 5.3: Legacy Completions Tests
//!
//! 测试 Legacy Completions API (/v1/completions)

use serde_json::json;
use vllm_client::{CompletionResponse, VllmClient};

// ============================================================================
// Test: test_completions_request_building
// 输入: client.completions.create().model().prompt().max_tokens()
// 预期: 请求对象构建成功
// ============================================================================
#[test]
fn test_completions_request_building() {
    let client = VllmClient::new("http://localhost:8000/v1");

    // 构建最小请求
    let _request = client
        .completions
        .create()
        .model("test-model")
        .prompt(json!("Hello"))
        .max_tokens(10);

    // 如果能成功构建请求对象，测试通过
}

// ============================================================================
// Test: test_completions_request_with_all_params
// 输入: 设置所有参数的请求
// 预期: 所有参数正确设置，请求对象构建成功
// ============================================================================
#[test]
fn test_completions_request_with_all_params() {
    let client = VllmClient::new("http://localhost:8000/v1");

    let _request = client
        .completions
        .create()
        .model("test-model")
        .prompt(json!("Once upon a time"))
        .max_tokens(100)
        .temperature(0.8)
        .top_p(0.95)
        .top_k(50)
        .stop(json!(["END", "\n"]))
        .stream(false);

    // 如果能成功构建包含所有参数的请求对象，测试通过
}

// ============================================================================
// Test: test_completions_response_format
// 输入: Mock 响应 JSON
// 预期: response.choices[0].text 正确
// ============================================================================
#[test]
fn test_completions_response_format() {
    let raw = json!({
        "id": "cmpl-123",
        "object": "text_completion",
        "model": "test-model",
        "choices": [
            {
                "text": "This is a generated text.",
                "index": 0,
                "logprobs": null,
                "finish_reason": "length"
            }
        ],
        "usage": {
            "prompt_tokens": 5,
            "completion_tokens": 10,
            "total_tokens": 15
        }
    });

    let response = CompletionResponse::from_raw(raw).unwrap();

    assert_eq!(response.id, "cmpl-123");
    assert_eq!(response.object, "text_completion");
    assert_eq!(response.model, "test-model");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].text, "This is a generated text.");
    assert_eq!(response.choices[0].index, 0);
    assert_eq!(
        response.choices[0].finish_reason,
        Some("length".to_string())
    );

    let usage = response.usage.as_ref().unwrap();
    assert_eq!(usage.prompt_tokens, 5);
    assert_eq!(usage.completion_tokens, 10);
    assert_eq!(usage.total_tokens, 15);
}

// ============================================================================
// Test: test_completions_multiple_choices
// 输入: 包含多个 choices 的响应
// 预期: 所有 choices 正确解析
// ============================================================================
#[test]
fn test_completions_multiple_choices() {
    let raw = json!({
        "id": "cmpl-multi",
        "object": "text_completion",
        "model": "test-model",
        "choices": [
            {
                "text": "First completion.",
                "index": 0,
                "finish_reason": "stop"
            },
            {
                "text": "Second completion.",
                "index": 1,
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 5,
            "completion_tokens": 20,
            "total_tokens": 25
        }
    });

    let response = CompletionResponse::from_raw(raw).unwrap();

    assert_eq!(response.choices.len(), 2);
    assert_eq!(response.choices[0].text, "First completion.");
    assert_eq!(response.choices[1].text, "Second completion.");
}

// ============================================================================
// Test: test_completions_prompt_formats
// 输入: 不同格式的 prompt 参数
// 预期: 都能正确接受
// ============================================================================
#[test]
fn test_completions_prompt_formats() {
    let client = VllmClient::new("http://localhost:8000/v1");

    // 单个字符串
    let _req1 = client
        .completions
        .create()
        .model("model")
        .prompt(json!("Hello"));

    // 字符串数组
    let _req2 = client
        .completions
        .create()
        .model("model")
        .prompt(json!(["Hello", "Hi", "Hey"]));

    // 使用 json! 宏
    let _req3 = client
        .completions
        .create()
        .model("model")
        .prompt(json!("Tell me a story"));
}

// ============================================================================
// Test: test_completions_stop_sequences
// 输入: 不同格式的 stop 参数
// 预期: 都能正确接受
// ============================================================================
#[test]
fn test_completions_stop_sequences() {
    let client = VllmClient::new("http://localhost:8000/v1");

    let base_request = || {
        client
            .completions
            .create()
            .model("model")
            .prompt(json!("test"))
    };

    // 单个字符串
    let _req1 = base_request().stop(json!("END"));

    // 字符串数组
    let _req2 = base_request().stop(json!(["END", "STOP", "\n\n"]));
}

// ============================================================================
// Test: test_completions_finish_reasons
// 输入: 不同 finish_reason 的响应
// 预期: 都能正确解析
// ============================================================================
#[test]
fn test_completions_finish_reasons() {
    let finish_reasons = vec!["stop", "length", "content_filter"];

    for reason in finish_reasons {
        let raw = json!({
            "id": "cmpl-finish",
            "object": "text_completion",
            "model": "test-model",
            "choices": [{
                "text": "Test text",
                "index": 0,
                "finish_reason": reason
            }],
            "usage": {
                "prompt_tokens": 5,
                "completion_tokens": 5,
                "total_tokens": 10
            }
        });

        let response = CompletionResponse::from_raw(raw).unwrap();
        assert_eq!(response.choices[0].finish_reason, Some(reason.to_string()));
    }
}

// ============================================================================
// Test: test_completions_raw_preserved
// 输入: 任意响应
// 预期: response.raw 保留原始 JSON，可访问任意字段
// ============================================================================
#[test]
fn test_completions_raw_preserved() {
    let raw = json!({
        "id": "cmpl-raw",
        "object": "text_completion",
        "model": "test-model",
        "choices": [{
            "text": "Test",
            "index": 0,
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 5,
            "completion_tokens": 5,
            "total_tokens": 10
        },
        "custom_field": "custom_value"
    });

    let response = CompletionResponse::from_raw(raw.clone()).unwrap();

    // 原始 JSON 被保留
    assert_eq!(response.raw, raw);

    // 可以访问自定义字段
    assert_eq!(response.raw["custom_field"], "custom_value");
}

// ============================================================================
// Test: test_completions_optional_usage
// 输入: 不包含 usage 的响应
// 预期: response.usage == None，程序不崩溃
// ============================================================================
#[test]
fn test_completions_optional_usage() {
    let raw = json!({
        "id": "cmpl-no-usage",
        "object": "text_completion",
        "model": "test-model",
        "choices": [{
            "text": "Test",
            "index": 0,
            "finish_reason": "stop"
        }]
    });

    let response = CompletionResponse::from_raw(raw).unwrap();
    assert_eq!(response.usage, None);
}

// ============================================================================
// Test: test_completions_logprobs
// 输入: 包含 logprobs 的响应
// 预期: logprobs 字段正确保存
// ============================================================================
#[test]
fn test_completions_logprobs() {
    let raw = json!({
        "id": "cmpl-logprobs",
        "object": "text_completion",
        "model": "test-model",
        "choices": [{
            "text": "Test",
            "index": 0,
            "logprobs": {
                "tokens": ["Test"],
                "token_logprobs": [-0.5],
                "top_logprobs": [{"Test": -0.5}]
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 5,
            "completion_tokens": 1,
            "total_tokens": 6
        }
    });

    let response = CompletionResponse::from_raw(raw).unwrap();

    // logprobs 字段应该保存
    assert!(response.choices[0].logprobs.is_some());

    let logprobs = response.choices[0].logprobs.as_ref().unwrap();
    assert!(logprobs.is_object());
}

// ============================================================================
// Test: test_completions_builder_chain
// 输入: 链式调用构建请求
// 预期: 链式调用正常工作
// ============================================================================
#[test]
fn test_completions_builder_chain() {
    let client = VllmClient::new("http://localhost:8000/v1");

    // 测试链式调用
    let request = client
        .completions
        .create()
        .model("model-1")
        .prompt(json!("test"));

    // 确认可以继续链式调用
    let _request = request.temperature(0.5).max_tokens(50).top_p(0.9);
}

// ============================================================================
// Test: test_completions_empty_text
// 输入: text 为空字符串的响应
// 预期: 正确解析，程序不崩溃
// ============================================================================
#[test]
fn test_completions_empty_text() {
    let raw = json!({
        "id": "cmpl-empty",
        "object": "text_completion",
        "model": "test-model",
        "choices": [{
            "text": "",
            "index": 0,
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 5,
            "completion_tokens": 0,
            "total_tokens": 5
        }
    });

    let response = CompletionResponse::from_raw(raw).unwrap();
    assert_eq!(response.choices[0].text, "");
}

// ============================================================================
// Test: test_completions_request_builder_pattern
// 输入: 使用 builder 模式创建请求
// 预期: builder 模式正常工作
// ============================================================================
#[test]
fn test_completions_request_builder_pattern() {
    let client = VllmClient::new("http://localhost:8000/v1");

    // 测试分步构建
    let request_builder = client.completions.create();
    let request_with_model = request_builder.model("test-model");
    let request_with_prompt = request_with_model.prompt(json!("Hello"));
    let _final_request = request_with_prompt.max_tokens(10);

    // 测试链式构建
    let _request = client
        .completions
        .create()
        .model("test-model")
        .prompt(json!("Hello"))
        .max_tokens(10)
        .temperature(0.7);
}
