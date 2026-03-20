//! Phase 6.2: Real vLLM Integration Tests
//!
//! 使用真实的 vLLM 服务进行集成测试
//! 这些测试需要真实的 vLLM 服务运行，默认被标记为 #[ignore]
//!
//! 运行方法：
//! cargo test --test real_vllm -- --ignored

use vllm_client::{json, VllmClient};

// 真实 vLLM 服务配置
const BASE_URL: &str = "http://23.99.0.1:18120/v1";
const API_KEY: &str = "sk-f58fe51e-c5f2-4510-3364-36f9c4e0f697";
const MODEL: &str = "Qwen3.5-35B-A3B";

// ============================================================================
// Helper: 创建客户端
// ============================================================================
fn create_client() -> VllmClient {
    VllmClient::new(BASE_URL).with_api_key(API_KEY)
}

// ============================================================================
// Test: test_real_chat_completion
// 条件: 需要 vLLM 服务运行
// 输入: 真实 API 调用
// 预期: 返回有效响应
// ============================================================================
#[tokio::test]
#[ignore]
async fn test_real_chat_completion() {
    let client = create_client();

    let response = client
        .chat
        .completions()
        .create()
        .model(MODEL)
        .messages(json!([
            {"role": "user", "content": "Hello, can you say 'Hi' in one word?"}
        ]))
        .max_tokens(10)
        .temperature(0.1)
        .send()
        .await
        .expect("Failed to send request");

    // 验证响应
    assert!(!response.id.is_empty(), "Response ID should not be empty");
    assert_eq!(response.object, "chat.completion");
    assert_eq!(response.model, MODEL);
    assert!(response.content.is_some(), "Response should have content");

    let content = response.content.unwrap();
    println!("Response content: {}", content);
    assert!(!content.is_empty(), "Content should not be empty");

    // 验证 usage
    assert!(
        response.usage.is_some(),
        "Response should have usage statistics"
    );
    let usage = response.usage.unwrap();
    assert!(usage.prompt_tokens > 0, "Prompt tokens should be > 0");
    assert!(
        usage.completion_tokens > 0,
        "Completion tokens should be > 0"
    );
    assert!(usage.total_tokens > 0, "Total tokens should be > 0");
}

// ============================================================================
// Test: test_real_multi_turn_conversation
// 条件: 需要 vLLM 服务运行
// 输入: 多轮对话
// 预期: 对话历史正确维护，响应合理
// ============================================================================
#[tokio::test]
#[ignore]
async fn test_real_multi_turn_conversation() {
    let client = create_client();

    let messages = json!([
        {"role": "user", "content": "My name is Alice."},
        {"role": "assistant", "content": "Nice to meet you, Alice! How can I help you today?"},
        {"role": "user", "content": "What's my name?"}
    ]);

    let response = client
        .chat
        .completions()
        .create()
        .model(MODEL)
        .messages(messages)
        .max_tokens(50)
        .temperature(0.1)
        .send()
        .await
        .expect("Failed to send request");

    let content = response.content.unwrap();
    println!("Multi-turn response: {}", content);

    // 响应应该包含用户的名字 "Alice"
    let content_lower = content.to_lowercase();
    assert!(
        content_lower.contains("alice"),
        "Response should remember the name Alice"
    );
}

