# Client API

The `VllmClient` is the main entry point for interacting with the vLLM API.

## Creating a Client

### Simple Construction

```rust
use vllm_client::VllmClient;

let client = VllmClient::new("http://localhost:8000/v1");
```

### With API Key

```rust
use vllm_client::VllmClient;

let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key("sk-your-api-key");
```

### With Timeout

```rust
use vllm_client::VllmClient;

let client = VllmClient::new("http://localhost:8000/v1")
    .timeout_secs(120); // 2 minutes
```

### Using the Builder Pattern

For more complex configurations, use the builder:

```rust
use vllm_client::VllmClient;

let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .api_key("sk-your-api-key")
    .timeout_secs(300)
    .build();
```

## Methods Reference

### `new(base_url: impl Into<String>) -> Self`

Create a new client with the given base URL.

```rust
let client = VllmClient::new("http://localhost:8000/v1");
```

**Parameters:**
- `base_url` - The base URL of the vLLM server (should include `/v1` path)

**Notes:**
- Trailing slashes are automatically removed
- The client is cheap to create but should be reused when possible

### `with_api_key(self, api_key: impl Into<String>) -> Self`

Set the API key for authentication (builder pattern).

```rust
let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key("sk-xxx");
```

**Parameters:**
- `api_key` - The API key to use for Bearer authentication

**Notes:**
- The API key is sent as a Bearer token in the `Authorization` header
- This method returns a new client instance

### `timeout_secs(self, secs: u64) -> Self`

Set the request timeout in seconds (builder pattern).

```rust
let client = VllmClient::new("http://localhost:8000/v1")
    .timeout_secs(300);
```

**Parameters:**
- `secs` - Timeout duration in seconds

**Notes:**
- Applies to all requests made by this client
- For long-running generation tasks, consider setting a higher timeout

### `base_url(&self) -> &str`

Get the base URL of the client.

```rust
let client = VllmClient::new("http://localhost:8000/v1");
assert_eq!(client.base_url(), "http://localhost:8000/v1");
```

### `api_key(&self) -> Option<&str>`

Get the API key, if configured.

```rust
let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key("sk-xxx");
assert_eq!(client.api_key(), Some("sk-xxx"));
```

### `builder() -> VllmClientBuilder`

Create a new client builder for more configuration options.

```rust
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .api_key("sk-xxx")
    .timeout_secs(120)
    .build();
```

## API Modules

The client provides access to different API modules:

### `chat` - Chat Completions API

Access the chat completions API for conversational interactions:

```rust
let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "Hello!"}]))
    .send()
    .await?;
```

### `completions` - Legacy Completions API

Access the legacy completions API for text completion:

```rust
let response = client.completions.create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .prompt("Once upon a time")
    .send()
    .await?;
```

## VllmClientBuilder

The builder provides a flexible way to configure the client.

### Methods

| Method | Type | Description |
|--------|------|-------------|
| `base_url(url)` | `impl Into<String>` | Set the base URL |
| `api_key(key)` | `impl Into<String>` | Set the API key |
| `timeout_secs(secs)` | `u64` | Set timeout in seconds |
| `build()` | - | Build the client |

### Default Values

| Option | Default |
|--------|---------|
| `base_url` | `http://localhost:8000/v1` |
| `api_key` | `None` |
| `timeout_secs` | HTTP client default (30s) |

## Usage Examples

### Basic Usage

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let response = client.chat.completions().create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "Hello!"}
        ]))
        .send()
        .await?;
    
    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

### With Environment Variables

```rust
use std::env;
use vllm_client::VllmClient;

fn create_client() -> VllmClient {
    let base_url = env::var("VLLM_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8000/v1".to_string());
    
    let api_key = env::var("VLLM_API_KEY").ok();
    
    let mut builder = VllmClient::builder().base_url(&base_url);
    
    if let Some(key) = api_key {
        builder = builder.api_key(&key);
    }
    
    builder.build()
}
```

### Multiple Requests

Reuse the client for multiple requests:

```rust
use vllm_client::{VllmClient, json};

async fn process_prompts(client: &VllmClient, prompts: &[&str]) -> Vec<String> {
    let mut results = Vec::new();
    
    for prompt in prompts {
        let response = client.chat.completions().create()
            .model("Qwen/Qwen2.5-7B-Instruct")
            .messages(json!([{"role": "user", "content": prompt}]))
            .send()
            .await;
        
        match response {
            Ok(r) => results.push(r.content.unwrap_or_default()),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    
    results
}
```

## Thread Safety

The `VllmClient` is thread-safe and can be shared across threads:

```rust
use std::sync::Arc;
use vllm_client::VllmClient;

let client = Arc::new(VllmClient::new("http://localhost:8000/v1"));

// Can be cloned and shared across threads
let client_clone = Arc::clone(&client);
```

## See Also

- [Chat Completions](./chat-completions.md) - Chat completions API
- [Streaming](./streaming.md) - Streaming response handling
- [Configuration](../getting-started/configuration.md) - Configuration options