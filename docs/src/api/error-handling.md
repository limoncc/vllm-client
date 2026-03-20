# Error Handling

This document covers error handling in vLLM Client.

## VllmError Enum

All errors in vLLM Client are represented by the `VllmError` enum:

```rust
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum VllmError {
    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("JSON error: {0}")]
    Json(String),

    #[error("API error (status {status_code}): {message}")]
    ApiError {
        status_code: u16,
        message: String,
        error_type: Option<String>,
    },

    #[error("Stream error: {0}")]
    Stream(String),

    #[error("Connection timeout")]
    Timeout,

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("No response content")]
    NoContent,

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("{0}")]
    Other(String),
}
```

## Error Types

| Variant | When It Occurs |
|---------|----------------|
| `Http` | Network errors, connection failures |
| `Json` | Serialization/deserialization errors |
| `ApiError` | Server returned an error response |
| `Stream` | Errors during streaming response |
| `Timeout` | Request timed out |
| `ModelNotFound` | Specified model doesn't exist |
| `MissingParameter` | Required parameter not provided |
| `NoContent` | Response has no content |
| `InvalidResponse` | Unexpected response format |
| `Other` | Miscellaneous errors |

## Basic Error Handling

```rust
use vllm_client::{VllmClient, json, VllmError};

async fn chat(prompt: &str) -> Result<String, VllmError> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": prompt}]))
        .send()
        .await?;

    Ok(response.content.unwrap_or_default())
}

#[tokio::main]
async fn main() {
    match chat("Hello!").await {
        Ok(text) => println!("Response: {}", text),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Detailed Error Handling

Handle specific error types differently:

```rust
use vllm_client::{VllmClient, json, VllmError};

#[tokio::main]
async fn main() {
    let client = VllmClient::new("http://localhost:8000/v1");

    let result = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": "Hello!"}]))
        .send()
        .await;

    match result {
        Ok(response) => {
            println!("Success: {}", response.content.unwrap_or_default());
        }
        Err(VllmError::ApiError { status_code, message, error_type }) => {
            eprintln!("API Error (HTTP {}): {}", status_code, message);
            if let Some(etype) = error_type {
                eprintln!("Error type: {}", etype);
            }
        }
        Err(VllmError::Timeout) => {
            eprintln!("Request timed out. Try increasing timeout.");
        }
        Err(VllmError::Http(msg)) => {
            eprintln!("Network error: {}", msg);
        }
        Err(VllmError::ModelNotFound(model)) => {
            eprintln!("Model '{}' not found. Check available models.", model);
        }
        Err(VllmError::MissingParameter(param)) => {
            eprintln!("Missing required parameter: {}", param);
        }
        Err(e) => {
            eprintln!("Other error: {}", e);
        }
    }
}
```

## HTTP Status Codes

Common API error status codes:

| Code | Meaning | Action |
|------|---------|--------|
| 400 | Bad Request | Check request parameters |
| 401 | Unauthorized | Check API key |
| 403 | Forbidden | Check permissions |
| 404 | Not Found | Check endpoint or model name |
| 429 | Rate Limited | Implement retry with backoff |
| 500 | Server Error | Retry or contact admin |
| 502 | Bad Gateway | Check vLLM server status |
| 503 | Service Unavailable | Wait and retry |
| 504 | Gateway Timeout | Increase timeout or retry |

## Retryable Errors

Check if an error is retryable:

```rust
use vllm_client::VllmError;

fn should_retry(error: &VllmError) -> bool {
    error.is_retryable()
}

// Manual check
match error {
    VllmError::Timeout => true,
    VllmError::ApiError { status_code: 429, .. } => true,  // Rate limit
    VllmError::ApiError { status_code: 500..=504, .. } => true,  // Server errors
    _ => false,
}
```

## Retry with Exponential Backoff

```rust
use vllm_client::{VllmClient, json, VllmError};
use std::time::Duration;
use tokio::time::sleep;

async fn chat_with_retry(
    client: &VllmClient,
    prompt: &str,
    max_retries: u32,
) -> Result<String, VllmError> {
    let mut retries = 0;

    loop {
        let result = client
            .chat
            .completions()
            .create()
            .model("Qwen/Qwen2.5-7B-Instruct")
            .messages(json!([{"role": "user", "content": prompt}]))
            .send()
            .await;

        match result {
            Ok(response) => {
                return Ok(response.content.unwrap_or_default());
            }
            Err(e) if e.is_retryable() && retries < max_retries => {
                retries += 1;
                let delay = Duration::from_millis(100 * 2u64.pow(retries - 1));
                eprintln!("Retry {} after {:?}: {}", retries, delay, e);
                sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Streaming Error Handling

Handle errors during streaming:

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
            StreamEvent::Done => break,
            StreamEvent::Error(e) => return Err(e),
            _ => {}
        }
    }

    Ok(content)
}
```

## Error Context

Add context to errors for better debugging:

```rust
use vllm_client::{VllmClient, json, VllmError};

async fn chat_with_context(prompt: &str) -> Result<String, String> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": prompt}]))
        .send()
        .await
        .map_err(|e| format!("Failed to get chat response: {}", e))?;

    Ok(response.content.unwrap_or_default())
}
```

## Using anyhow or eyre

For applications using `anyhow` or `eyre`:

```rust
use vllm_client::{VllmClient, json, VllmError};
use anyhow::{Context, Result};

async fn chat(prompt: &str) -> Result<String> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": prompt}]))
        .send()
        .await
        .context("Failed to send chat request")?;

    Ok(response.content.unwrap_or_default())
}
```

## Best Practices

### 1. Always Handle Errors

```rust
// Bad
let response = client.chat.completions().create()
    .send().await.unwrap();

// Good
match client.chat.completions().create().send().await {
    Ok(r) => { /* handle */ },
    Err(e) => eprintln!("Error: {}", e),
}
```

### 2. Use Appropriate Timeout

```rust
let client = VllmClient::new("http://localhost:8000/v1")
    .timeout_secs(300); // 5 minutes for long tasks
```

### 3. Log Errors with Context

```rust
Err(e) => {
    log::error!("Chat request failed: {}", e);
    log::debug!("Request details: model={}, prompt_len={}", model, prompt.len());
}
```

### 4. Implement Graceful Degradation

```rust
match primary_client.chat.completions().create().send().await {
    Ok(r) => r,
    Err(e) => {
        log::warn!("Primary client failed: {}, trying fallback", e);
        fallback_client.chat.completions().create().send().await?
    }
}
```

## See Also

- [Client](./client.md) - Client configuration
- [Streaming](./streaming.md) - Streaming error handling
- [Timeouts & Retries](../advanced/timeouts.md) - Advanced timeout configuration