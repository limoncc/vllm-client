# Advanced Topics

This section covers advanced features and configurations for the vLLM Client.

## Table of Contents

- [Thinking Mode](#thinking-mode)
- [Custom Headers](#custom-headers)
- [Timeouts & Retries](#timeouts--retries)
- [Multi-modal Support](#multi-modal-support)
- [Debugging](#debugging)

## Thinking Mode

Some models (like Qwen with thinking mode) can output reasoning/thinking content before the final response. The vLLM Client provides built-in support for parsing these reasoning tokens.

### Enabling Thinking Mode

For Qwen models, enable thinking mode via the `extra` parameter:

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

let client = VllmClient::new("http://localhost:8000/v1");

let mut stream = client
    .chat
    .completions()
    .create()
    .model("Qwen3-35B")
    .messages(json!([
        {"role": "user", "content": "Solve: What is 15 * 23?"}
    ]))
    .extra(json!({
        "chat_template_kwargs": {
            "enable_thinking": true
        }
    }))
    .stream(true)
    .send_stream()
    .await?;

while let Some(event) = stream.next().await {
    match &event {
        StreamEvent::Reasoning(delta) => {
            // Thinking/reasoning content
            eprint!("[Thinking] {}", delta);
        }
        StreamEvent::Content(delta) => {
            // Final response content
            print!("{}", delta);
        }
        _ => {}
    }
}
```

### Reasoning vs Content

| Event Type | Description |
|------------|-------------|
| `StreamEvent::Reasoning` | Internal reasoning/thinking process |
| `StreamEvent::Content` | Final response content |

## Custom Headers

You can add custom headers to requests if needed:

```rust
use vllm_client::VllmClient;

let client = VllmClient::new("http://localhost:8000/v1")
    .with_header("X-Custom-Header", "custom-value")
    .with_header("X-Request-ID", "12345");
```

### Common Use Cases

- Adding trace IDs for debugging
- Custom authentication schemes
- Rate limiting headers

## Timeouts & Retries

### Setting Timeouts

```rust
use std::time::Duration;
use vllm_client::VllmClient;

let client = VllmClient::new("http://localhost:8000/v1")
    .with_timeout(Duration::from_secs(120)); // 2 minutes
```

### Implementing Retries

For retry logic, use a crate like `tokio-retry` or implement your own:

```rust
use std::time::Duration;
use vllm_client::{VllmClient, json, VllmError};

async fn send_with_retry(
    client: &VllmClient,
    messages: serde_json::Value,
    max_retries: u32,
) -> Result<ChatCompletionResponse, VllmError> {
    let mut attempts = 0;
    
    loop {
        match client
            .chat
            .completions()
            .create()
            .model("llama-3-70b")
            .messages(messages.clone())
            .send()
            .await
        {
            Ok(response) => return Ok(response),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                tokio::time::sleep(Duration::from_secs(2_u64.pow(attempts))).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Multi-modal Support

### Sending Images

For vision models, include images in your messages:

```rust
use vllm_client::{VllmClient, json};

let client = VllmClient::new("http://localhost:8000/v1");

let response = client
    .chat
    .completions()
    .create()
    .model("llava-v1.6-34b")
    .messages(json!([
        {
            "role": "user",
            "content": [
                {
                    "type": "text",
                    "text": "What's in this image?"
                },
                {
                    "type": "image_url",
                    "image_url": {
                        "url": "https://example.com/image.jpg"
                    }
                }
            ]
        }
    ]))
    .send()
    .await?;
```

### Base64 Images

You can also use base64-encoded images:

```rust
let response = client
    .chat
    .completions()
    .create()
    .model("llava-v1.6-34b")
    .messages(json!([
        {
            "role": "user",
            "content": [
                {
                    "type": "text",
                    "text": "Describe this image"
                },
                {
                    "type": "image_url",
                    "image_url": {
                        "url": "data:image/jpeg;base64,/9j/4AAQ..."
                    }
                }
            ]
        }
    ]))
    .send()
    .await?;
```

## Debugging

### Enabling Debug Logging

Enable debug logging to see request/response details:

```rust
use vllm_client::VllmClient;

let client = VllmClient::new("http://localhost:8000/v1")
    .with_debug(true);
```

### Inspecting Requests

For more detailed debugging, you can use environment variables:

```bash
RUST_LOG=debug cargo run
```

Or use `tracing` crate for structured logging:

```rust
use tracing::{info, debug};

info!("Sending request to vLLM");
debug!("Request payload: {:?}", payload);
```

## Performance Tips

### Connection Pooling

The client automatically uses connection pooling via `reqwest`. To customize:

```rust
use std::time::Duration;
use vllm_client::VllmClient;

let client = VllmClient::new("http://localhost:8000/v1")
    .with_pool_max_idle_per_host(10)
    .with_pool_idle_timeout(Duration::from_secs(90));
```

### Streaming vs Non-Streaming

- Use **streaming** for long-running responses to get immediate feedback
- Use **non-streaming** for batch processing or when you need the complete response at once

### Batch Processing

For processing multiple requests concurrently:

```rust
use futures::future::join_all;

let tasks: Vec<_> = prompts
    .iter()
    .map(|prompt| {
        client
            .chat
            .completions()
            .create()
            .model("llama-3-70b")
            .messages(json!([{"role": "user", "content": prompt}]))
            .send()
    })
    .collect();

let results = join_all(tasks).await;
```
