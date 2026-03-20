# 客户端 API

`VllmClient` 是使用 vLLM API 的主要入口。

## 创建客户端

### 简单创建

```rust
use vllm_client::VllmClient;

let client = VllmClient::new("http://localhost:8000/v1");
```

### 带 API Key

```rust
use vllm_client::VllmClient;

let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key("sk-your-api-key");
```

### 设置超时

```rust
use vllm_client::VllmClient;

let client = VllmClient::new("http://localhost:8000/v1")
    .timeout_secs(120); // 2 分钟
```

### 使用构建器模式

复杂配置可以用构建器：

```rust
use vllm_client::VllmClient;

let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .api_key("sk-your-api-key")
    .timeout_secs(300)
    .build();
```

## 方法参考

### `new(base_url: impl Into<String>) -> Self`

用指定的 base URL 创建客户端。

```rust
let client = VllmClient::new("http://localhost:8000/v1");
```

**参数：**
- `base_url` - vLLM 服务的基础 URL（需包含 `/v1` 路径）

**注意：**
- 末尾斜杠会自动移除
- 客户端创建开销很小，但仍建议复用

### `with_api_key(self, api_key: impl Into<String>) -> Self`

设置 API Key（构建器模式）。

```rust
let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key("sk-xxx");
```

**参数：**
- `api_key` - 用于 Bearer 认证的 API Key

**注意：**
- API Key 会作为 Bearer Token 放在 `Authorization` 请求头中
- 此方法返回新的客户端实例

### `timeout_secs(self, secs: u64) -> Self`

设置请求超时时间（构建器模式）。

```rust
let client = VllmClient::new("http://localhost:8000/v1")
    .timeout_secs(300);
```

**参数：**
- `secs` - 超时时间（秒）

**注意：**
- 对该客户端发起的所有请求生效
- 长时间生成任务建议调大超时时间

### `base_url(&self) -> &str`

获取客户端的 base URL。

```rust
let client = VllmClient::new("http://localhost:8000/v1");
assert_eq!(client.base_url(), "http://localhost:8000/v1");
```

### `api_key(&self) -> Option<&str>`

获取已配置的 API Key。

```rust
let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key("sk-xxx");
assert_eq!(client.api_key(), Some("sk-xxx"));
```

### `builder() -> VllmClientBuilder`

创建新的客户端构建器，支持更多配置选项。

```rust
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .api_key("sk-xxx")
    .timeout_secs(120)
    .build();
```

## API 模块

客户端提供多个 API 模块：

### `chat` - 对话补全 API

访问对话补全接口：

```rust
let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "你好！"}]))
    .send()
    .await?;
```

### `completions` - 传统补全 API

访问传统文本补全接口：

```rust
let response = client.completions.create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .prompt("从前有座山")
    .send()
    .await?;
```

## VllmClientBuilder

构建器提供灵活的客户端配置方式。

### 方法

| 方法 | 类型 | 说明 |
|--------|------|------|
| `base_url(url)` | `impl Into<String>` | 设置基础 URL |
| `api_key(key)` | `impl Into<String>` | 设置 API Key |
| `timeout_secs(secs)` | `u64` | 设置超时时间（秒） |
| `build()` | - | 构建客户端 |

### 默认值

| 选项 | 默认值 |
|--------|---------|
| `base_url` | `http://localhost:8000/v1` |
| `api_key` | `None` |
| `timeout_secs` | HTTP 客户端默认值（30秒） |

## 使用示例

### 基础用法

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let response = client.chat.completions().create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "你好！"}
        ]))
        .send()
        .await?;
    
    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

### 使用环境变量

```rust
use std::env;
use vllm_client::VllmClient;

fn create_client() -> VllmClient {
    let base_url = env::var("VLLM_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8000/v1".to_string());
    
    let api_key = env::var("VLLM_API_KEY").ok();
    
    let mut builder = VllmClient::builder().base_url(&base_url);
    
    if let Some(key) = api_key {
        builder = builder.api_key(&key);
    }
    
    builder.build()
}
```

### 多次请求

复用客户端处理多次请求：

```rust
use vllm_client::{VllmClient, json};

async fn process_prompts(client: &VllmClient, prompts: &[&str]) -> Vec<String> {
    let mut results = Vec::new();
    
    for prompt in prompts {
        let response = client.chat.completions().create()
            .model("Qwen/Qwen2.5-7B-Instruct")
            .messages(json!([{"role": "user", "content": prompt}]))
            .send()
            .await;
        
        match response {
            Ok(r) => results.push(r.content.unwrap_or_default()),
            Err(e) => eprintln!("错误: {}", e),
        }
    }
    
    results
}
```

## 线程安全

`VllmClient` 是线程安全的，可以跨线程共享：

```rust
use std::sync::Arc;
use vllm_client::VllmClient;

let client = Arc::new(VllmClient::new("http://localhost:8000/v1"));

// 可以克隆并在多线程间传递
let client_clone = Arc::clone(&client);
```

## 相关链接

- [对话补全](./chat-completions.md) - 对话补全 API
- [流式响应](./streaming.md) - 流式响应处理
- [配置说明](../getting-started/configuration.md) - 配置选项