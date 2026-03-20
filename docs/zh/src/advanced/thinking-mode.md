# 思考模式

思考模式（也称为推理模式）允许模型在给出最终答案之前输出其推理过程。这对于复杂推理任务特别有用。

## 概述

一些模型，如启用思考模式的 Qwen，可以输出两种类型的内容：

1. **推理内容** - 模型的内部"思考"过程
2. **内容** - 给用户的最终响应

## 启用思考模式

### Qwen 模型

对于 Qwen 模型，通过 `extra` 参数启用思考模式：

```rust
use vllm_client::{VllmClient, json};

let response = client
    .chat
    .completions()
    .create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "计算: 15 * 23 + 47 等于多少？"}
    ]))
    .extra(json!({
        "chat_template_kwargs": {
            "think_mode": true
        }
    }))
    .send()
    .await?;
```

### 检查推理内容

在非流式响应中，单独访问推理内容：

```rust
// 检查推理内容
if let Some(reasoning) = response.reasoning_content {
    println!("推理: {}", reasoning);
}

// 获取最终内容
if let Some(content) = response.content {
    println!("答案: {}", content);
}
```

## 带思考模式的流式响应

使用思考模式的最佳方式是流式响应：

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
        .model("Qwen/Qwen2.5-72B-Instruct")
        .messages(json!([
            {"role": "user", "content": "逐步思考: 如果我有 5 个苹果，给朋友 2 个，然后又买了 3 个，我有多少个？"}
        ]))
        .extra(json!({
            "chat_template_kwargs": {
                "think_mode": true
            }
        }))
        .stream(true)
        .send_stream()
        .await?;

    println!("=== 思考过程 ===\n");
    
    let mut in_thinking = true;
    let mut reasoning = String::new();
    let mut content = String::new();

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Reasoning(delta) => {
                reasoning.push_str(&delta);
                print!("{}", delta);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            StreamEvent::Content(delta) => {
                if in_thinking {
                    in_thinking = false;
                    println!("\n\n=== 最终答案 ===\n");
                }
                content.push_str(&delta);
                print!("{}", delta);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            StreamEvent::Done => break,
            StreamEvent::Error(e) => {
                eprintln!("\n错误: {}", e);
                break;
            }
            _ => {}
        }
    }

    println!();

    Ok(())
}
```

## 使用场景

### 数学推理

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

async fn solve_math_problem(client: &VllmClient, problem: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-72B-Instruct")
        .messages(json!([
            {"role": "system", "content": "你是一个数学辅导员。清晰地展示你的工作过程。"},
            {"role": "user", "content": problem}
        ]))
        .extra(json!({
            "chat_template_kwargs": {
                "think_mode": true
            }
        }))
        .stream(true)
        .send_stream()
        .await?;

    let mut answer = String::new();

    while let Some(event) = stream.next().await {
        if let StreamEvent::Content(delta) = event {
            answer.push_str(&delta);
        }
    }

    Ok(answer)
}
```

### 代码分析

```rust
let response = client
    .chat
    .completions()
    .create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "分析这段代码的潜在 bug 和安全问题:\n\n```rust\nfn process_input(input: &str) -> String {\n    let mut result = String::new();\n    for c in input.chars() {\n        result.push(c);\n    }\n    result\n}\n```"}
    ]))
    .extra(json!({
        "chat_template_kwargs": {
            "think_mode": true
        }
    }))
    .send()
    .await?;
```

### 复杂决策

```rust
let response = client
    .chat
    .completions()
    .create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "system", "content": "你是一个决策支持助手。仔细考虑所有选项。"},
        {"role": "user", "content": "我需要在公司 A（高薪，通勤远）和公司 B（中等薪资，远程工作）之间选择。帮我决定。"}
    ]))
    .extra(json!({
        "chat_template_kwargs": {
            "think_mode": true
        }
    }))
    .max_tokens(2048)
    .send()
    .await?;
```

## 分离推理和答案

对于需要将推理与最终答案分离的应用：

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

struct ThinkingResponse {
    reasoning: String,
    content: String,
}

