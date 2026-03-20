# Examples

This section contains practical code examples demonstrating vLLM Client usage patterns.

## Available Examples

### Basic Usage

| Example | Description |
|---------|-------------|
| [Basic Chat](./examples/basic-chat.md) | Simple chat completion requests |
| [Streaming Chat](./examples/streaming-chat.md) | Real-time streaming responses |
| [Tool Calling](./examples/tool-calling.md) | Function calling integration |
| [Multi-modal](./examples/multimodal.md) | Image and multi-modal inputs |

## Quick Examples

### Hello World

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let response = client.chat.completions().create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": "Hello!"}]))
        .send()
        .await?;
    
    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

### Streaming Output

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let mut stream = client.chat.completions().create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": "Tell me a story"}]))
        .stream(true)
        .send_stream()
        .await?;
    
    while let Some(event) = stream.next().await {
        if let StreamEvent::Content(delta) = event {
            print!("{}", delta);
        }
    }
    
    println!();
    Ok(())
}
```

### Tool Calling

```rust
use vllm_client::{VllmClient, json};

let tools = json!([
    {
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "Get weather for a location",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                },
                "required": ["location"]
            }
        }
    }
]);

let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-7B-Instruct")
    .messages(json!([
        {"role": "user", "content": "What's the weather in Tokyo?"}
    ]))
    .tools(tools)
    .send()
    .await?;

if response.has_tool_calls() {
    // Execute tools and return results
}
```

## Example Structure

Each example includes:
- Complete, runnable code
- Required dependencies
- Step-by-step explanations
- Common variations and use cases

## Running Examples

### Prerequisites

1. A running vLLM server:
   ```bash
   pip install vllm
   vllm serve Qwen/Qwen2.5-7B-Instruct --port 8000
   ```

2. Rust toolchain:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### Running an Example

```bash
# Create a new project
cargo new my-vllm-app
cd my-vllm-app

# Add dependencies
cargo add vllm-client
cargo add tokio --features full
cargo add serde_json

# Copy example code to src/main.rs
# Then run:
cargo run
```

## Common Patterns

### Environment Configuration

```rust
use std::env;
use vllm_client::VllmClient;

fn create_client() -> VllmClient {
    VllmClient::builder()
        .base_url(env::var("VLLM_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8000/v1".to_string()))
        .api_key(env::var("VLLM_API_KEY").ok())
        .timeout_secs(300)
        .build()
}
```

### Error Handling

```rust
use vllm_client::{VllmClient, VllmError};

async fn safe_chat(prompt: &str) -> Result<String, VllmError> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let response = client.chat.completions().create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": prompt}]))
        .send()
        .await?;
    
    Ok(response.content.unwrap_or_default())
}
```

### Reusing Client

```rust
use std::sync::Arc;
use vllm_client::VllmClient;

// Share client across threads
let client = Arc::new(VllmClient::new("http://localhost:8000/v1"));

// Use in multiple async tasks
let client1 = Arc::clone(&client);
let client2 = Arc::clone(&client);
```

## See Also

- [Getting Started](./getting-started.md) - Installation and setup
- [API Reference](./api.md) - Detailed API documentation
- [Advanced Topics](./advanced.md) - Advanced usage patterns