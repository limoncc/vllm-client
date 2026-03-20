# 错误处理

本文档介绍 vLLM Client 中的错误处理机制。

## VllmError 枚举

vLLM Client 中的所有错误都通过 `VllmError` 枚举表示：

```rust
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum VllmError {
    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("JSON error: {0}")]
    Json(String),

    #[error("API error (status {status_code}): {message}")]
    ApiError {
        status_code: u16,
        message: String,
        error_type: Option<String>,
    },

    #[error("Stream error: {0}")]
    Stream(String),

    #[error("Connection timeout")]
    Timeout,

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("No response content")]
    NoContent,

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("{0}")]
    Other(String),
}
```

## 错误类型

| 变体 | 发生场景 |
|------|----------|
| `Http` | 网络错误、连接失败 |
| `Json` | 序列化/反序列化错误 |
| `ApiError` | 服务器返回错误响应 |
| `Stream` | 流式响应过程中的错误 |
| `Timeout` | 请求超时 |
| `ModelNotFound` | 指定的模型不存在 |
| `MissingParameter` | 缺少必需参数 |
| `NoContent` | 响应无内容 |
| `InvalidResponse` | 响应格式不符合预期 |
| `Other` | 其他错误 |

## 基础错误处理

```rust
use vllm_client::{VllmClient, json, VllmError};

async fn chat(prompt: &str) -> Result<String, VllmError> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": prompt}]))
        .send()
        .await?;

    Ok(response.content.unwrap_or_default())
}

#[tokio::main]
async fn main() {
    match chat("你好！").await {
        Ok(text) => println!("响应: {}", text),
        Err(e) => eprintln!("错误: {}", e),
    }
}
```

## 详细错误处理

针对不同错误类型进行不同处理：

```rust
use vllm_client::{VllmClient, json, VllmError};

#[tokio::main]
async fn main() {
    let client = VllmClient::new("http://localhost:8000/v1");

    let result = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": "你好！"}]))
        .send()
        .await;

    match result {
        Ok(response) => {
            println!("成功: {}", response.content.unwrap_or_default());
        }
        Err(VllmError::ApiError { status_code, message, error_type }) => {
            eprintln!("API 错误 (HTTP {}): {}", status_code, message);
            if let Some(etype) = error_type {
                eprintln!("错误类型: {}", etype);
            }
        }
        Err(VllmError::Timeout) => {
            eprintln!("请求超时，请尝试增加超时时间。");
        }
        Err(VllmError::Http(msg)) => {
            eprintln!("网络错误: {}", msg);
        }
        Err(VllmError::ModelNotFound(model)) => {
            eprintln!("模型 '{}' 未找到，请检查可用模型。", model);
        }
        Err(VllmError::MissingParameter(param)) => {
            eprintln!("缺少必需参数: {}", param);
        }
        Err(e) => {
            eprintln!("其他错误: {}", e);
        }
    }
}
```

## HTTP 状态码

常见的 API 错误状态码：

| 状态码 | 含义 | 处理建议 |
|--------|------|----------|
| 400 | 请求格式错误 | 检查请求参数 |
| 401 | 未授权 | 检查 API Key |
| 403 | 禁止访问 | 检查权限 |
| 404 | 未找到 | 检查端点或模型名称 |
| 429 | 请求频率限制 | 实现退避重试 |
| 500 | 服务器内部错误 | 重试或联系管理员 |
| 502 | 网关错误 | 检查 vLLM 服务器状态 |
| 503 | 服务不可用 | 等待后重试 |
| 504 | 网关超时 | 增加超时时间或重试 |

## 可重试错误

检查错误是否可重试：

```rust
use vllm_client::VllmError;

fn should_retry(error: &VllmError) -> bool {
    error.is_retryable()
}

// 手动检查
match error {
    VllmError::Timeout => true,
    VllmError::ApiError { status_code: 429, .. } => true,  // 频率限制
    VllmError::ApiError { status_code: 500..=504, .. } => true,  // 服务器错误
    _ => false,
}
```

## 指数退避重试

```rust
use vllm_client::{VllmClient, json, VllmError};
use std::time::Duration;
use tokio::time::sleep;

async fn chat_with_retry(
    client: &VllmClient,
    prompt: &str,
    max_retries: u32,
) -> Result<String, VllmError> {
    let mut retries = 0;

    loop {
        let result = client
            .chat
            .completions()
            .create()
            .model("Qwen/Qwen2.5-7B-Instruct")
            .messages(json!([{"role": "user", "content": prompt}]))
            .send()
            .await;

        match result {
            Ok(response) => {
                return Ok(response.content.unwrap_or_default());
            }
            Err(e) if e.is_retryable() && retries < max_retries => {
                retries += 1;
                let delay = Duration::from_millis(100 * 2u64.pow(retries - 1));
                eprintln!("第 {} 次重试，等待 {:?}: {}", retries, delay, e);
                sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## 流式响应错误处理

处理流式响应过程中的错误：

```rust
use vllm_client::{VllmClient, json, StreamEvent, VllmError};
use futures::StreamExt;

async fn stream_chat(prompt: &str) -> Result<String, VllmError> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let mut stream = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": prompt}]))
        .stream(true)
        .send_stream()
        .await?;

    let mut content = String::new();

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Content(delta) => content.push_str(&delta),
            StreamEvent::Done => break,
            StreamEvent::Error(e) => return Err(e),
            _ => {}
        }
    }

    Ok(content)
}
```

## 错误上下文

为错误添加上下文信息，便于调试：

```rust
use vllm_client::{VllmClient, json, VllmError};

async fn chat_with_context(prompt: &str) -> Result<String, String> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": prompt}]))
        .send()
        .await
        .map_err(|e| format!("获取对话响应失败: {}", e))?;

    Ok(response.content.unwrap_or_default())
}
```

## 使用 anyhow 或 eyre

对于使用 `anyhow` 或 `eyre` 的应用程序：

```rust
use vllm_client::{VllmClient, json, VllmError};
use anyhow::{Context, Result};

async fn chat(prompt: &str) -> Result<String> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": prompt}]))
        .send()
        .await
        .context("发送对话请求失败")?;

    Ok(response.content.unwrap_or_default())
}
```

## 最佳实践

### 1. 始终处理错误

```rust
// 不好的做法
let response = client.chat.completions().create()
    .send().await.unwrap();

// 好的做法
match client.chat.completions().create().send().await {
    Ok(r) => { /* 处理响应 */ },
    Err(e) => eprintln!("错误: {}", e),
}
```

### 2. 设置适当的超时时间

```rust
let client = VllmClient::new("http://localhost:8000/v1")
    .timeout_secs(300); // 长时间任务设置为 5 分钟
```

### 3. 记录带上下文的错误

```rust
Err(e) => {
    log::error!("对话请求失败: {}", e);
    log::debug!("请求详情: model={}, prompt_len={}", model, prompt.len());
}
```

### 4. 实现优雅降级

```rust
match primary_client.chat.completions().create().send().await {
    Ok(r) => r,
    Err(e) => {
        log::warn!("主客户端失败: {}, 尝试备用客户端", e);
        fallback_client.chat.completions().create().send().await?
    }
}
```

## 相关链接

- [客户端](./client.md) - 客户端配置
- [流式响应](./streaming.md) - 流式响应错误处理
- [超时与重试](../advanced/timeouts.md) - 高级超时配置