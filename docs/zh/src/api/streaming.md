# 流式响应 API

流式响应可以实时处理大语言模型的输出，逐个 token 接收，无需等待完整响应。

## 概述

vLLM Client 通过 Server-Sent Events (SSE) 提供流式支持。使用 `send_stream()` 替代 `send()` 即可获得流式响应。

## 基础流式调用

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
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "写一首关于春天的诗"}
        ]))
        .stream(true)
        .send_stream()
        .await?;

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Content(delta) => print!("{}", delta),
            StreamEvent::Done => break,
            _ => {}
        }
    }

    println!();
    Ok(())
}
```

## StreamEvent 类型

`StreamEvent` 枚举表示不同类型的流式事件：

| 变体 | 说明 |
|---------|-------------|
| `Content(String)` | 普通内容 token 增量 |
| `Reasoning(String)` | 推理/思考内容（思考模型） |
| `ToolCallDelta` | 流式工具调用增量 |
| `ToolCallComplete(ToolCall)` | 完整工具调用，可执行 |
| `Usage(Usage)` | Token 使用统计 |
| `Done` | 流式传输完成 |
| `Error(VllmError)` | 发生错误 |

### 内容事件

最常见的事件类型，包含文本 token：

```rust
match event {
    StreamEvent::Content(delta) => {
        print!("{}", delta);
        std::io::Write::flush(&mut std::io::stdout()).ok();
    }
    _ => {}
}
```

### 推理事件

用于带推理能力的模型（如开启思考模式的 Qwen）：

```rust
match event {
    StreamEvent::Reasoning(delta) => {
        eprintln!("[思考] {}", delta);
    }
    StreamEvent::Content(delta) => {
        print!("{}", delta);
    }
    _ => {}
}
```

### 工具调用事件

工具调用会先增量推送，完成后通知：

```rust
match event {
    StreamEvent::ToolCallDelta { index, id, name, arguments } => {
        println!("工具增量: index={}, name={}", index, name);
        // arguments 是部分 JSON 字符串
    }
    StreamEvent::ToolCallComplete(tool_call) => {
        println!("工具就绪: {}({})", tool_call.name, tool_call.arguments);
        // 执行工具并返回结果
    }
    _ => {}
}
```

### 使用统计事件

Token 使用信息通常在最后发送：

```rust
match event {
    StreamEvent::Usage(usage) => {
        println!("Tokens: 提示词={}, 补全={}, 总计={}",
            usage.prompt_tokens,
            usage.completion_tokens,
            usage.total_tokens
        );
    }
    _ => {}
}
```

## MessageStream

`MessageStream` 类型是一个异步迭代器，产出 `StreamEvent` 值。

### 方法

| 方法 | 返回类型 | 说明 |
|--------|-------------|------|
| `next()` | `Option<StreamEvent>` | 获取下一个事件（异步） |
| `collect_content()` | `String` | 收集所有内容为字符串 |
| `into_stream()` | `impl Stream` | 转换为通用流 |

### 收集全部内容

为方便使用，可以一次性收集所有内容：

```rust
let content = stream.collect_content().await?;
println!("完整响应: {}", content);
```

注意：这种方式会等待完整响应，失去了流式的意义。仅当需要同时显示流式输出和保存完整文本时使用。

## 完整流式示例

```rust
use vllm_client::{VllmClient, json, StreamEvent, VllmError};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), VllmError> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let mut stream = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "system", "content": "你是一个有帮助的助手。"},
            {"role": "user", "content": "用简单的语言解释量子计算"}
        ]))
        .temperature(0.7)
        .max_tokens(1024)
        .stream(true)
        .send_stream()
        .await?;

    let mut reasoning = String::new();
    let mut content = String::new();
    let mut usage = None;

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Reasoning(delta) => {
                reasoning.push_str(&delta);
            }
            StreamEvent::Content(delta) => {
                content.push_str(&delta);
                print!("{}", delta);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            StreamEvent::Usage(u) => {
                usage = Some(u);
            }
            StreamEvent::Done => {
                println!("\n[流式传输完成]");
            }
            StreamEvent::Error(e) => {
                eprintln!("\n错误: {}", e);
                return Err(e);
            }
            _ => {}
        }
    }

    // 打印摘要
    if !reasoning.is_empty() {
        eprintln!("\n--- 推理过程 ---");
        eprintln!("{}", reasoning);
    }

    if let Some(usage) = usage {
        eprintln!("\n--- Token 使用 ---");
        eprintln!("提示词: {}, 补全: {}, 总计: {}",
            usage.prompt_tokens,
            usage.completion_tokens,
            usage.total_tokens
        );
    }

    Ok(())
}
```

## 流式工具调用

使用工具时，工具调用会增量推送：

```rust
use vllm_client::{VllmClient, json, StreamEvent, ToolCall};
use futures::StreamExt;