async fn think_and_respond(
    client: &VllmClient,
    prompt: &str,
) -> Result<ThinkingResponse, Box<dyn std::error::Error>> {
    let mut stream = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-72B-Instruct")
        .messages(json!([
            {"role": "user", "content": prompt}
        ]))
        .extra(json!({
            "chat_template_kwargs": {
                "think_mode": true
            }
        }))
        .stream(true)
        .send_stream()
        .await?;

    let mut response = ThinkingResponse {
        reasoning: String::new(),
        content: String::new(),
    };

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Reasoning(delta) => response.reasoning.push_str(&delta),
            StreamEvent::Content(delta) => response.content.push_str(&delta),
            StreamEvent::Done => break,
            _ => {}
        }
    }

    Ok(response)
}
```

## 模型支持

| 模型 | 思考模式支持 |
|------|-------------|
| Qwen/Qwen2.5-72B-Instruct | ✅ 支持 |
| Qwen/Qwen2.5-32B-Instruct | ✅ 支持 |
| Qwen/Qwen2.5-7B-Instruct | ✅ 支持 |
| DeepSeek-R1 | ✅ 支持（内置） |
| 其他模型 | ❌ 取决于模型 |

检查您的 vLLM 服务器配置以验证思考模式支持。

## 配置选项

### 思考模型检测

模型自动处理思考标记：

```rust
// 推理内容从特殊标记中解析
// 通常结构为: <tool_call>...</think> 或类似格式
```

### 非流式访问

对于带推理的非流式请求：

```rust
let response = client
    .chat
    .completions()
    .create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "解释量子纠缠"}
    ]))
    .extra(json!({
        "chat_template_kwargs": {
            "think_mode": true
        }
    }))
    .send()
    .await?;

// 访问推理内容（如果存在）
if let Some(reasoning) = response.reasoning_content {
    println!("推理:\n{}\n", reasoning);
}

// 访问最终答案
println!("答案:\n{}", response.content.unwrap_or_default());
```

## 最佳实践

### 1. 用于复杂任务

思考模式对于以下场景最有价值：
- 多步推理
- 数学问题
- 代码分析
- 复杂决策

```rust
// 好: 复杂推理任务
.messages(json!([
    {"role": "user", "content": "解这道题: 父亲的年龄是儿子的 4 倍。20 年后，他只会是儿子的 2 倍。他们现在各多少岁？"}
]))

// 收益较小: 简单查询
.messages(json!([
    {"role": "user", "content": "2 + 2 等于几？"}
]))
```

### 2. 选择性显示推理

您可能希望在生产环境中隐藏推理，但在调试时显示：

```rust
let show_reasoning = std::env::var("SHOW_REASONING").is_ok();

while let Some(event) = stream.next().await {
    match event {
        StreamEvent::Reasoning(delta) => {
            if show_reasoning {
                eprintln!("[思考中] {}", delta);
            }
        }
        StreamEvent::Content(delta) => print!("{}", delta),
        _ => {}
    }
}
```

### 3. 结合系统提示

使用系统提示引导思考过程：

```rust
.messages(json!([
    {
        "role": "system", 
        "content": "逐步思考问题。在确定答案之前考虑多种方法。"
    },
    {"role": "user", "content": problem}
]))
```

### 4. 调整最大 Token 数

思考模式使用更多 token。请相应调整：

```rust
.max_tokens(4096)  // 考虑推理和答案两部分
```

## 故障排除

### 没有推理内容

如果看不到推理内容：

1. 确保在 `extra` 参数中启用了思考模式
2. 验证模型支持思考模式
3. 检查 vLLM 服务器配置

```bash
# 检查 vLLM 服务器日志以发现问题
```

### 流式响应不完整

如果流式响应似乎不完整：

```rust
// 确保处理所有事件类型
while let Some(event) = stream.next().await {
    match event {
        StreamEvent::Reasoning(delta) => { /* 处理 */ },
        StreamEvent::Content(delta) => { /* 处理 */ },
        StreamEvent::Done => break,
        StreamEvent::Error(e) => {
            eprintln!("错误: {}", e);
            break;
        }
        _ => {}  // 不要忘记其他事件
    }
}
```

## 相关链接

- [流式 API](../api/streaming.md) - 流式响应文档
- [示例](../examples.md) - 更多使用示例
- [高级主题](../advanced.md) - 其他高级功能