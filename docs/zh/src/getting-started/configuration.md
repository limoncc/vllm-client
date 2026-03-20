# 配置说明

本文档介绍 `vllm-client` 的全部配置选项。

## 客户端配置

### 基础配置

```rust
use vllm_client::VllmClient;

let client = VllmClient::new("http://localhost:8000/v1");
```

### 构建器模式

需要更复杂的配置时，使用构建器模式：

```rust
use vllm_client::VllmClient;

let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .api_key("your-api-key")
    .timeout_secs(120)
    .build();
```

## 配置选项

### Base URL

vLLM 服务的地址，需要包含 `/v1` 路径以兼容 OpenAI 接口。

```rust
// 本地开发
let client = VllmClient::new("http://localhost:8000/v1");

// 远程服务
let client = VllmClient::new("https://api.example.com/v1");

// 末尾斜杠会自动处理
let client = VllmClient::new("http://localhost:8000/v1/");
// 等同于: "http://localhost:8000/v1"
```

### API Key

如果 vLLM 服务需要认证，配置 API Key：

```rust
// 链式调用
let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key("sk-your-api-key");

// 构建器模式
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .api_key("sk-your-api-key")
    .build();
```

API Key 会作为 Bearer Token 放在 `Authorization` 请求头中发送。

### 超时设置

长时间运行的任务需要调大超时时间：

```rust
// 链式调用
let client = VllmClient::new("http://localhost:8000/v1")
    .timeout_secs(300); // 5 分钟

// 构建器模式
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .timeout_secs(300)
    .build();
```

默认使用底层 HTTP 客户端的超时设置（通常为 30 秒）。

## 请求参数配置

发起请求时，可以配置以下参数：

### 模型选择

```rust
use vllm_client::{VllmClient, json};

let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "你好！"}]))
    .send()
    .await?;
```

### 采样参数

```rust
let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "你好！"}]))
    .temperature(0.7)      // 0.0 - 2.0
    .top_p(0.9)            // 0.0 - 1.0
    .top_k(50)             // vLLM 扩展参数
    .max_tokens(1024)      // 最大输出 token 数
    .send()
    .await?;
```

| 参数 | 类型 | 范围 | 说明 |
|-----------|------|-------|-------------|
| `temperature` | f32 | 0.0 - 2.0 | 控制随机性，值越高输出越随机 |
| `top_p` | f32 | 0.0 - 1.0 | 核采样阈值 |
| `top_k` | i32 | 1+ | Top-K 采样（vLLM 扩展） |
| `max_tokens` | u32 | 1+ | 最大生成 token 数 |

### 停止序列

```rust
use serde_json::json;

// 多个停止序列
let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "你好！"}]))
    .stop(json!(["END", "STOP", "\n\n"]))
    .send()
    .await?;

// 单个停止序列
let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "你好！"}]))
    .stop(json!("END"))
    .send()
    .await?;
```

### 扩展参数

vLLM 支持通过 `extra()` 方法传入额外参数：

```rust
let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "请思考这个问题"}]))
    .extra(json!({
        "chat_template_kwargs": {
            "think_mode": true
        },
        "reasoning_effort": "high"
    }))
    .send()
    .await?;
```

## 环境变量

可以通过环境变量配置客户端：

```rust
use std::env;
use vllm_client::VllmClient;

let base_url = env::var("VLLM_BASE_URL")
    .unwrap_or_else(|_| "http://localhost:8000/v1".to_string());

let api_key = env::var("VLLM_API_KEY").ok();

let mut client_builder = VllmClient::builder()
    .base_url(&base_url);

if let Some(key) = api_key {
    client_builder = client_builder.api_key(&key);
}

let client = client_builder.build();
```

### 常用环境变量

| 变量名 | 说明 | 示例 |
|----------|-------------|---------|
| `VLLM_BASE_URL` | vLLM 服务地址 | `http://localhost:8000/v1` |
| `VLLM_API_KEY` | API Key（可选） | `sk-xxx` |
| `VLLM_TIMEOUT` | 超时时间（秒） | `300` |

## 最佳实践

### 复用客户端

客户端应该创建一次、多次复用：

```rust
// 推荐：复用客户端
let client = VllmClient::new("http://localhost:8000/v1");

for prompt in prompts {
    let response = client.chat.completions().create()
        .model("Qwen/Qwen2.5-72B-Instruct")
        .messages(json!([{"role": "user", "content": prompt}]))
        .send()
        .await?;
}

// 避免：每次请求都创建客户端
for prompt in prompts {
    let client = VllmClient::new("http://localhost:8000/v1"); // 效率低！
    // ...
}
```

### 超时时间选择

根据使用场景选择合适的超时时间：

| 使用场景 | 建议超时 |
|----------|---------------------|
| 简单问答 | 30 秒 |
| 复杂推理 | 2-5 分钟 |
| 长文本生成 | 10 分钟以上 |

### 错误处理

务必正确处理错误：

```rust
use vllm_client::{VllmClient, VllmError};

match client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "你好！"}]))
    .send()
    .await
{
    Ok(response) => println!("{}", response.content.unwrap()),
    Err(VllmError::Timeout) => eprintln!("请求超时"),
    Err(VllmError::ApiError { status_code, message, .. }) => {
        eprintln!("API 错误 ({}): {}", status_code, message);
    }
    Err(e) => eprintln!("错误: {}", e),
}
```

## 下一步

- [快速上手](./quick-start.md) - 基本用法示例
- [API 参考](../api.md) - 完整 API 文档
- [错误处理](../api/error-handling.md) - 详细错误处理指南