# Streaming API

Streaming responses allow you to process LLM output in real-time, token by token, instead of waiting for the complete response.

## Overview

vLLM Client provides streaming support through Server-Sent Events (SSE). Use `send_stream()` instead of `send()` to get a streaming response.

## Basic Streaming

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
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "Write a poem about spring"}
        ]))
        .stream(true)
        .send_stream()
        .await?;

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Content(delta) => print!("{}", delta),
            StreamEvent::Done => break,
            _ => {}
        }
    }

    println!();
    Ok(())
}
```

## StreamEvent Types

The `StreamEvent` enum represents different types of streaming events:

| Variant | Description |
|---------|-------------|
| `Content(String)` | Regular content token delta |
| `Reasoning(String)` | Reasoning/thinking content (for thinking models) |
| `ToolCallDelta` | Streaming tool call delta |
| `ToolCallComplete(ToolCall)` | Complete tool call ready to execute |
| `Usage(Usage)` | Token usage statistics |
| `Done` | Stream completed successfully |
| `Error(VllmError)` | An error occurred |

### Content Events

The most common event type, containing text tokens:

```rust
match event {
    StreamEvent::Content(delta) => {
        print!("{}", delta);
        std::io::Write::flush(&mut std::io::stdout()).ok();
    }
    _ => {}
}
```

### Reasoning Events

For models with reasoning capabilities (like Qwen with thinking mode):

```rust
match event {
    StreamEvent::Reasoning(delta) => {
        eprintln!("[thinking] {}", delta);
    }
    StreamEvent::Content(delta) => {
        print!("{}", delta);
    }
    _ => {}
}
```

### Tool Call Events

Tool calls are streamed incrementally and then completed:

```rust
match event {
    StreamEvent::ToolCallDelta { index, id, name, arguments } => {
        println!("Tool delta: index={}, name={}", index, name);
        // Arguments are streamed as partial JSON
    }
    StreamEvent::ToolCallComplete(tool_call) => {
        println!("Tool ready: {}({})", tool_call.name, tool_call.arguments);
        // Execute the tool and return result
    }
    _ => {}
}
```

### Usage Events

Token usage information is typically sent at the end:

```rust
match event {
    StreamEvent::Usage(usage) => {
        println!("Tokens: prompt={}, completion={}, total={}",
            usage.prompt_tokens,
            usage.completion_tokens,
            usage.total_tokens
        );
    }
    _ => {}
}
```

## MessageStream

The `MessageStream` type is an async iterator that yields `StreamEvent` values.

### Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `next()` | `Option<StreamEvent>` | Get next event (async) |
| `collect_content()` | `String` | Collect all content into a string |
| `into_stream()` | `impl Stream` | Convert to generic stream |

### Collect All Content

For convenience, you can collect all content at once:

```rust
let content = stream.collect_content().await?;
println!("Full response: {}", content);
```

Note: This waits for the complete response, defeating the purpose of streaming. Use only when you need both streaming display and the full text.

## Complete Streaming Example

```rust
use vllm_client::{VllmClient, json, StreamEvent, VllmError};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), VllmError> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let mut stream = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "Explain quantum computing in simple terms"}
        ]))
        .temperature(0.7)
        .max_tokens(1024)
        .stream(true)
        .send_stream()
        .await?;

    let mut reasoning = String::new();
    let mut content = String::new();
    let mut usage = None;

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Reasoning(delta) => {
                reasoning.push_str(&delta);
            }
            StreamEvent::Content(delta) => {
                content.push_str(&delta);
                print!("{}", delta);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            StreamEvent::Usage(u) => {
                usage = Some(u);
            }
            StreamEvent::Done => {
                println!("\n[Stream completed]");
            }
            StreamEvent::Error(e) => {
                eprintln!("\nError: {}", e);
                return Err(e);
            }
            _ => {}
        }
    }

    // Print summary
    if !reasoning.is_empty() {
        eprintln!("\n--- Reasoning ---");
        eprintln!("{}", reasoning);
    }

    if let Some(usage) = usage {
        eprintln!("\n--- Token Usage ---");
        eprintln!("Prompt: {}, Completion: {}, Total: {}",
            usage.prompt_tokens,
            usage.completion_tokens,
            usage.total_tokens
        );
    }

    Ok(())
}
```

## Streaming with Tool Calling

When streaming with tools, you'll receive incremental tool call updates:

```rust
use vllm_client::{VllmClient, json, StreamEvent, ToolCall};
use futures::StreamExt;

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

