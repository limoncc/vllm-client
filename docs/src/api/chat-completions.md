# Chat Completions API

The Chat Completions API is the primary interface for generating text responses from a language model.

## Overview

Access the chat completions API through `client.chat.completions()`:

```rust
use vllm_client::{VllmClient, json};

let client = VllmClient::new("http://localhost:8000/v1");

let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "Hello!"}
    ]))
    .send()
    .await?;
```

## Request Builder

### Required Parameters

#### `model(name: impl Into<String>)`

Set the model name to use for generation.

```rust
.model("Qwen/Qwen2.5-72B-Instruct")
// or
.model("meta-llama/Llama-3-70b")
```

#### `messages(messages: Value)`

Set the conversation messages as a JSON array.

```rust
.messages(json!([
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "What is Rust?"}
]))
```

### Message Types

| Role | Description |
|------|-------------|
| `system` | Set the behavior of the assistant |
| `user` | User input |
| `assistant` | Assistant response (for multi-turn) |
| `tool` | Tool result (for function calling) |

### Sampling Parameters

#### `temperature(temp: f32)`

Controls randomness. Range: `0.0` to `2.0`.

```rust
.temperature(0.7)  // Default-like behavior
.temperature(0.0)  // Deterministic
.temperature(1.5)  // More creative
```

#### `max_tokens(tokens: u32)`

Maximum number of tokens to generate.

```rust
.max_tokens(1024)
.max_tokens(4096)
```

#### `top_p(p: f32)`

Nucleus sampling threshold. Range: `0.0` to `1.0`.

```rust
.top_p(0.9)
```

#### `top_k(k: i32)`

Top-K sampling (vLLM extension). Limits to top K tokens.

```rust
.top_k(50)
```

#### `stop(sequences: Value)`

Stop generation when encountering these sequences.

```rust
// Multiple sequences
.stop(json!(["END", "STOP", "\n\n"]))

// Single sequence
.stop(json!("---"))
```

### Tool Calling Parameters

#### `tools(tools: Value)`

Define tools/functions that the model can call.

```rust
.tools(json!([
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
]))
```

#### `tool_choice(choice: Value)`

Control tool selection behavior.

```rust
.tool_choice(json!("auto"))       // Model decides
.tool_choice(json!("none"))       // No tools
.tool_choice(json!("required"))   // Force tool use
.tool_choice(json!({
    "type": "function",
    "function": {"name": "get_weather"}
}))
```

### Advanced Parameters

#### `stream(enable: bool)`

Enable streaming response.

```rust
.stream(true)
```

#### `extra(params: Value)`

Pass vLLM-specific or additional parameters.

```rust
.extra(json!({
    "chat_template_kwargs": {
        "think_mode": true
    },
    "reasoning_effort": "high"
}))
```

## Sending Requests

### `send()` - Synchronous Response

Returns the complete response at once.

```rust
let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "Hello!"}]))
    .send()
    .await?;
```

### `send_stream()` - Streaming Response

Returns a stream for real-time output.

```rust
let mut stream = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "Hello!"}]))
    .stream(true)
    .send_stream()
    .await?;
```

See [Streaming](./streaming.md) for detailed streaming documentation.

## Response Structure

### `ChatCompletionResponse`

| Field | Type | Description |
|-------|------|-------------|
| `raw` | `Value` | Raw JSON response |
| `id` | `String` | Response ID |
| `object` | `String` | Object type |
| `model` | `String` | Model used |
| `content` | `Option<String>` | Generated content |
| `reasoning_content` | `Option<String>` | Reasoning content (thinking models) |
| `tool_calls` | `Option<Vec<ToolCall>>` | Tool calls made |
| `finish_reason` | `Option<String>` | Why generation stopped |
| `usage` | `Option<Usage>` | Token usage statistics |

### Example Usage

```rust
let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "What is 2+2?"}
    ]))
    .send()
    .await?;

// Access content
println!("Content: {}", response.content.unwrap_or_default());

// Check for reasoning (thinking models)
if let Some(reasoning) = response.reasoning_content {
    println!("Reasoning: {}", reasoning);
}

// Check finish reason
match response.finish_reason.as_deref() {
    Some("stop") => println!("Natural stop"),
    Some("length") => println!("Max tokens reached"),
    Some("tool_calls") => println!("Tool calls made"),
    _ => {}
}

// Token usage
if let Some(usage) = response.usage {
    println!("Prompt tokens: {}", usage.prompt_tokens);
    println!("Completion tokens: {}", usage.completion_tokens);
    println!("Total tokens: {}", usage.total_tokens);
}
```

## Complete Example

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client.chat.completions().create()
        .model("Qwen/Qwen2.5-72B-Instruct")
        .messages(json!([
            {"role": "system", "content": "You are a helpful coding assistant."},
            {"role": "user", "content": "Write a function to reverse a string in Rust"}
        ]))
        .temperature(0.7)
        .max_tokens(1024)
        .top_p(0.9)
        .send()
        .await?;

    if let Some(content) = response.content {
        println!("{}", content);
    }

    Ok(())
}
```

## Multi-turn Conversation

```rust
use vllm_client::{VllmClient, json};

let client = VllmClient::new("http://localhost:8000/v1");

// First message
let response1 = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "My name is Alice"}
    ]))
    .send()
    .await?;

// Continue conversation
let response2 = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "My name is Alice"},
        {"role": "assistant", "content": response1.content.unwrap()},
        {"role": "user", "content": "What's my name?"}
    ]))
    .send()
    .await?;
```

## See Also

- [Streaming](./streaming.md) - Streaming responses
- [Tool Calling](./tool-calling.md) - Function calling
- [Client](./client.md) - Client configuration