let tools = json!([
    {
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "获取某地天气",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                },
                "required": ["location"]
            }
        }
    }
]);

let mut stream = client
    .chat
    .completions()
    .create()
    .model("Qwen/Qwen2.5-7B-Instruct")
    .messages(json!([
        {"role": "user", "content": "东京的天气怎么样？"}
    ]))
    .tools(tools)
    .stream(true)
    .send_stream()
    .await?;

let mut tool_calls: Vec<ToolCall> = Vec::new();

while let Some(event) = stream.next().await {
    match event {
        StreamEvent::Content(delta) => print!("{}", delta),
        StreamEvent::ToolCallComplete(tool_call) => {
            tool_calls.push(tool_call);
        }
        StreamEvent::Done => break,
        _ => {}
    }
}

// 执行工具调用
for tool_call in tool_calls {
    println!("工具: {} 参数: {}", tool_call.name, tool_call.arguments);
    // 执行并在下一条消息中返回结果
}
```

## 错误处理

流式传输过程中随时可能发生错误：

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
            StreamEvent::Error(e) => return Err(e),
            StreamEvent::Done => break,
            _ => {}
        }
    }

    Ok(content)
}
```

## 最佳实践

### 刷新输出

实时显示时，每次输出后刷新 stdout：

```rust
use std::io::{self, Write};

match event {
    StreamEvent::Content(delta) => {
        print!("{}", delta);
        io::stdout().flush().ok();
    }
    _ => {}
}
```

### 处理中断

交互式应用中，优雅地处理 Ctrl+C：

```rust
use tokio::signal;

tokio::select! {
    result = process_stream(&mut stream) => {
        // 正常完成
    }
    _ = signal::ctrl_c() => {
        println!("\n[已中断]");
    }
}
```

### 空闲流超时

为可能卡住的流设置超时：

```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(60),
    stream.next()
).await;

match result {
    Ok(Some(event)) => { /* 处理事件 */ }
    Ok(None) => { /* 流结束 */ }
    Err(_) => { /* 超时 */ }
}
```

## Completions 流式 API

vLLM Client 同时支持旧版 `/v1/completions` API 的流式调用，使用 `CompletionStreamEvent`。

### CompletionStreamEvent 类型

| 变体 | 说明 |
|---------|-------------|
| `Text(String)` | 文本 token 增量 |
| `FinishReason(String)` | 流结束原因（如 "stop", "length"） |
| `Usage(Usage)` | Token 使用统计 |
| `Done` | 流式传输完成 |
| `Error(VllmError)` | 发生错误 |

### Completions 流式示例

```rust
use vllm_client::{VllmClient, json, CompletionStreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let mut stream = client
        .completions
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .prompt("写一首关于春天的诗")
        .max_tokens(1024)
        .temperature(0.7)
        .stream(true)
        .send_stream()
        .await?;

    while let Some(event) = stream.next().await {
        match event {
            CompletionStreamEvent::Text(delta) => {
                print!("{}", delta);
                std::io::stdout().flush().ok();
            }
            CompletionStreamEvent::FinishReason(reason) => {
                println!("\n[结束原因: {}]", reason);
            }
            CompletionStreamEvent::Usage(usage) => {
                println!("\nTokens: 提示词={}, 补全={}, 总计={}",
                    usage.prompt_tokens,
                    usage.completion_tokens,
                    usage.total_tokens
                );
            }
            CompletionStreamEvent::Done => {
                println!("\n[流式传输完成]");
            }
            CompletionStreamEvent::Error(e) => {
                eprintln!("错误: {}", e);
                return Err(e.into());
            }
        }
    }

    Ok(())
}
```

### CompletionStream 方法

| 方法 | 返回类型 | 说明 |
|--------|-------------|------|
| `next()` | `Option<CompletionStreamEvent>` | 获取下一个事件（异步） |
| `collect_text()` | `String` | 收集所有文本为字符串 |
| `into_stream()` | `impl Stream` | 转换为通用流 |

## 相关链接

- [工具调用](./tool-calling.md) - 使用函数调用
- [错误处理](./error-handling.md) - 完整错误处理指南
- [示例代码](../examples/streaming-chat.md) - 更多流式示例