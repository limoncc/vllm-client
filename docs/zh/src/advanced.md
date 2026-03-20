# 高级主题

本文档介绍 vLLM Client 的高级功能和用法。

## 目录

- [思考模式](#思考模式)
- [自定义请求头](#自定义请求头)
- [超时与重试](#超时与重试)
- [多模态支持](#多模态支持)

## 思考模式

某些模型（如 Qwen-3）支持"思考模式"，可以输出推理过程。

### 启用思考模式

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

let client = VllmClient::new("http://localhost:8000/v1");

let mut stream = client
    .chat
    .completions()
    .create()
    .model("qwen-3")
    .messages(json!([
        {"role": "user", "content": "请解释什么是递归"}
    ]))
    .extra(json!({
        "chat_template_kwargs": {
            "enable_thinking": true
        }
    }))
    .stream(true)
    .send_stream()
    .await?;

while let Some(event) = stream.next().await {
    match &event {
        // 思考/推理内容
        StreamEvent::Reasoning(delta) => {
            print!("[思考] {}", delta);
        }
        // 常规回复内容
        StreamEvent::Content(delta) => {
            print!("{}", delta);
        }
        _ => {}
    }
}
```

### 思考内容格式

在思考模式下，模型的输出分为两部分：

| 事件类型 | 描述 |
|---------|------|
| `StreamEvent::Reasoning` | 模型的推理/思考过程 |
| `StreamEvent::Content` | 最终的回复内容 |

思考内容通常包含在 `<think>` 标签中，客户端会自动解析。

### 禁用思考模式

```rust
.extra(json!({
    "chat_template_kwargs": {
        "enable_thinking": false
    }
}))
```

---

## 自定义请求头

如果需要添加自定义请求头（如代理认证、追踪ID等）：

```rust
use vllm_client::VllmClient;

let client = VllmClient::new("http://localhost:8000/v1")
    .with_header("X-Custom-Header", "custom-value")
    .with_header("X-Request-ID", "req-12345");
```

### 常见用例

```rust
// 添加代理认证
let client = VllmClient::new("http://localhost:8000/v1")
    .with_header("Proxy-Authorization", "Bearer proxy-token");

// 添加追踪ID用于调试
let client = VllmClient::new("http://localhost:8000/v1")
    .with_header("X-Trace-ID", &uuid::Uuid::new_v4().to_string());
```

---

## 超时与重试

### 设置超时

```rust
use std::time::Duration;
use vllm_client::VllmClient;

// 设置60秒超时
let client = VllmClient::new("http://localhost:8000/v1")
    .with_timeout(Duration::from_secs(60));

// 设置5分钟超时（适用于长文本生成）
let client = VllmClient::new("http://localhost:8000/v1")
    .with_timeout(Duration::from_secs(300));
```

### 实现重试逻辑

```rust
use vllm_client::{VllmClient, json, VllmError};
use std::time::Duration;
use tokio::time::sleep;

async fn send_with_retry(
    client: &VllmClient,
    messages: serde_json::Value,
    max_retries: u32,
) -> Result<vllm_client::ChatCompletionResponse, VllmError> {
    let mut attempts = 0;
    
    loop {
        match client
            .chat
            .completions()
            .create()
            .model("llama-3-70b")
            .messages(messages.clone())
            .send()
            .await
        {
            Ok(response) => return Ok(response),
            Err(e) => {
                attempts += 1;
                if attempts >= max_retries {
                    return Err(e);
                }
                // 指数退避
                sleep(Duration::from_millis(100 * 2u64.pow(attempts))).await;
            }
        }
    }
}
```

---

## 多模态支持

### 图像输入

```rust
use vllm_client::{VllmClient, json};

let client = VllmClient::new("http://localhost:8000/v1");

// 使用图像URL
let response = client
    .chat
    .completions()
    .create()
    .model("llava-v1.6")
    .messages(json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "这张图片里有什么？"},
                {
                    "type": "image_url",
                    "image_url": {
                        "url": "https://example.com/image.jpg"
                    }
                }
            ]
        }
    ]))
    .send()
    .await?;

// 使用Base64编码图像
let base64_image = "data:image/jpeg;base64,/9j/4AAQ...";
let response = client
    .chat
    .completions()
    .create()
    .model("llava-v1.6")
    .messages(json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "描述这张图片"},
                {
                    "type": "image_url",
                    "image_url": {"url": base64_image}
                }
            ]
        }
    ]))
    .send()
    .await?;
```

### 多图像支持

```rust
let response = client
    .chat
    .completions()
    .create()
    .model("llava-v1.6")
    .messages(json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "比较这两张图片"},
                {"type": "image_url", "image_url": {"url": "https://example.com/image1.jpg"}},
                {"type": "image_url", "image_url": {"url": "https://example.com/image2.jpg"}}
            ]
        }
    ]))
    .send()
    .await?;
```

---

## 最佳实践

### 1. 连接池管理

对于高并发场景，建议复用客户端实例：

```rust
// 推荐：共享客户端实例
use std::sync::Arc;

let client = Arc::new(VllmClient::new("http://localhost:8000/v1"));

// 在多个任务中使用
let client_clone = client.clone();
tokio::spawn(async move {
    client_clone.chat.completions().create()
        .model("llama-3")
        .messages(json!([{"role": "user", "content": "Hello"}]))
        .send()
        .await
});
```

### 2. 错误处理

```rust
use vllm_client::{VllmClient, VllmError};

match client.chat.completions().create().send().await {
    Ok(response) => {
        println!("成功: {:?}", response);
    }
    Err(VllmError::ApiError { message, code }) => {
        eprintln!("API 错误 ({}): {}", code, message);
        // 根据错误码处理
        match code {
            429 => println!("被限流，请稍后重试"),
            401 => println!("认证失败，检查API密钥"),
            _ => {}
        }
    }
    Err(e) => {
        eprintln!("其他错误: {}", e);
    }
}
```

### 3. 流式响应的资源管理

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

let mut stream = client
    .chat
    .completions()
    .create()
    .model("llama-3")
    .messages(json!([{"role": "user", "content": "Hello"}]))
    .stream(true)
    .send_stream()
    .await?;

// 使用 take 限制处理的消息数量
while let Some(event) = stream.take(1000).next().await {
    match &event {
        StreamEvent::Content(delta) => print!("{}", delta),
        StreamEvent::Done | StreamEvent::Error(_) => break,
        _ => {}
    }
}
```
