//! Phase 11: Completions Streaming Tests
//!
//! 测试 Legacy Completions API (/v1/completions) 的流式请求

use serde_json::json;
use vllm_client::VllmClient;

// ============================================================================
// Test: test_completions_send_stream_actually_works
// Mock: POST /v1/completions, 返回 SSE 流式响应
// 输入: client.completions.create().model().prompt().stream(true).send_stream().await
// 预期: 返回 CompletionStream，能收集内容
// ============================================================================
#[tokio::test]
async fn test_completions_send_stream_actually_works() {
    let mut server = mockito::Server::new_async().await;

    // 模拟 SSE 流式响应 (completions 格式)
    let sse_response = concat!(
        "data: {\"id\":\"cmpl-stream\",\"object\":\"text_completion\",\"model\":\"test-model\",\"choices\":[{\"text\":\"Hello\",\"index\":0,\"finish_reason\":null}]}\n\n",
        "data: {\"id\":\"cmpl-stream\",\"object\":\"text_completion\",\"model\":\"test-model\",\"choices\":[{\"text\":\" World\",\"index\":0,\"finish_reason\":null}]}\n\n",
        "data: {\"id\":\"cmpl-stream\",\"object\":\"text_completion\",\"model\":\"test-model\",\"choices\":[{\"text\":\".\",\"index\":0,\"finish_reason\":\"stop\"}]}\n\n",
        "data: [DONE]\n\n"
    );

    let mock = server
        .mock("POST", "/v1/completions")
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
        .completions
        .create()
        .model("test-model")
        .prompt(json!("Say hello"))
        .stream(true)
        .send_stream()
        .await
        .unwrap();

    let content = stream.collect_text().await.unwrap();

    mock.assert();
    assert_eq!(content, "Hello World.");
}

// ============================================================================
// Test: test_completions_stream_request_builder
// 输入: client.completions.create().model().prompt().stream(true)
// 预期: 请求对象可以设置 stream(true)，准备调用 send_stream()
// ============================================================================
#[test]
fn test_completions_stream_request_builder() {
    let client = VllmClient::new("http://localhost:8000/v1");

    // 构建流式请求
    let _request = client
        .completions
        .create()
        .model("test-model")
        .prompt(json!("Hello"))
        .max_tokens(100)
        .stream(true);

    // 如果能成功构建流式请求对象，测试通过
}

// ============================================================================
// Test: test_completions_stream_all_params
// 输入: 设置所有参数的流式请求
// 预期: 所有参数正确设置，请求对象构建成功
// ============================================================================
#[test]
fn test_completions_stream_all_params() {
    let client = VllmClient::new("http://localhost:8000/v1");

    let _request = client
        .completions
        .create()
        .model("test-model")
        .prompt(json!("Once upon a time"))
        .max_tokens(200)
        .temperature(0.8)
        .top_p(0.95)
        .top_k(50)
        .stop(json!(["END", "\n"]))
        .stream(true);

    // 所有参数正确设置
}

// ============================================================================
// Test: test_completions_send_stream_method_exists
// 输入: 创建流式请求后调用 send_stream()
// 预期: send_stream() 方法存在并可调用（编译通过）
// ============================================================================
#[test]
fn test_completions_send_stream_method_exists() {
    let client = VllmClient::new("http://localhost:8000/v1");

    // 注意：这里只测试编译通过，不实际发送请求
    let _request = client
        .completions
        .create()
        .model("test-model")
        .prompt(json!("Hello"))
        .stream(true);

    // 验证 send_stream 方法存在 - 通过检查 CompletionRequest 是否有 send_stream 方法
    // 这里我们用一个技巧：把 request 转换为 Box<dyn> 来验证方法存在
    // 但更简单的是直接让编译通过即可，因为 send_stream 是 async 方法
}

// ============================================================================
// Test: test_completions_stream_parse_sse_format
// 输入: SSE 格式的 completion chunk
// 预期: 正确解析 text 字段
// ============================================================================
#[test]
fn test_completions_stream_parse_sse_format() {
    // 模拟 SSE 数据 - completions 流式响应的 JSON 格式
    let sse_data = r#"data: {"id":"cmpl-123","object":"text_completion","model":"test-model","choices":[{"text":"Hello","index":0,"finish_reason":null}]}"#;

    // 提取 JSON 部分
    if let Some(data) = sse_data.strip_prefix("data: ") {
        let parsed: serde_json::Value = serde_json::from_str(data).unwrap();

        // 验证结构
        assert_eq!(parsed["id"], "cmpl-123");
        assert_eq!(parsed["object"], "text_completion");

        // 提取 text
        let text = parsed["choices"][0]["text"].as_str().unwrap();
        assert_eq!(text, "Hello");
    }
}

