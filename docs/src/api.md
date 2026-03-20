# API Reference

This section provides detailed documentation for the vLLM Client API.

## Design Philosophy

The vLLM Client API follows these design principles:

### Builder Pattern

All request constructions use the builder pattern for ergonomic and flexible API calls:

```rust
let response = client.chat.completions().create()
    .model("model-name")
    .messages(json!([{"role": "user", "content": "Hello"}]))
    .temperature(0.7)
    .max_tokens(1024)
    .send()
    .await?;
```

### Async-First

All API operations are async, built on Tokio. Use `#[tokio::main]` or integrate with your existing runtime:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Your async code here
}
```

### Type Safety

Strong types are used throughout the library with Serde serialization:

- `ChatCompletionResponse` - Response from chat completions
- `StreamEvent` - Events from streaming responses
- `ToolCall` - Tool/function call data
- `VllmError` - Comprehensive error types

### OpenAI Compatibility

The API mirrors the OpenAI API structure, making it easy to migrate existing code:

| OpenAI | vLLM Client |
|--------|-------------|
| `client.chat.completions.create(...)` | `client.chat.completions().create()...send().await` |
| `stream=True` | `.stream(true).send_stream().await` |
| `tools=[...]` | `.tools(json!([...]))` |

## Module Structure

```
VllmClient
├── chat
│   └── completions()      # Chat completions API
│       ├── create()       # Create request builder
│       └── send()         # Execute request
│       └── send_stream()  # Execute with streaming
├── completions            # Legacy completions API
└── builder()              # Client builder
```

## Core Types

### Request Types

| Type | Description |
|------|-------------|
| `ChatCompletionsRequest` | Builder for chat completion requests |
| `VllmClientBuilder` | Builder for client configuration |

### Response Types

| Type | Description |
|------|-------------|
| `ChatCompletionResponse` | Response from chat completions |
| `CompletionResponse` | Response from legacy completions |
| `MessageStream` | Streaming response iterator |
| `StreamEvent` | Individual stream events |
| `ToolCall` | Tool/function call data |
| `Usage` | Token usage statistics |

### Error Types

| Type | Description |
|------|-------------|
| `VllmError::Http` | HTTP request failed |
| `VllmError::Json` | JSON serialization error |
| `VllmError::ApiError` | API returned error |
| `VllmError::Stream` | Streaming error |
| `VllmError::Timeout` | Connection timeout |

## Quick Reference

### Creating a Client

```rust
use vllm_client::VllmClient;

// Simple
let client = VllmClient::new("http://localhost:8000/v1");

// With API key
let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key("sk-xxx");

// With builder
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .api_key("sk-xxx")
    .timeout_secs(120)
    .build();
```

### Chat Completion

```rust
use vllm_client::{VllmClient, json};

let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-7B-Instruct")
    .messages(json!([
        {"role": "user", "content": "Hello!"}
    ]))
    .temperature(0.7)
    .max_tokens(1024)
    .send()
    .await?;

println!("{}", response.content.unwrap());
```

### Streaming

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

let mut stream = client.chat.completions().create()
    .model("Qwen/Qwen2.5-7B-Instruct")
    .messages(json!([{"role": "user", "content": "Hello!"}]))
    .stream(true)
    .send_stream()
    .await?;

while let Some(event) = stream.next().await {
    match event {
        StreamEvent::Content(delta) => print!("{}", delta),
        StreamEvent::Reasoning(delta) => eprintln!("[thinking] {}", delta),
        StreamEvent::Done => break,
        _ => {}
    }
}
```

## Sections

- [Client](./api/client.md) - VllmClient configuration and methods
- [Chat Completions](./api/chat-completions.md) - Chat completions API
- [Streaming](./api/streaming.md) - Streaming response handling
- [Tool Calling](./api/tool-calling.md) - Function/tool calling
- [Error Handling](./api/error-handling.md) - Error types and handling