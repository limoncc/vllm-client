# Streaming Completions 示例

本示例演示如何使用旧版 `/v1/completions` API 进行流式调用。

## 基础流式 Completions

```rust
use vllm_client::{VllmClient, json, CompletionStreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    println!("=== 流式 Completions 示例 ===\n");
    println!("模型: Qwen/Qwen2.5-7B-Instruct\n");
    println!("提示词: 什么是机器学习？");
    println!("\n生成文本: ");

    let mut stream = client
        .completions
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .prompt("什么是机器学习？")
        .max_tokens(500)
        .temperature(0.7)
        .stream(true)
        .send_stream()
        .await?;

    // 处理流式事件
    while let Some(event) = stream.next().await {
        match event {
            CompletionStreamEvent::Text(delta) => {
                // 打印文本增量（实时输出）
                print!("{}", delta);
                // 刷新缓冲区，实现实时显示
                std::io::stdout().flush().ok();
            }
            CompletionStreamEvent::FinishReason(reason) => {
                println!("\n\n--- 结束原因: {} ---", reason);
            }
            CompletionStreamEvent::Usage(usage) => {
                // 流结束时输出 token 使用统计
                println!("\n\n--- Token 使用统计 ---");
                println!("提示词 tokens: {}", usage.prompt_tokens);
                println!("生成 tokens: {}", usage.completion_tokens);
                println!("总计 tokens: {}", usage.total_tokens);
            }
            CompletionStreamEvent::Done => {
                println!("\n\n=== 生成完成 ===");
                break;
            }
            CompletionStreamEvent::Error(e) => {
                eprintln!("\n错误: {}", e);
                return Err(e.into());
            }
        }
    }

    Ok(())
}
```

## 与 Chat 流式的区别

| 方面 | Chat Completions | Completions |
|--------|-----------------|-------------|
| 事件类型 | `StreamEvent` | `CompletionStreamEvent` |
| 内容变体 | `Content(String)` | `Text(String)` |
| 额外事件 | `Reasoning`, `ToolCall` | `FinishReason` |
| 适用场景 | 对话式 | 单提示词 |

## 何时使用 Completions API

- 简单的单提示词文本生成
- 与 OpenAI API 的旧版兼容
- 不需要聊天消息格式的场景

对于新项目，建议使用 Chat Completions API (`client.chat.completions()`)，它提供更灵活的功能和更好的消息格式。

## 相关链接

- [流式聊天](./streaming-chat.md) - 聊天流式示例
- [API 流式](../api/streaming.md) - 流式 API 参考
- [基础聊天](./basic-chat.md) - 非流式 completions 示例