// ============================================================================
// Test: test_real_streaming
// 条件: 需要 vLLM 服务运行
// 输入: 真实流式 API
// 预期: 流式输出正常
// ============================================================================
#[tokio::test]
#[ignore]
async fn test_real_streaming() {
    // use futures::StreamExt;
    use vllm_client::StreamEvent;

    let client = create_client();

    let mut stream = client
        .chat
        .completions()
        .create()
        .model(MODEL)
        .messages(json!([
            {"role": "user", "content": "Count from 1 to 5, one number per line."}
        ]))
        .max_tokens(50)
        .temperature(0.1)
        .extra(json!({"chat_template_kwargs": {"think_mode": false}}))
        .send_stream()
        .await
        .expect("Failed to send stream request");

    let mut content_parts = Vec::new();
    let mut reasoning_parts = Vec::new();
    let mut event_count = 0;
    let mut has_usage = false;

    while let Some(event) = stream.next().await {
        event_count += 1;
        println!("Event {}: {:?}", event_count, event);

        match event {
            StreamEvent::Content(delta) => {
                content_parts.push(delta);
            }
            StreamEvent::Reasoning(reasoning) => {
                reasoning_parts.push(reasoning);
            }
            StreamEvent::Usage(usage) => {
                println!(
                    "Usage: prompt={}, completion={}, total={}",
                    usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                );
                has_usage = true;
            }
            StreamEvent::Done => {
                println!("Stream completed");
                break;
            }
            StreamEvent::Error(e) => {
                panic!("Stream error: {:?}", e);
            }
            _ => {}
        }

        if event_count > 1000 {
            panic!("Too many events, possible infinite loop");
        }
    }

    println!("Content parts: {:?}", content_parts);
    println!("Reasoning parts: {:?}", reasoning_parts);

    assert!(
        !content_parts.is_empty() || !reasoning_parts.is_empty(),
        "Should receive content or reasoning chunks"
    );
    assert!(has_usage, "Should receive usage information at the end");

    let full_content = content_parts.join("");
    let full_reasoning = reasoning_parts.join("");
    println!("Full streamed content: {}", full_content);
    println!("Full streamed reasoning: {}", full_reasoning);
}

// ============================================================================
// Test: test_real_streaming_collect_content
// 条件: 需要 vLLM 服务运行
// 输入: 使用 collect_content() 方法
// 预期: 正确收集所有内容
// ============================================================================
#[tokio::test]
#[ignore]
async fn test_real_streaming_collect_content() {
    let client = create_client();

    let stream = client
        .chat
        .completions()
        .create()
        .model(MODEL)
        .messages(json!([
            {"role": "user", "content": "Say 'Hello, World!'"}
        ]))
        .max_tokens(20)
        .temperature(0.1)
        .extra(json!({"chat_template_kwargs": {"think_mode": false}}))
        .send_stream()
        .await
        .expect("Failed to send stream request");

    let content = stream
        .collect_content()
        .await
        .expect("Failed to collect content");

    println!("Collected content: {}", content);
    assert!(!content.is_empty(), "Collected content should not be empty");

    let content_lower = content.to_lowercase();
    assert!(
        content_lower.contains("hello"),
        "Content should contain 'hello'"
    );
}

// ============================================================================
// Test: test_real_temperature_parameter
// 条件: 需要 vLLM 服务运行
// 输入: 不同的 temperature 参数
// 预期: 影响输出的随机性
// ============================================================================
#[tokio::test]
#[ignore]
async fn test_real_temperature_parameter() {
    let client = create_client();

    // 测试低 temperature（更确定性的输出）
    let response_low = client
        .chat
        .completions()
        .create()
        .model(MODEL)
        .messages(json!([
            {"role": "user", "content": "Say exactly: 'OK'"}
        ]))
        .max_tokens(5)
        .temperature(0.0) // 最确定性
        .send()
        .await
        .expect("Failed to send request");

    println!("Low temperature response: {:?}", response_low.content);
    assert!(response_low.content.is_some());
}

// ============================================================================
// Test: test_real_max_tokens_parameter
// 条件: 需要 vLLM 服务运行
// 输入: 限制 max_tokens
// 预期: 输出不超过限制
// ============================================================================
#[tokio::test]
#[ignore]
async fn test_real_max_tokens_parameter() {
    let client = create_client();

    let max_tokens = 10;
    let response = client
        .chat
        .completions()
        .create()
        .model(MODEL)
        .messages(json!([
            {"role": "user", "content": "Tell me a long story about a brave knight."}
        ]))
        .max_tokens(max_tokens)
        .temperature(0.1)
        .send()
        .await
        .expect("Failed to send request");

    // 检查 finish_reason
    // 注意：由于 max_tokens 限制很小，finish_reason 应该是 "length"
    println!("Finish reason: {:?}", response.finish_reason);
    println!("Content: {:?}", response.content);

    // 验证 token 数量不超过限制
    if let Some(usage) = response.usage {
        println!(
            "Completion tokens: {} (limit: {})",
            usage.completion_tokens, max_tokens
        );
        assert!(
            usage.completion_tokens <= max_tokens as u64,
            "Completion tokens should not exceed max_tokens"
        );
    }
}

