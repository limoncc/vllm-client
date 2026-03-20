# Custom Headers

This document explains how to use custom HTTP headers with vLLM Client.

## Overview

While the vLLM Client handles standard authentication via API keys, you may need to add custom headers for:
- Custom authentication schemes
- Request tracing and debugging
- Rate limiting identifiers
- Custom metadata

## Current Limitations

The current version of vLLM Client does not provide a built-in method for custom headers. However, you can work around this limitation in several ways.

## Workaround: Environment Variables

If your vLLM server accepts configuration via environment variables or specific API parameters:

```rust
use vllm_client::{VllmClient, json};

let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key(std::env::var("MY_API_KEY").unwrap_or_default());
```

## Workaround: Via Extra Parameters

Some custom configurations can be passed through the `extra()` method:

```rust
use vllm_client::{VllmClient, json};

let response = client
    .chat
    .completions()
    .create()
    .model("Qwen/Qwen2.5-7B-Instruct")
    .messages(json!([{"role": "user", "content": "Hello!"}]))
    .extra(json!({
        "custom_field": "custom_value",
        "request_id": "req-12345"
    }))
    .send()
    .await?;
```

## Future Support

Custom header support is planned for future versions. The API will likely look like:

```rust,ignore
// Future API (not yet implemented)
let client = VllmClient::new("http://localhost:8000/v1")
    .with_header("X-Custom-Header", "value")
    .with_header("X-Request-ID", "req-123");
```

## Common Use Cases

### Tracing Headers

For distributed tracing (when supported):

```rust,ignore
// Future API
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .header("X-Trace-ID", trace_id)
    .header("X-Span-ID", span_id)
    .build();
```

### Custom Authentication

For non-standard authentication schemes:

```rust,ignore
// Future API
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .header("X-API-Key", "custom-key")
    .header("X-Tenant-ID", "tenant-123")
    .build();
```

### Request Metadata

Add metadata for logging or analytics:

```rust,ignore
// Future API
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .header("X-Request-Source", "mobile-app")
    .header("X-User-ID", "user-456")
    .build();
```

## Alternative: Custom HTTP Client

For advanced use cases, you can use the underlying `reqwest` client directly:

```rust
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let response = client
        .post("http://localhost:8000/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer your-api-key")
        .header("X-Custom-Header", "custom-value")
        .json(&json!({
            "model": "Qwen/Qwen2.5-7B-Instruct",
            "messages": [{"role": "user", "content": "Hello!"}]
        }))
        .send()
        .await?;
    
    let result: serde_json::Value = response.json().await?;
    println!("{:?}", result);
    
    Ok(())
}
```

## Best Practices

### 1. Use Standard Authentication When Possible

```rust
// Preferred
let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key("your-api-key");

// Avoid custom auth unless necessary
```

### 2. Document Custom Headers

When using custom headers, document their purpose:

```rust,ignore
// Future API
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    // For request tracing in logs
    .header("X-Request-ID", &request_id)
    // For multi-tenant identification
    .header("X-Tenant-ID", &tenant_id)
    .build();
```

### 3. Validate Server Support

Ensure your vLLM server accepts and processes custom headers. Some proxies or load balancers may strip unknown headers.

## Security Considerations

### Don't Expose Sensitive Headers

Avoid logging headers that contain sensitive information:

```rust,ignore
// Be careful with logging
let auth_header = "Bearer secret-key";
// Don't log this directly!
```

### Use HTTPS

Always use HTTPS when transmitting sensitive headers:

```rust
// Good
let client = VllmClient::new("https://api.example.com/v1");

// Avoid for sensitive data
let client = VllmClient::new("http://api.example.com/v1");
```

## Requesting This Feature

If you need custom header support, please open an issue on GitHub with:
1. Your use case
2. Required headers
3. How you'd like the API to look

## See Also

- [Timeouts & Retries](./timeouts.md) - Configure timeouts and retry logic
- [Thinking Mode](./thinking-mode.md) - Reasoning model support
- [Client API](../api/client.md) - Client configuration options