let mut stream = client
    .chat
    .completions()
    .create()
    .model("Qwen/Qwen2.5-7B-Instruct")
    .messages(json!([
        {"role": "user", "content": "What's the weather in Tokyo?"}
    ]))
    .tools(tools)
    .stream(true)
    .send_stream()
    .await?;

let mut tool_calls: Vec<ToolCall> = Vec::new();

while let Some(event) = stream.next().await {
    match event {
        StreamEvent::Content(delta) => print!("{}", delta),
        StreamEvent::ToolCallComplete(tool_call) => {
            tool_calls.push(tool_call);
        }
        StreamEvent::Done => break,
        _ => {}
    }
}

// Execute tool calls
for tool_call in tool_calls {
    println!("Tool: {} with args: {}", tool_call.name, tool_call.arguments);
    // Execute and return result in next message
}
```

## Error Handling

Streaming errors can occur at any point:

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
            StreamEvent::Error(e) => return Err(e),
            StreamEvent::Done => break,
            _ => {}
        }
    }

    Ok(content)
}
```

## Best Practices

### Flush Output

For real-time display, flush stdout after each token:

```rust
use std::io::{self, Write};

match event {
    StreamEvent::Content(delta) => {
        print!("{}", delta);
        io::stdout().flush().ok();
    }
    _ => {}
}
```

### Handle Interruption

For interactive applications, handle Ctrl+C gracefully:

```rust
use tokio::signal;

tokio::select! {
    result = process_stream(&mut stream) => {
        // Normal completion
    }
    _ = signal::ctrl_c() => {
        println!("\n[interrupted]");
    }
}
```

### Timeout for Idle Streams

Set a timeout for streams that may hang:

```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(60),
    stream.next()
).await;

match result {
    Ok(Some(event)) => { /* process event */ }
    Ok(None) => { /* stream ended */ }
    Err(_) => { /* timeout */ }
}
```

## Completions Streaming

The vLLM Client also supports streaming for the legacy `/v1/completions` API using `CompletionStreamEvent`.

### CompletionStreamEvent Types

| Variant | Description |
|---------|-------------|
| `Text(String)` | Text token delta |
| `FinishReason(String)` | Reason why the stream finished (e.g., "stop", "length") |
| `Usage(Usage)` | Token usage statistics |
| `Done` | Stream completed successfully |
| `Error(VllmError)` | An error occurred |

### Completions Streaming Example

```rust
use vllm_client::{VllmClient, json, CompletionStreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let mut stream = client
        .completions
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .prompt("Write a poem about spring")
        .max_tokens(1024)
        .temperature(0.7)
        .stream(true)
        .send_stream()
        .await?;

    while let Some(event) = stream.next().await {
        match event {
            CompletionStreamEvent::Text(delta) => {
                print!("{}", delta);
                std::io::stdout().flush().ok();
            }
            CompletionStreamEvent::FinishReason(reason) => {
                println!("\n[Finish reason: {}]", reason);
            }
            CompletionStreamEvent::Usage(usage) => {
                println!("\nTokens: prompt={}, completion={}, total={}",
                    usage.prompt_tokens,
                    usage.completion_tokens,
                    usage.total_tokens
                );
            }
            CompletionStreamEvent::Done => {
                println!("\n[Stream completed]");
            }
            CompletionStreamEvent::Error(e) => {
                eprintln!("Error: {}", e);
                return Err(e.into());
            }
        }
    }

    Ok(())
}
```

### CompletionStream Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `next()` | `Option<CompletionStreamEvent>` | Get next event (async) |
| `collect_text()` | `String` | Collect all text into a string |
| `into_stream()` | `impl Stream` | Convert to generic stream |

## Next Steps

- [Tool Calling](./tool-calling.md) - Using function calling
- [Error Handling](./error-handling.md) - Comprehensive error handling
- [Examples](../examples/streaming-chat.md) - More streaming examples