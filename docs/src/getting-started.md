# Getting Started

## Installation

Add `vllm-client` to your `Cargo.toml`:

```toml
[dependencies]
vllm-client = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

### Basic Chat Completion

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = VllmClient::new("http://localhost:8000/v1");
    
    // Send a chat completion request
    let response = client
        .chat
        .completions()
        .create()
        .model("your-model-name")
        .messages(json!([
            {"role": "user", "content": "Hello, how are you?"}
        ]))
        .send()
        .await?;
    
    // Print the response
    println!("{}", response.choices[0].message.content);
    
    Ok(())
}
```

### Streaming Response

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
        .model("your-model-name")
        .messages(json!([
            {"role": "user", "content": "Write a poem about spring"}
        ]))
        .stream(true)
        .send_stream()
        .await?;
    
    while let Some(event) = stream.next().await {
        match &event {
            StreamEvent::Reasoning(delta) => print!("{}", delta),
            StreamEvent::Content(delta) => print!("{}", delta),
            _ => {}
        }
    }
    
    println!();
    Ok(())
}
```

## Configuration

### API Key

If your vLLM server requires authentication:

```rust
let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key("your-api-key");
```

### Custom Timeout

```rust
let client = VllmClient::new("http://localhost:8000/v1")
    .with_timeout(std::time::Duration::from_secs(60));
```

## Next Steps

- [API Reference](./api.md) - Complete API documentation
- [Examples](./examples/basic.md) - More usage examples
- [Advanced Features](./advanced/thinking.md) - Thinking mode, tool calling, etc.