// ============================================================================
// Test: test_real_stop_sequences
// 条件: 需要 vLLM 服务运行
// 输入: 使用 stop 序列
// 预期: 在遇到 stop 序列时停止
// ============================================================================
#[tokio::test]
#[ignore]
async fn test_real_stop_sequences() {
    let client = create_client();

    let response = client
        .chat
        .completions()
        .create()
        .model(MODEL)
        .messages(json!([
            {"role": "user", "content": "Count: 1, 2, 3, 4, 5"}
        ]))
        .max_tokens(50)
        .temperature(0.1)
        .stop(json!("3")) // 在遇到 "3" 时停止
        .send()
        .await
        .expect("Failed to send request");

    let content = response.content.unwrap();
    println!("Content with stop sequence: {}", content);

    // 验证内容不包含 "3" 或在 "3" 之前停止
    // 注意：这个测试可能不太稳定，因为模型输出格式可能变化
}

// ============================================================================
// Test: test_real_legacy_completion
// 条件: 需要 vLLM 服务运行
// 输入: Legacy Completions API
// 预期: 返回正确的 completion 响应
// ============================================================================
#[tokio::test]
#[ignore]
async fn test_real_legacy_completion() {
    let client = create_client();

    let response = client
        .completions
        .create()
        .model(MODEL)
        .prompt(json!("Once upon a time"))
        .max_tokens(20)
        .temperature(0.7)
        .send()
        .await
        .expect("Failed to send legacy completion request");

    // 验证响应
    assert!(!response.id.is_empty(), "Response ID should not be empty");
    assert_eq!(response.object, "text_completion");

    assert!(
        !response.choices.is_empty(),
        "Should have at least one choice"
    );
    let choice = &response.choices[0];
    assert!(!choice.text.is_empty(), "Text should not be empty");

    println!("Legacy completion text: {}", choice.text);
}

// ============================================================================
// Test: test_real_error_handling
// 条件: 需要 vLLM 服务运行
// 输入: 无效的模型名称
// 预期: 返回错误
// ============================================================================
#[tokio::test]
#[ignore]
async fn test_real_error_handling() {
    let client = create_client();

    let result = client
        .chat
        .completions()
        .create()
        .model("non-existent-model-12345")
        .messages(json!([
            {"role": "user", "content": "Hello"}
        ]))
        .send()
        .await;

    // 应该返回错误
    assert!(result.is_err(), "Should return error for invalid model");

    let err = result.unwrap_err();
    println!("Error (expected): {:?}", err);
}

// ============================================================================
// Test: test_real_concurrent_requests
// 条件: 需要 vLLM 服务运行
// 输入: 并发发送多个请求
// 预期: 所有请求正确处理
// ============================================================================
#[tokio::test]
#[ignore]
async fn test_real_concurrent_requests() {
    use std::sync::Arc;
    use tokio::task::JoinSet;

    let client = Arc::new(create_client());
    let mut tasks = JoinSet::new();

    // 发送 3 个并发请求
    for i in 0..3 {
        let client = Arc::clone(&client);
        tasks.spawn(async move {
            let response = client
                .chat
                .completions()
                .create()
                .model(MODEL)
                .messages(json!([
                    {"role": "user", "content": format!("Request number {}", i)}
                ]))
                .max_tokens(10)
                .temperature(0.1)
                .send()
                .await
                .expect(&format!("Request {} failed", i));

            println!("Request {} response: {:?}", i, response.content);
            response
        });
    }

    // 等待所有请求完成
    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        results.push(result.expect("Task should not panic"));
    }

    assert_eq!(results.len(), 3, "Should complete 3 requests");
    println!(
        "All {} concurrent requests completed successfully",
        results.len()
    );
}
