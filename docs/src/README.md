# vLLM Client

A Rust client library for vLLM API with OpenAI-compatible interface.

## Features

- **OpenAI Compatible**: Uses the same API structure as OpenAI, making it easy to switch
- **Streaming Support**: Full support for streaming responses with Server-Sent Events (SSE)
- **Tool Calling**: Support for function/tool calling with streaming delta updates
- **Reasoning Models**: Built-in support for reasoning/thinking models (like Qwen with thinking mode)
- **Async/Await**: Fully async using Tokio runtime
- **Type Safe**: Strong types with Serde serialization

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
vllm-client = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Basic Usage

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let response = client
        .chat
        .completions()
        .create()
        .model("your-model-name")
        .messages(json!([
            {"role": "user", "content": "Hello, world!"}
        ]))
        .send()
        .await?;
    
    println!("{}", response.choices[0].message.content);
    Ok(())
}
```

## Documentation

- [Getting Started](./getting-started.md) - Installation and basic setup
- [API Reference](./api.md) - Complete API documentation
- [Examples](./examples/basic.md) - Code examples
- [Advanced Topics](./advanced/streaming.md) - Streaming, tools, and more

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.