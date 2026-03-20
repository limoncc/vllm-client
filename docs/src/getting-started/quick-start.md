# Quick Start

This guide will help you make your first API call with vLLM Client.

## Prerequisites

- Rust 1.70 or later
- A running vLLM server

## Basic Chat Completion

The simplest way to use the client is with a synchronous-style chat completion:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client pointing to your vLLM server
    let client = VllmClient::new("http://localhost:8000/v1");

    // Send a chat completion request
    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "Hello, how are you?"}
        ]))
        .send()
        .await?;

    // Print the response
    println!("Response: {}", response.content.unwrap_or_default());

    Ok(())
}
```

## Streaming Response

For real-time output, use streaming:

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    // Create a streaming request
    let mut stream = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "Write a short poem about spring"}
        ]))
        .stream(true)
        .send_stream()
        .await?;

    // Process streaming events
    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Content(delta) => print!("{}", delta),
            StreamEvent::Reasoning(delta) => eprint!("[thinking: {}]", delta),
            StreamEvent::Done => println!("\n[Done]"),
            StreamEvent::Error(e) => eprintln!("\nError: {}", e),
            _ => {}
        }
    }

    Ok(())
}
```

## Using the Builder Pattern

For more configuration options, use the builder:

```rust
use vllm_client::VllmClient;

let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .api_key("your-api-key")  // Optional
    .timeout_secs(120)         // Optional
    .build();
```

## Complete Example with Options

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
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "What is the capital of France?"}
        ]))
        .temperature(0.7)
        .max_tokens(1024)
        .top_p(0.9)
        .send()
        .await?;

    println!("Response: {}", response.content.unwrap_or_default());
    
    // Print usage statistics if available
    if let Some(usage) = response.usage {
        println!("Tokens: prompt={}, completion={}, total={}",
            usage.prompt_tokens,
            usage.completion_tokens,
            usage.total_tokens
        );
    }

    Ok(())
}
```

## Error Handling

Handle errors gracefully:

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
            {"role": "user", "content": "Hello!"}
        ]))
        .send()
        .await?;

    Ok(response.content.unwrap_or_default())
}

#[tokio::main]
async fn main() {
    match chat().await {
        Ok(text) => println!("Response: {}", text),
        Err(VllmError::ApiError { status_code, message, .. }) => {
            eprintln!("API Error ({}): {}", status_code, message);
        }
        Err(VllmError::Timeout) => {
            eprintln!("Request timed out");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
```

## Next Steps

- [Configuration](./configuration.md) - Learn about all configuration options
- [API Reference](../api.md) - Detailed API documentation
- [Examples](../examples.md) - More usage examples