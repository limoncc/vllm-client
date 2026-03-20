# Configuration

This page covers all configuration options for `vllm-client`.

## Client Configuration

### Basic Setup

```rust
use vllm_client::VllmClient;

let client = VllmClient::new("http://localhost:8000/v1");
```

### Using the Builder Pattern

For more complex configurations, use the builder pattern:

```rust
use vllm_client::VllmClient;

let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .api_key("your-api-key")
    .timeout_secs(120)
    .build();
```

## Configuration Options

### Base URL

The base URL of your vLLM server. This should include the `/v1` path for OpenAI compatibility.

```rust
// Local development
let client = VllmClient::new("http://localhost:8000/v1");

// Remote server
let client = VllmClient::new("https://api.example.com/v1");

// With trailing slash (automatically normalized)
let client = VllmClient::new("http://localhost:8000/v1/");
// Equivalent to: "http://localhost:8000/v1"
```

### API Key

If your vLLM server requires authentication, configure the API key:

```rust
// Using method chain
let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key("sk-your-api-key");

// Using builder
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .api_key("sk-your-api-key")
    .build();
```

The API key is sent as a Bearer token in the `Authorization` header.

### Timeout

Configure the request timeout for long-running operations:

```rust
// Using method chain
let client = VllmClient::new("http://localhost:8000/v1")
    .timeout_secs(300); // 5 minutes

// Using builder
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .timeout_secs(300)
    .build();
```

Default timeout uses the underlying HTTP client's default (usually 30 seconds).

## Request Configuration

When making requests, you can configure various parameters:

### Model Selection

```rust
use vllm_client::{VllmClient, json};

let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "Hello!"}]))
    .send()
    .await?;
```

### Sampling Parameters

```rust
let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "Hello!"}]))
    .temperature(0.7)      // 0.0 - 2.0
    .top_p(0.9)            // 0.0 - 1.0
    .top_k(50)             // vLLM extension
    .max_tokens(1024)      // Max output tokens
    .send()
    .await?;
```

| Parameter | Type | Range | Description |
|-----------|------|-------|-------------|
| `temperature` | f32 | 0.0 - 2.0 | Controls randomness. Higher = more random |
| `top_p` | f32 | 0.0 - 1.0 | Nucleus sampling threshold |
| `top_k` | i32 | 1+ | Top-K sampling (vLLM extension) |
| `max_tokens` | u32 | 1+ | Maximum tokens to generate |

### Stop Sequences

```rust
use serde_json::json;

// Multiple stop sequences
let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "Hello!"}]))
    .stop(json!(["END", "STOP", "\n\n"]))
    .send()
    .await?;

// Single stop sequence
let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "Hello!"}]))
    .stop(json!("END"))
    .send()
    .await?;
```

### Extra Parameters

vLLM supports additional parameters via the `extra()` method:

```rust
let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "Think about this"}]))
    .extra(json!({
        "chat_template_kwargs": {
            "think_mode": true
        },
        "reasoning_effort": "high"
    }))
    .send()
    .await?;
```

## Environment Variables

You can use environment variables to configure the client:

```rust
use std::env;
use vllm_client::VllmClient;

let base_url = env::var("VLLM_BASE_URL")
    .unwrap_or_else(|_| "http://localhost:8000/v1".to_string());

let api_key = env::var("VLLM_API_KEY").ok();

let mut client_builder = VllmClient::builder()
    .base_url(&base_url);

if let Some(key) = api_key {
    client_builder = client_builder.api_key(&key);
}

let client = client_builder.build();
```

### Recommended Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `VLLM_BASE_URL` | vLLM server URL | `http://localhost:8000/v1` |
| `VLLM_API_KEY` | API key (optional) | `sk-xxx` |
| `VLLM_TIMEOUT` | Timeout in seconds | `300` |

## Best Practices

### Reusing the Client

Create the client once and reuse it for multiple requests:

```rust
// Good: Reuse client
let client = VllmClient::new("http://localhost:8000/v1");

for prompt in prompts {
    let response = client.chat.completions().create()
        .model("Qwen/Qwen2.5-72B-Instruct")
        .messages(json!([{"role": "user", "content": prompt}]))
        .send()
        .await?;
}

// Avoid: Creating client for each request
for prompt in prompts {
    let client = VllmClient::new("http://localhost:8000/v1"); // Inefficient!
    // ...
}
```

### Timeout Selection

Choose appropriate timeouts based on your use case:

| Use Case | Recommended Timeout |
|----------|---------------------|
| Simple queries | 30 seconds |
| Complex reasoning | 2-5 minutes |
| Long document generation | 10+ minutes |

### Error Handling

Always handle errors appropriately:

```rust
use vllm_client::{VllmClient, VllmError};

match client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "Hello!"}]))
    .send()
    .await
{
    Ok(response) => println!("{}", response.content.unwrap()),
    Err(VllmError::Timeout) => eprintln!("Request timed out"),
    Err(VllmError::ApiError { status_code, message, .. }) => {
        eprintln!("API error ({}): {}", status_code, message);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Next Steps

- [Quick Start](./quick-start.md) - Basic usage examples
- [API Reference](../api.md) - Complete API documentation
- [Error Handling](../api/error-handling.md) - Detailed error handling guide