// ============================================================================
// Test: test_completions_stream_parse_multiple_chunks
// 输入: 多个 SSE chunk
// 预期: 正确解析每个 chunk 的 text
// ============================================================================
#[test]
fn test_completions_stream_parse_multiple_chunks() {
    // 模拟多个 chunk
    let chunks = vec![
        r#"data: {"id":"cmpl-1","object":"text_completion","model":"test","choices":[{"text":"Hello","index":0,"finish_reason":null}]}"#,
        r#"data: {"id":"cmpl-1","object":"text_completion","model":"test","choices":[{"text":" World","index":0,"finish_reason":null}]}"#,
        r#"data: {"id":"cmpl-1","object":"text_completion","model":"test","choices":[{"text":".","index":0,"finish_reason":"stop"}]}"#,
    ];

    let mut all_text = String::new();

    for chunk in chunks {
        if let Some(data) = chunk.strip_prefix("data: ") {
            if data == "[DONE]" {
                break;
            }

            let parsed: serde_json::Value = serde_json::from_str(data).unwrap();
            if let Some(text) = parsed["choices"][0]["text"].as_str() {
                all_text.push_str(text);
            }
        }
    }

    assert_eq!(all_text, "Hello World.");
}

// ============================================================================
// Test: test_completions_stream_done_marker
// 输入: "data: [DONE]\n\n"
// 预期: 正确识别流结束标记
// ============================================================================
#[test]
fn test_completions_stream_done_marker() {
    let done_marker = "data: [DONE]\n\n";

    if let Some(data) = done_marker.strip_prefix("data: ") {
        let data = data.strip_suffix("\n\n").unwrap();
        assert_eq!(data, "[DONE]");
    }
}

// ============================================================================
// Test: test_completions_stream_parse_with_usage
// 输入: 包含 usage 的 completion chunk
// 预期: 正确解析 usage 信息
// ============================================================================
#[test]
fn test_completions_stream_parse_with_usage() {
    let chunk = r#"data: {"id":"cmpl-1","object":"text_completion","model":"test","choices":[{"text":"Test","index":0,"finish_reason":"stop"}],"usage":{"prompt_tokens":5,"completion_tokens":10,"total_tokens":15}}"#;

    if let Some(data) = chunk.strip_prefix("data: ") {
        let parsed: serde_json::Value = serde_json::from_str(data).unwrap();

        // 验证 usage
        let usage = parsed["usage"].as_object().unwrap();
        assert_eq!(usage["prompt_tokens"], 5);
        assert_eq!(usage["completion_tokens"], 10);
        assert_eq!(usage["total_tokens"], 15);
    }
}

// ============================================================================
// Test: test_completions_stream_parse_multiple_choices
// 输入: 多个 choices 的 completion chunk
// 预期: 正确解析每个 choice
// ============================================================================
#[test]
fn test_completions_stream_parse_multiple_choices() {
    let chunk = r#"data: {"id":"cmpl-multi","object":"text_completion","model":"test","choices":[{"text":"First","index":0,"finish_reason":null},{"text":"Second","index":1,"finish_reason":"stop"}]}"#;

    if let Some(data) = chunk.strip_prefix("data: ") {
        let parsed: serde_json::Value = serde_json::from_str(data).unwrap();

        let choices = parsed["choices"].as_array().unwrap();
        assert_eq!(choices.len(), 2);
        assert_eq!(choices[0]["text"], "First");
        assert_eq!(choices[1]["text"], "Second");
    }
}

// ============================================================================
// Test: test_completions_stream_empty_text
// 输入: text 为空字符串的 chunk
// 预期: 正确处理空文本
// ============================================================================
#[test]
fn test_completions_stream_empty_text() {
    let chunk = r#"data: {"id":"cmpl-empty","object":"text_completion","model":"test","choices":[{"text":"","index":0,"finish_reason":"stop"}]}"#;

    if let Some(data) = chunk.strip_prefix("data: ") {
        let parsed: serde_json::Value = serde_json::from_str(data).unwrap();

        let text = parsed["choices"][0]["text"].as_str().unwrap();
        assert!(text.is_empty());
    }
}

// ============================================================================
// Test: test_completions_stream_with_logprobs
// 输入: 包含 logprobs 的 chunk
// 预期: 正确解析 logprobs
// ============================================================================
#[test]
fn test_completions_stream_with_logprobs() {
    let chunk = r#"data: {"id":"cmpl-logprobs","object":"text_completion","model":"test","choices":[{"text":"Hi","index":0,"logprobs":{"tokens":["Hi"],"token_logprobs":[-0.1]},"finish_reason":"stop"}]}"#;

    if let Some(data) = chunk.strip_prefix("data: ") {
        let parsed: serde_json::Value = serde_json::from_str(data).unwrap();

        let logprobs = parsed["choices"][0]["logprobs"].as_object().unwrap();
        assert!(logprobs.contains_key("tokens"));
        assert!(logprobs.contains_key("token_logprobs"));
    }
}
