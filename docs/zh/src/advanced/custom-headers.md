# 自定义请求头

本文档介绍如何在 vLLM Client 中使用自定义 HTTP 请求头。

## 概述

虽然 vLLM Client 通过 API Key 处理标准认证，但您可能需要添加自定义请求头用于：
- 自定义认证方案
- 请求追踪和调试
- 速率限制标识符
- 自定义元数据

## 当前限制

当前版本的 vLLM Client 不提供内置的自定义请求头方法。但是，您可以通过几种方式解决这个限制。

## 变通方法：环境变量

如果您的 vLLM 服务器通过环境变量或特定 API 参数接受配置：

```rust
use vllm_client::{VllmClient, json};

let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key(std::env::var("MY_API_KEY").unwrap_or_default());
```

## 变通方法：通过额外参数

一些自定义配置可以通过 `extra()` 方法传递：

```rust
use vllm_client::{VllmClient, json};

let response = client
    .chat
    .completions()
    .create()
    .model("Qwen/Qwen2.5-7B-Instruct")
    .messages(json!([{"role": "user", "content": "你好！"}]))
    .extra(json!({
        "custom_field": "custom_value",
        "request_id": "req-12345"
    }))
    .send()
    .await?;
```

## 未来支持

自定义请求头支持计划在未来版本中实现。API 可能类似于：

```rust,ignore
// 未来 API（尚未实现）
let client = VllmClient::new("http://localhost:8000/v1")
    .with_header("X-Custom-Header", "value")
    .with_header("X-Request-ID", "req-123");
```

## 常见使用案例

### 追踪请求头

用于分布式追踪（当支持时）：

```rust,ignore
// 未来 API
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .header("X-Trace-ID", trace_id)
    .header("X-Span-ID", span_id)
    .build();
```

### 自定义认证

用于非标准认证方案：

```rust,ignore
// 未来 API
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .header("X-API-Key", "custom-key")
    .header("X-Tenant-ID", "tenant-123")
    .build();
```

### 请求元数据

添加元数据用于日志或分析：

```rust,ignore
// 未来 API
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .header("X-Request-Source", "mobile-app")
    .header("X-User-ID", "user-456")
    .build();
```

## 替代方案：自定义 HTTP 客户端

对于高级用例，您可以直接使用底层的 `reqwest` 客户端：

```rust
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let response = client
        .post("http://localhost:8000/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer your-api-key")
        .header("X-Custom-Header", "custom-value")
        .json(&json!({
            "model": "Qwen/Qwen2.5-7B-Instruct",
            "messages": [{"role": "user", "content": "你好！"}]
        }))
        .send()
        .await?;
    
    let result: serde_json::Value = response.json().await?;
    println!("{:?}", result);
    
    Ok(())
}
```

## 最佳实践

### 1. 尽可能使用标准认证

```rust
// 推荐
let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key("your-api-key");

// 除非必要，避免使用自定义认证
```

### 2. 文档化自定义请求头

使用自定义请求头时，记录其用途：

```rust,ignore
// 未来 API
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    // 用于日志中的请求追踪
    .header("X-Request-ID", &request_id)
    // 用于多租户标识
    .header("X-Tenant-ID", &tenant_id)
    .build();
```

### 3. 验证服务器支持

确保您的 vLLM 服务器接受并处理自定义请求头。一些代理或负载均衡器可能会移除未知的请求头。

## 安全考虑

### 不要暴露敏感请求头

避免记录包含敏感信息的请求头：

```rust,ignore
// 记录日志时要小心
let auth_header = "Bearer secret-key";
// 不要直接记录这个！
```

### 使用 HTTPS

传输敏感请求头时始终使用 HTTPS：

```rust
// 好
let client = VllmClient::new("https://api.example.com/v1");

// 对于敏感数据避免使用
let client = VllmClient::new("http://api.example.com/v1");
```

## 请求此功能

如果您需要自定义请求头支持，请在 GitHub 上提交 issue，包括：
1. 您的使用场景
2. 需要的请求头
3. 您希望 API 如何设计

## 相关链接

- [超时与重试](./timeouts.md) - 配置超时和重试逻辑
- [思考模式](./thinking-mode.md) - 推理模型支持
- [客户端 API](../api/client.md) - 客户端配置选项