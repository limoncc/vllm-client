# Streaming Chat Example

This example demonstrates how to use streaming responses for real-time output.

## Basic Streaming

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
            {"role": "user", "content": "Write a short story about a robot learning to paint."}
        ]))
        .temperature(0.8)
        .max_tokens(1024)
        .stream(true)
        .send_stream()
        .await?;

    print!("Response: ");
    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Content(delta) => {
                print!("{}", delta);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            StreamEvent::Done => break,
            StreamEvent::Error(e) => {
                eprintln!("\nError: {}", e);
                break;
            }
            _ => {}
        }
    }
    println!();

    Ok(())
}
```

## Streaming with Reasoning (Thinking Models)

For models that support thinking/reasoning mode:

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
            {"role": "user", "content": "Solve: What is 15 * 23 + 47?"}
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
                eprintln!("[thinking] {}", delta);
            }
            StreamEvent::Content(delta) => {
                content.push_str(&delta);
                print!("{}", delta);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            StreamEvent::Done => break,
            StreamEvent::Error(e) => {
                eprintln!("\nError: {}", e);
                break;
            }
            _ => {}
        }
    }

    println!("\n");
    if !reasoning.is_empty() {
        println!("--- Reasoning Process ---");
        println!("{}", reasoning);
    }

    Ok(())
}
```

## Streaming with Progress Indicator

Add a typing indicator while waiting for the first token:

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

    // Spawn typing indicator task
    let indicator = tokio::spawn(async move {
        let chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let mut i = 0;
        while waiting_clone.load(Ordering::Relaxed) {
            print!("\r{} Thinking...", chars[i]);
            std::io::Write::flush(&mut std::io::stdout()).ok();
            i = (i + 1) % chars.len();
            tokio::time::sleep(Duration::from_millis(80)).await;
        }
        print!("\r        \r"); // Clear the indicator
    });

    let mut stream = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "Explain quantum entanglement in simple terms."}
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
                    println!("Response:");
                    println!("---------");
                }
                content.push_str(&delta);
                print!("{}", delta);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            StreamEvent::Done => break,
            StreamEvent::Error(e) => {
                waiting.store(false, Ordering::Relaxed);
                eprintln!("\nError: {}", e);
                break;
            }
            _ => {}
        }
    }

    println!("\n");

    Ok(())
}
```

## Multi-turn Streaming Conversation

Handle a conversation with streaming responses:

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;
use std::io::{self, BufRead, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    let mut messages: Vec<serde_json::Value> = Vec::new();

    println!("Chat with the AI (type 'quit' to exit)");
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

        // Add user message
        messages.push(json!({"role": "user", "content": input}));

        // Stream response
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
                    eprintln!("\nError: {}", e);
                    break;
                }
                _ => {}
            }
        }

        println!("\n");

        // Add assistant response to history
        messages.push(json!({"role": "assistant", "content": response_content}));
    }

    println!("Goodbye!");
    Ok(())
}
```

## Streaming with Timeout

Add timeout handling for slow responses:

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
            {"role": "user", "content": "Write a detailed essay about AI."}
        ]))
        .stream(true)
        .send_stream()
        .await?;

    let mut content = String::new();

    loop {
        // 30 second timeout per event
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
                        eprintln!("\nStream error: {}", e);
                        return Err(e.into());
                    }
                    _ => {}
                }
            }
            Ok(None) => break,
            Err(_) => {
                eprintln!("\nTimeout waiting for next token");
                break;
            }
        }
    }

    println!("\n\nGenerated {} characters", content.len());

    Ok(())
}
```

## Collecting Usage Statistics

Track token usage during streaming:

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
            {"role": "user", "content": "Write a poem about the ocean."}
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
    println!("--- Statistics ---");
    println!("Time: {:.2}s", elapsed.as_secs_f64());
    println!("Characters: {}", content.len());

    if let Some(usage) = usage {
        println!("Prompt tokens: {}", usage.prompt_tokens);
        println!("Completion tokens: {}", usage.completion_tokens);
        println!("Total tokens: {}", usage.total_tokens);
        println!("Tokens/second: {:.2}", 
            usage.completion_tokens as f64 / elapsed.as_secs_f64());
    }

    Ok(())
}
```

## See Also

- [Basic Chat](./basic-chat.md) - Simple chat completion
- [Tool Calling](./tool-calling.md) - Function calling examples
- [Streaming API](../api/streaming.md) - Streaming API reference