# vLLM Client

一个兼容 OpenAI 接口的 vLLM API Rust 客户端库。

## 特性

- **OpenAI 兼容**：使用与 OpenAI 相同的 API 结构，方便迁移
- **流式响应**：完整支持 Server-Sent Events (SSE) 流式响应
- **工具调用**：支持函数/工具调用，支持流式增量更新
- **推理模型**：内置支持推理/思考模型（如启用了思考模式的 Qwen）
- **异步支持**：基于 Tokio 运行时的完全异步实现
- **类型安全**：使用 Serde 序列化的强类型定义

## 快速开始

添加到你的 `Cargo.toml`：

```toml
[dependencies]
vllm-client = "0.1"
tokio = { version = "1", features = ["full"] }
```

## 基本用法

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let response = client
        .chat
        .completions()
        .create()
        .model("your-model-name")
        .messages(json!([
            {"role": "user", "content": "你好，世界！"}
        ]))
        .send()
        .await?;
    
    println!("{}", response.choices[0].message.content);
    Ok(())
}
```

## 文档

- [快速开始](./getting-started.md) - 安装和基本配置
- [API 参考](./api.md) - 完整的 API 文档
- [示例代码](./examples.md) - 代码示例
- [高级主题](./advanced/streaming.md) - 流式响应、工具调用等

## 语言

- [English](../) - English documentation
- **中文** - 当前页面

## 许可证

根据 Apache 许可证 2.0 版本或 MIT 许可证任选其一授权。