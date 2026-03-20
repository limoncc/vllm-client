# Advanced Topics

This section covers advanced features and patterns for vLLM Client.

## Available Topics

| Topic | Description |
|-------|-------------|
| [Thinking Mode](./advanced/thinking-mode.md) | Reasoning models and thinking content |
| [Custom Headers](./advanced/custom-headers.md) | Custom HTTP headers and authentication |
| [Timeouts & Retries](./advanced/timeouts.md) | Timeout configuration and retry strategies |

## Thinking Mode

For models that support reasoning (like Qwen with thinking mode), access the `reasoning_content` field:

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

let mut stream = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "Solve this puzzle"}]))
    .extra(json!({"chat_template_kwargs": {"think_mode": true}}))
    .stream(true)
    .send_stream()
    .await?;

while let Some(event) = stream.next().await {
    match event {
        StreamEvent::Reasoning(delta) => eprintln!("[thinking] {}", delta),
        StreamEvent::Content(delta) => print!("{}", delta),
        _ => {}
    }
}
```

## Custom Configuration

### Environment-Based Configuration

```rust
use std::env;
use vllm_client::VllmClient;

fn create_client() -> VllmClient {
    VllmClient::builder()
        .base_url(env::var("VLLM_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8000/v1".to_string()))
        .api_key(env::var("VLLM_API_KEY").ok())
        .timeout_secs(env::var("VLLM_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300))
        .build()
}
```

### Multiple Clients

```rust
use vllm_client::VllmClient;

let primary = VllmClient::new("http://primary-server:8000/v1");
let fallback = VllmClient::new("http://fallback-server:8000/v1");
```

## Production Patterns

### Connection Pooling

The client reuses HTTP connections automatically. Create once and share:

```rust
use std::sync::Arc;
use vllm_client::VllmClient;

let client = Arc::new(VllmClient::new("http://localhost:8000/v1"));

// Clone the Arc for each task
let client1 = Arc::clone(&client);
let client2 = Arc::clone(&client);
```

### Graceful Shutdown

Handle graceful shutdown with channels:

```rust
use tokio::signal;
use tokio::sync::broadcast;

let (shutdown_tx, _) = broadcast::channel::<()>(1);

// In your request loop
tokio::select! {
    result = make_request(&client) => {
        // Handle result
    }
    _ = shutdown_rx.recv() => {
        println!("Shutting down gracefully");
        break;
    }
}
```

### Request Queuing

For rate limiting, implement a queue:

```rust
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(10)); // Max 10 concurrent

async fn queued_request(client: &VllmClient, prompt: &str) -> Result<String, VllmError> {
    let _permit = semaphore.acquire().await.unwrap();
    client.chat.completions().create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": prompt}]))
        .send()
        .await
        .map(|r| r.content.unwrap_or_default())
}
```

## Performance Tips

### 1. Reuse the Client

Creating a client has some overhead. Reuse it across requests:

```rust
// Good
let client = VllmClient::new("http://localhost:8000/v1");
for prompt in prompts {
    let _ = client.chat.completions().create()...;
}

// Avoid
for prompt in prompts {
    let client = VllmClient::new("http://localhost:8000/v1"); // Inefficient!
    let _ = client.chat.completions().create()...;
}
```

### 2. Use Streaming for Long Responses

Get faster time-to-first-token with streaming:

```rust
// Faster perceived latency
let mut stream = client.chat.completions().create()
    .stream(true)
    .send_stream()
    .await?;
```

### 3. Set Appropriate Timeouts

Match timeout to expected response time:

```rust
// Short queries
let client = VllmClient::new("http://localhost:8000/v1")
    .timeout_secs(30);

// Long generation tasks
let client = VllmClient::new("http://localhost:8000/v1")
    .timeout_secs(600);
```

### 4. Batch Requests

Process multiple prompts concurrently:

```rust
use futures::stream::{self, StreamExt};

let prompts = vec!["Hello", "Hi", "Hey"];
let results: Vec<_> = stream::iter(prompts)
    .map(|p| async {
        client.chat.completions().create()
            .model("Qwen/Qwen2.5-7B-Instruct")
            .messages(json!([{"role": "user", "content": p}]))
            .send()
            .await
    })
    .buffer_unordered(5) // Max 5 concurrent
    .collect()
    .await;
```

## Security Considerations

### API Key Storage

Never hardcode API keys:

```rust
// Good: Use environment variables
let api_key = std::env::var("VLLM_API_KEY")?;

// Avoid: Hardcoded keys
let api_key = "sk-secret-key"; // DON'T DO THIS!
```

### TLS Verification

The client uses `reqwest` which verifies TLS certificates by default. For development with self-signed certificates:

```rust
// Use a custom HTTP client if needed
let http = reqwest::Client::builder()
    .danger_accept_invalid_certs(true) // Only for development!
    .timeout(std::time::Duration::from_secs(300))
    .build()?;
```

## See Also

- [API Reference](./api.md) - Complete API documentation
- [Examples](./examples.md) - Practical code examples
- [Error Handling](./api/error-handling.md) - Error handling strategies