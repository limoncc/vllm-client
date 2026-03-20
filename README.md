# VLLM Client

一个 Rust 客户端库，用于对接 vLLM 推理引擎的 OpenAI 兼容 API。

## 设计理念

- **Python 兼容**：API 风格对齐 openai-python，降低迁移成本
- **灵活优先**：输入输出均支持 `serde_json::Value`，最大化灵活性
- **最小抽象**：不做过度封装，让用户直接操作 JSON
- **便捷辅助**：提供解析辅助方法，但不强制使用

## 特性

- ✅ Chat Completions API (`/v1/chat/completions`)
- ✅ Legacy Completions API (`/v1/completions`)
- ✅ 流式响应 (SSE)
- ✅ 工具调用 (Function Calling)
- ✅ 多模态支持（图像输入）
- ✅ 思考模式（vLLM 推理模型扩展）

## 安装

```toml
[dependencies]
vllm-client = "0.1"
```

## 快速开始

### 简单对话

```rust
use vllm_client::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let response = client.chat.completions.create()
        .model("Qwen/Qwen2.5-72B-Instruct")
        .messages(json!([
            {"role": "user", "content": "你好，介绍一下自己"}
        ]))
        .temperature(0.7)
        .max_tokens(512)
        .send()
        .await?;
    
    println!("{}", response.content.unwrap());
    Ok(())
}
```

### 流式输出

```rust
let mut stream = client.chat.completions.create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "写一首诗"}]))
    .stream(true)
    .send_stream()
    .await?;

while let Some(event) = stream.next().await {
    if let StreamEvent::Content(delta) = event {
        print!("{}", delta);
    }
}
```

### 工具调用

```rust
let response = client.chat.completions.create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "北京天气？"}]))
    .tools(json!([
        {
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "获取天气",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "city": {"type": "string"}
                    },
                    "required": ["city"]
                }
            }
        }
    ]))
    .send()
    .await?;

if response.has_tool_calls() {
    for call in &response.tool_calls {
        let args: serde_json::Value = call.parse_args()?;
        let result = execute_tool(&call.name, args);
        
        // 构造工具结果消息
        let tool_message = call.result(json!({"temp": 25}));
    }
}
```

## API 风格

```rust
// 对齐 openai-python 的链式调用
client.chat.completions.create()
    .model("model-name")
    .messages(json!([...]))
    .temperature(0.7)
    .max_tokens(1024)
    .tools(json!([...]))
    .stream(true)
    .send()
    .await?
```

## 文档

- [API 设计文档](../api设计文档)
- [TDD 开发计划](./plan.md)

## License

MIT OR Apache-2.0