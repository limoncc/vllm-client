# API Reference

This document provides a comprehensive reference for the vLLM Client API.

## Table of Contents

- [Client](#client)
- [Chat Completions](#chat-completions)
- [Streaming](#streaming)
- [Tool Calling](#tool-calling)
- [Types](#types)
- [Error Handling](#error-handling)

## Client

### `VllmClient`

The main client for interacting with vLLM API.

```rust
use vllm_client::VllmClient;

// Create a new client
let client = VllmClient::new("http://localhost:8000/v1");

// With API key
let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key("your-api-key");

// With custom timeout
let client = VllmClient::new("http://localhost:8000/v1")
    .with_timeout(std::time::Duration::from_secs(60));
```

#### Methods

| Method | Description |
|--------|-------------|
| `new(base_url: &str)` | Create a new client with the given base URL |
| `with_api_key(key: &str)` | Set the API key for authentication |
| `with_timeout(duration)` | Set the request timeout |
| `chat` | Access the chat completions API |

---

## Chat Completions

### Creating a Completion

```rust
use vllm_client::{VllmClient, json};

let client = VllmClient::new("http://localhost:8000/v1");

let response = client
    .chat
    .completions()
    .create()
    .model("llama-3-70b")
    .messages(json!([
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello!"}
    ]))
    .temperature(0.7)
    .max_tokens(1000)
    .send()
    .await?;
```

### Builder Methods

| Method | Type | Description |
|--------|------|-------------|
| `model(name)` | `&str` | Model name to use |
| `messages(msgs)` | `Value` | Chat messages array |
| `temperature(temp)` | `f32` | Sampling temperature (0.0-2.0) |
| `max_tokens(tokens)` | `u32` | Maximum tokens to generate |
| `top_p(p)` | `f32` | Nucleus sampling parameter |
| `top_k(k)` | `u32` | Top-k sampling parameter |
| `stream(enable)` | `bool` | Enable streaming response |
| `tools(tools)` | `Value` | Tool definitions for function calling |
| `extra(json)` | `Value` | Extra parameters (vendor-specific) |

### Response Structure

```rust
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: Option<String>,
}

pub struct Message {
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

---

## Streaming

### Streaming Completions

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

let client = VllmClient::new("http://localhost:8000/v1");

let mut stream = client
    .chat
    .completions()
    .create()
    .model("llama-3-70b")
    .messages(json!([
        {"role": "user", "content": "Write a poem"}
    ]))
    .stream(true)
    .send_stream()
    .await?;

while let Some(event) = stream.next().await {
    match &event {
        StreamEvent::Reasoning(delta) => {
            // Reasoning content (for thinking models)
            print!("{}", delta);
        }
        StreamEvent::Content(delta) => {
            // Regular content
            print!("{}", delta);
        }
        StreamEvent::ToolCallDelta { tool_call_id, delta } => {
            // Tool call streaming
        }
        StreamEvent::ToolCallComplete(tool_call) => {
            // Complete tool call
        }
        StreamEvent::Usage(usage) => {
            // Token usage information
        }
        StreamEvent::Done => {
            // Stream completed
            break;
        }
        StreamEvent::Error(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
```

### StreamEvent Types

| Variant | Description |
|---------|-------------|
| `Reasoning(String)` | Reasoning/thinking content |
| `Content(String)` | Regular content delta |
| `ToolCallDelta { tool_call_id, delta }` | Streaming tool call |
| `ToolCallComplete(ToolCall)` | Complete tool call |
| `Usage(Usage)` | Token usage stats |
| `Done` | Stream finished |
| `Error(VllmError)` | Error occurred |

---

## Tool Calling

### Defining Tools

```rust
use vllm_client::json;

let tools = json!([
    {
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "Get current weather for a location",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "City name"
                    }
                },
                "required": ["location"]
            }
        }
    }
]);

let response = client
    .chat
    .completions()
    .create()
    .model("llama-3-70b")
    .messages(json!([
        {"role": "user", "content": "What's the weather in Tokyo?"}
    ]))
    .tools(tools)
    .send()
    .await?;

// Handle tool calls
if let Some(tool_calls) = response.choices[0].message.tool_calls {
    for tool_call in tool_calls {
        println!("Function: {}", tool_call.function.name);
        println!("Arguments: {}", tool_call.function.arguments);
    }
}
```

### ToolCall Structure

```rust
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: FunctionCall,
}

pub struct FunctionCall {
    pub name: String,
    pub arguments: String, // JSON string
}
```

### Returning Tool Results

```rust
// After executing the tool, return the result
let response = client
    .chat
    .completions()
    .create()
    .model("llama-3-70b")
    .messages(json!([
        {"role": "user", "content": "What's the weather in Tokyo?"},
        {"role": "assistant", "tool_calls": [
            {
                "id": "call_123",
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "arguments": "{\"location\": \"Tokyo\"}"
                }
            }
        ]},
        {
            "role": "tool",
            "tool_call_id": "call_123",
            "content": "{\"temperature\": 25, \"condition\": \"sunny\"}"
        }
    ]))
    .tools(tools)
    .send()
    .await?;
```

---

## Types

### Message Types

```rust
// System message
json!({"role": "system", "content": "You are a helpful assistant."})

// User message
json!({"role": "user", "content": "Hello!"})

// Assistant message
json!({"role": "assistant", "content": "Hi there!"})

// Tool result message
json!({
    "role": "tool",
    "tool_call_id": "call_123",
    "content": "result"
})
```

### vLLM-Specific Parameters

Use `.extra()` to pass vLLM-specific parameters:

```rust
client
    .chat
    .completions()
    .create()
    .model("qwen-3")
    .messages(json!([{"role": "user", "content": "Think about this"}]))
    .extra(json!({
        "chat_template_kwargs": {
            "enable_thinking": true
        }
    }))
    .send()
    .await?;
```

---

## Error Handling

### VllmError

```rust
use vllm_client::VllmError;

match client.chat.completions().create().send().await {
    Ok(response) => { /* ... */ },
    Err(VllmError::HttpError(e)) => {
        eprintln!("HTTP error: {}", e);
    }
    Err(VllmError::ApiError { message, code }) => {
        eprintln!("API error ({}): {}", code, message);
    }
    Err(VllmError::StreamError(e)) => {
        eprintln!("Stream error: {}", e);
    }
    Err(VllmError::ParseError(e)) => {
        eprintln!("Parse error: {}", e);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

### Error Types

| Variant | Description |
|---------|-------------|
| `HttpError` | HTTP request/response errors |
| `ApiError` | API-level errors (rate limits, etc.) |
| `StreamError` | Streaming-specific errors |
| `ParseError` | JSON parsing errors |
| `IoError` | I/O errors |

---

## Complete Example

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1")
        .with_api_key("your-api-key");

    // Streaming example
    let mut stream = client
        .chat
        .completions()
        .create()
        .model("llama-3-70b")
        .messages(json!([
            {"role": "user", "content": "Write a haiku about coding"}
        ]))
        .temperature(0.7)
        .max_tokens(100)
        .stream(true)
        .send_stream()
        .await?;

    while let Some(event) = stream.next().await {
        match &event {
            StreamEvent::Content(delta) => print!("{}", delta),
            StreamEvent::Done => break,
            StreamEvent::Error(e) => eprintln!("Error: {}", e),
            _ => {}
        }
    }

    println!();
    Ok(())
}
```
