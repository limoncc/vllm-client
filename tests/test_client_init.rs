//! Phase 1.1: 客户端初始化测试
//!
//! 测试 VllmClient 的创建和基本配置

// ============================================================================
// Test: test_create_client_with_base_url
// 输入: VllmClient::new("http://localhost:8000/v1")
// 预期: 客户端创建成功，base_url 正确设置
// ============================================================================
#[test]
fn test_create_client_with_base_url() {
    let client = vllm_client::VllmClient::new("http://localhost:8000/v1");

    assert_eq!(client.base_url(), "http://localhost:8000/v1");
}

// ============================================================================
// Test: test_create_client_with_api_key
// 输入: VllmClient::new(url).api_key("sk-test")
// 预期: 客户端创建成功，api_key 正确设置
// ============================================================================
#[test]
fn test_create_client_with_api_key() {
    let client =
        vllm_client::VllmClient::new("http://localhost:8000/v1").with_api_key("sk-test-12345");

    assert_eq!(client.api_key(), Some("sk-test-12345"));
}

// ============================================================================
// Test: test_create_client_with_api_key_builder
// 输入: VllmClient::builder().base_url(...).api_key(...).build()
// 预期: 使用 builder 模式创建客户端
// ============================================================================
#[test]
fn test_create_client_with_api_key_builder() {
    let client = vllm_client::VllmClient::builder()
        .base_url("http://localhost:8000/v1")
        .api_key("sk-builder-test")
        .build();

    assert_eq!(client.base_url(), "http://localhost:8000/v1");
    assert_eq!(client.api_key(), Some("sk-builder-test"));
}

// ============================================================================
// Test: test_client_has_chat_module
// 输入: client.chat
// 预期: 返回 Chat 模块实例
// ============================================================================
#[test]
fn test_client_has_chat_module() {
    let client = vllm_client::VllmClient::new("http://localhost:8000/v1");

    // chat 模块应该存在
    let _chat = &client.chat;
}

// ============================================================================
// Test: test_client_has_completions_module
// 输入: client.completions
// 预期: 返回 Completions 模块实例
// ============================================================================
#[test]
fn test_client_has_completions_module() {
    let client = vllm_client::VllmClient::new("http://localhost:8000/v1");

    // completions 模块应该存在
    let _completions = &client.completions;
}

// ============================================================================
// Test: test_client_timeout_configuration
// 输入: VllmClient::new(url).timeout(60)
// 预期: 客户端创建成功，超时设置正确
// ============================================================================
#[test]
fn test_client_timeout_configuration() {
    let client = vllm_client::VllmClient::new("http://localhost:8000/v1").timeout_secs(60);

    // 验证客户端可以正常创建
    assert_eq!(client.base_url(), "http://localhost:8000/v1");
}

// ============================================================================
// Test: test_client_without_api_key
// 输入: VllmClient::new(url) 无 api_key
// 预期: api_key() 返回 None
// ============================================================================
#[test]
fn test_client_without_api_key() {
    let client = vllm_client::VllmClient::new("http://localhost:8000/v1");

    assert_eq!(client.api_key(), None);
}

// ============================================================================
// Test: test_create_client_with_api_key_fluent
// 输入: VllmClient::new(url).with_api_key("sk-test")
// 预期: 客户端创建成功，api_key 正确设置（流畅 API）
// ============================================================================
#[test]
fn test_create_client_with_api_key_fluent() {
    let client = vllm_client::VllmClient::new("http://localhost:8000/v1")
        .with_api_key("sk-fluent-test")
        .timeout_secs(60);

    assert_eq!(client.api_key(), Some("sk-fluent-test"));
}

// ============================================================================
// Test: test_client_base_url_trailing_slash_removed
// 输入: VllmClient::new("http://localhost:8000/v1/")
// 预期: 自动移除尾部斜杠
// ============================================================================
#[test]
fn test_client_base_url_trailing_slash_removed() {
    let client = vllm_client::VllmClient::new("http://localhost:8000/v1/");

    // 应该自动移除尾部斜杠
    assert_eq!(client.base_url(), "http://localhost:8000/v1");
}

// ============================================================================
// Test: test_client_default_configuration
// 输入: VllmClient::default() 或无参数创建
// 预期: 使用默认配置
// ============================================================================
#[test]
fn test_client_default_configuration() {
    let client = vllm_client::VllmClient::default();

    // 默认 base_url 应该是某个合理的默认值（如 localhost）
    // 或者要求必须提供 base_url
    // 这里我们选择要求必须提供，所以 default() 可能返回 localhost
    assert!(client.base_url().contains("localhost") || client.base_url().contains("127.0.0.1"));
}
