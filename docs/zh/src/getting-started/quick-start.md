# 快速上手

本节带你完成第一次 API 调用。

## 前置条件

- Rust 1.70 及以上版本
- 已启动的 vLLM 服务

## 基础对话补全

最简单的使用方式如下：

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端，指向 vLLM 服务地址
    let client = VllmClient::new("http://localhost:8000/v1");

    // 发送对话补全请求
    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "你好，最近怎么样？"}
        ]))
        .send()
        .await?;

    // 打印响应内容
    println!("回复: {}", response.content.unwrap_or_default());

    Ok(())
}
```

## 流式响应

如果需要实时输出，可以使用流式模式：

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    // 创建流式请求
    let mut stream = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "写一首关于春天的短诗"}
        ]))
        .stream(true)
        .send_stream()
        .await?;

    // 处理流式事件
    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Content(delta) => print!("{}", delta),
            StreamEvent::Reasoning(delta) => eprint!("[思考: {}]", delta),
            StreamEvent::Done => println!("\n[完成]"),
            StreamEvent::Error(e) => eprintln!("\n错误: {}", e),
            _ => {}
        }
    }

    Ok(())
}
```

## 使用构建器模式

需要更多配置时，可以使用构建器：

```rust
use vllm_client::VllmClient;

let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .api_key("your-api-key")  // 可选
    .timeout_secs(120)         // 可选
    .build();
```

## 完整示例

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "system", "content": "你是一个有帮助的助手。"},
            {"role": "user", "content": "法国的首都是哪里？"}
        ]))
        .temperature(0.7)
        .max_tokens(1024)
        .top_p(0.9)
        .send()
        .await?;

    println!("回复: {}", response.content.unwrap_or_default());
    
    // 打印 token 使用统计（如有）
    if let Some(usage) = response.usage {
        println!("Token 统计: 提示词={}, 补全={}, 总计={}",
            usage.prompt_tokens,
            usage.completion_tokens,
            usage.total_tokens
        );
    }

    Ok(())
}
```

## 错误处理

建议做好错误处理：

```rust
use vllm_client::{VllmClient, json, VllmError};

async fn chat() -> Result<String, VllmError> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "你好！"}
        ]))
        .send()
        .await?;

    Ok(response.content.unwrap_or_default())
}

#[tokio::main]
async fn main() {
    match chat().await {
        Ok(text) => println!("回复: {}", text),
        Err(VllmError::ApiError { status_code, message, .. }) => {
            eprintln!("API 错误 ({}): {}", status_code, message);
        }
        Err(VllmError::Timeout) => {
            eprintln!("请求超时");
        }
        Err(e) => {
            eprintln!("错误: {}", e);
        }
    }
}
```

## 下一步

- [配置说明](./configuration.md) - 了解全部配置选项
- [API 参考](../api.md) - 详细 API 文档
- [示例代码](../examples.md) - 更多使用示例