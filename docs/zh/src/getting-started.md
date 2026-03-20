# 快速开始

## 安装

将 `vllm-client` 添加到你的 `Cargo.toml`：

```toml
[dependencies]
vllm-client = "0.1"
tokio = { version = "1", features = ["full"] }
```

## 快速开始

### 基础聊天补全

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端
    let client = VllmClient::new("http://localhost:8000/v1");
    
    // 发送聊天补全请求
    let response = client
        .chat
        .completions()
        .create()
        .model("your-model-name")
        .messages(json!([
            {"role": "user", "content": "你好，你好吗？"}
        ]))
        .send()
        .await?;
    
    // 打印响应
    println!("{}", response.choices[0].message.content);
    
    Ok(())
}
```

### 流式响应

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let mut stream = client
        .chat
        .completions()
        .create()
        .model("your-model-name")
        .messages(json!([
            {"role": "user", "content": "写一首关于春天的诗"}
        ]))
        .stream(true)
        .send_stream()
        .await?;
    
    while let Some(event) = stream.next().await {
        match &event {
            StreamEvent::Reasoning(delta) => print!("{}", delta),
            StreamEvent::Content(delta) => print!("{}", delta),
            _ => {}
        }
    }
    
    println!();
    Ok(())
}
```

## 配置

### API 密钥

如果你的 vLLM 服务器需要认证：

```rust
let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key("your-api-key");
```

### 自定义超时

```rust
let client = VllmClient::new("http://localhost:8000/v1")
    .with_timeout(std::time::Duration::from_secs(60));
```

## 下一步

- [API 参考](./api.md) - 完整的 API 文档
- [示例](./examples.md) - 更多使用示例
- [高级功能](./advanced/thinking.md) - 思考模式、工具调用等