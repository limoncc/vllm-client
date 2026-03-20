# 流式聊天示例

本示例演示如何使用流式响应实现实时输出。

## 基础流式响应

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
            {"role": "user", "content": "写一个关于机器人学习绘画的短篇故事。"}
        ]))
        .temperature(0.8)
        .max_tokens(1024)
        .stream(true)
        .send_stream()
        .await?;

    print!("响应: ");
    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Content(delta) => {
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

## 带推理过程的流式响应（思考模型）

对于支持思考/推理模式的模型：

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
            {"role": "user", "content": "计算: 15 * 23 + 47 等于多少？"}
        ]))
        .extra(json!({
            "chat_template_kwargs": {
                "think_mode": true
            }
        }))
        .stream(true)
        .send_stream()
        .await?;

    let mut reasoning = String::new();
    let mut content = String::new();

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Reasoning(delta) => {
                reasoning.push_str(&delta);
                eprintln!("[思考中] {}", delta);
            }
            StreamEvent::Content(delta) => {
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

    println!("\n");
    if !reasoning.is_empty() {
        println!("--- 推理过程 ---");
        println!("{}", reasoning);
    }

    Ok(())
}
```

## 带进度指示器的流式响应

在等待第一个 token 时显示输入指示器：

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let waiting = Arc::new(AtomicBool::new(true));
    let waiting_clone = Arc::clone(&waiting);

    // 启动输入指示器任务
    let indicator = tokio::spawn(async move {
        let chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let mut i = 0;
        while waiting_clone.load(Ordering::Relaxed) {
            print!("\r{} 思考中...", chars[i]);
            std::io::Write::flush(&mut std::io::stdout()).ok();
            i = (i + 1) % chars.len();
            tokio::time::sleep(Duration::from_millis(80)).await;
        }
        print!("\r        \r"); // 清除指示器
    });

    let mut stream = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "用简单的语言解释量子纠缠。"}
        ]))
        .stream(true)
        .send_stream()
        .await?;

    let mut first_token = true;
    let mut content = String::new();

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Content(delta) => {
                if first_token {
                    waiting.store(false, Ordering::Relaxed);
                    indicator.await.ok();
                    first_token = false;
                    println!("响应:");
                    println!("---------");
                }
                content.push_str(&delta);
                print!("{}", delta);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            StreamEvent::Done => break,
            StreamEvent::Error(e) => {
                waiting.store(false, Ordering::Relaxed);
                eprintln!("\n错误: {}", e);
                break;
            }
            _ => {}
        }
    }

    println!("\n");

    Ok(())
}
```

## 多轮流式对话

处理带有流式响应的对话：

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;
use std::io::{self, BufRead, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    let mut messages: Vec<serde_json::Value> = Vec::new();

    println!("与 AI 聊天（输入 'quit' 退出）");
    println!("----------------------------------------\n");

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = line?;
        if input.trim() == "quit" {
            break;
        }
        if input.trim().is_empty() {
            continue;
        }

        // 添加用户消息
        messages.push(json!({"role": "user", "content": input}));

        // 流式响应
        let mut stream = client
            .chat
            .completions()
            .create()
            .model("Qwen/Qwen2.5-7B-Instruct")
            .messages(json!(messages))
            .stream(true)
            .send_stream()
            .await?;

        print!("AI: ");
        io::stdout().flush().ok();

        let mut response_content = String::new();

        while let Some(event) = stream.next().await {
            match event {
                StreamEvent::Content(delta) => {
                    response_content.push_str(&delta);
                    print!("{}", delta);
                    io::stdout().flush().ok();
                }
                StreamEvent::Done => break,
                StreamEvent::Error(e) => {
                    eprintln!("\n错误: {}", e);
                    break;
                }
                _ => {}
            }
        }

        println!("\n");

        // 将助手响应添加到历史
        messages.push(json!({"role": "assistant", "content": response_content}));
    }

    println!("再见！");
    Ok(())
}
```

## 带超时的流式响应

为慢速响应添加超时处理：

```rust
use vllm_client::{VllmClient, json, StreamEvent, VllmError};
use futures::StreamExt;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1")
        .timeout_secs(300);

    let mut stream = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "写一篇关于人工智能的详细论文。"}
        ]))
        .stream(true)
        .send_stream()
        .await?;

    let mut content = String::new();

    loop {
        // 每个事件 30 秒超时
        match timeout(Duration::from_secs(30), stream.next()).await {
            Ok(Some(event)) => {
                match event {
                    StreamEvent::Content(delta) => {
                        content.push_str(&delta);
                        print!("{}", delta);
                        std::io::Write::flush(&mut std::io::stdout()).ok();
                    }
                    StreamEvent::Done => break,
                    StreamEvent::Error(e) => {
                        eprintln!("\n流式错误: {}", e);
                        return Err(e.into());
                    }
                    _ => {}
                }
            }
            Ok(None) => break,
            Err(_) => {
                eprintln!("\n等待下一个 token 超时");
                break;
            }
        }
    }

    println!("\n\n生成了 {} 个字符", content.len());

    Ok(())
}
```

## 收集使用统计

在流式响应过程中追踪 token 使用情况：

```rust
use vllm_client::{VllmClient, json, StreamEvent, Usage};
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
            {"role": "user", "content": "写一首关于海洋的诗。"}
        ]))
        .stream(true)
        .send_stream()
        .await?;

    let mut content = String::new();
    let mut usage: Option<Usage> = None;
    let mut start_time = std::time::Instant::now();
    let mut token_count = 0;

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Content(delta) => {
                content.push_str(&delta);
                token_count += 1;
                print!("{}", delta);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            StreamEvent::Usage(u) => {
                usage = Some(u);
            }
            StreamEvent::Done => break,
            _ => {}
        }
    }

    let elapsed = start_time.elapsed();

    println!("\n");
    println!("--- 统计信息 ---");
    println!("耗时: {:.2}秒", elapsed.as_secs_f64());
    println!("字符数: {}", content.len());

    if let Some(usage) = usage {
        println!("提示词 tokens: {}", usage.prompt_tokens);
        println!("补全 tokens: {}", usage.completion_tokens);
        println!("总 tokens: {}", usage.total_tokens);
        println!("每秒 tokens: {:.2}", 
            usage.completion_tokens as f64 / elapsed.as_secs_f64());
    }

    Ok(())
}
```

## 相关链接

- [基础聊天](./basic-chat.md) - 简单聊天补全
- [工具调用](./tool-calling.md) - 函数调用示例
- [流式 API](../api/streaming.md) - 流式 API 参考