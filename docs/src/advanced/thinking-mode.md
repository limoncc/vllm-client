# Thinking Mode

Thinking mode (also known as reasoning mode) allows models to output their reasoning process before giving a final answer. This is particularly useful for complex reasoning tasks.

## Overview

Some models, like Qwen with thinking mode enabled, can output two types of content:

1. **Reasoning Content** - The model's internal "thinking" process
2. **Content** - The final response to the user

## Enabling Thinking Mode

### Qwen Models

For Qwen models, enable thinking mode via the `extra` parameter:

```rust
use vllm_client::{VllmClient, json};

let response = client
    .chat
    .completions()
    .create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "Solve: What is 15 * 23 + 47?"}
    ]))
    .extra(json!({
        "chat_template_kwargs": {
            "think_mode": true
        }
    }))
    .send()
    .await?;
```

### Checking for Reasoning Content

In non-streaming responses, access reasoning content separately:

```rust
// Check for reasoning content
if let Some(reasoning) = response.reasoning_content {
    println!("Reasoning: {}", reasoning);
}

// Get final content
if let Some(content) = response.content {
    println!("Answer: {}", content);
}
```

## Streaming with Thinking Mode

The best way to use thinking mode is with streaming:

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
        .model("Qwen/Qwen2.5-72B-Instruct")
        .messages(json!([
            {"role": "user", "content": "Think step by step: If I have 5 apples and give 2 to my friend, then buy 3 more, how many do I have?"}
        ]))
        .extra(json!({
            "chat_template_kwargs": {
                "think_mode": true
            }
        }))
        .stream(true)
        .send_stream()
        .await?;

    println!("=== Thinking Process ===\n");
    
    let mut in_thinking = true;
    let mut reasoning = String::new();
    let mut content = String::new();

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Reasoning(delta) => {
                reasoning.push_str(&delta);
                print!("{}", delta);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            StreamEvent::Content(delta) => {
                if in_thinking {
                    in_thinking = false;
                    println!("\n\n=== Final Answer ===\n");
                }
                content.push_str(&delta);
                print!("{}", delta);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            StreamEvent::Done => break,
            StreamEvent::Error(e) => {
                eprintln!("\nError: {}", e);
                break;
            }
            _ => {}
        }
    }

    println!();

    Ok(())
}
```

## Use Cases

### Mathematical Reasoning

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

async fn solve_math_problem(client: &VllmClient, problem: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-72B-Instruct")
        .messages(json!([
            {"role": "system", "content": "You are a math tutor. Show your work clearly."},
            {"role": "user", "content": problem}
        ]))
        .extra(json!({
            "chat_template_kwargs": {
                "think_mode": true
            }
        }))
        .stream(true)
        .send_stream()
        .await?;

    let mut answer = String::new();

    while let Some(event) = stream.next().await {
        if let StreamEvent::Content(delta) = event {
            answer.push_str(&delta);
        }
    }

    Ok(answer)
}
```

### Code Analysis

```rust
let response = client
    .chat
    .completions()
    .create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "Analyze this code for potential bugs and security issues:\n\n```rust\nfn process_input(input: &str) -> String {\n    let mut result = String::new();\n    for c in input.chars() {\n        result.push(c);\n    }\n    result\n}\n```"}
    ]))
    .extra(json!({
        "chat_template_kwargs": {
            "think_mode": true
        }
    }))
    .send()
    .await?;
```

### Complex Decision Making

```rust
let response = client
    .chat
    .completions()
    .create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "system", "content": "You are a decision support assistant. Think through all options carefully."},
        {"role": "user", "content": "I need to choose between job offers from Company A (high salary, long commute) and Company B (moderate salary, remote work). Help me decide."}
    ]))
    .extra(json!({
        "chat_template_kwargs": {
            "think_mode": true
        }
    }))
    .max_tokens(2048)
    .send()
    .await?;
```

## Separating Reasoning from Answer

For applications that need to separate reasoning from the final answer:

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

struct ThinkingResponse {
    reasoning: String,
    content: String,
}

async fn think_and_respond(
    client: &VllmClient,
    prompt: &str,
) -> Result<ThinkingResponse, Box<dyn std::error::Error>> {
    let mut stream = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-72B-Instruct")
        .messages(json!([
            {"role": "user", "content": prompt}
        ]))
        .extra(json!({
            "chat_template_kwargs": {
                "think_mode": true
            }
        }))
        .stream(true)
        .send_stream()
        .await?;

    let mut response = ThinkingResponse {
        reasoning: String::new(),
        content: String::new(),
    };

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Reasoning(delta) => response.reasoning.push_str(&delta),
            StreamEvent::Content(delta) => response.content.push_str(&delta),
            StreamEvent::Done => break,
            _ => {}
        }
    }

    Ok(response)
}
```

## Model Support

| Model | Thinking Mode Support |
|-------|----------------------|
| Qwen/Qwen2.5-72B-Instruct | ✅ Yes |
| Qwen/Qwen2.5-32B-Instruct | ✅ Yes |
| Qwen/Qwen2.5-7B-Instruct | ✅ Yes |
| DeepSeek-R1 | ✅ Yes (built-in) |
| Other models | ❌ Model dependent |

Check your vLLM server configuration to verify thinking mode support.

## Configuration Options

### Thinking Model Detection

The model automatically handles thinking tokens:

```rust
// Reasoning content is parsed from special tokens
// Usually structured as: <think>...</think> or similar
```

### Non-Streaming Access

For non-streaming requests with reasoning:

```rust
let response = client
    .chat
    .completions()
    .create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "Explain quantum entanglement"}
    ]))
    .extra(json!({
        "chat_template_kwargs": {
            "think_mode": true
        }
    }))
    .send()
    .await?;

// Access reasoning (if present)
if let Some(reasoning) = response.reasoning_content {
    println!("Reasoning:\n{}\n", reasoning);
}

// Access final answer
println!("Answer:\n{}", response.content.unwrap_or_default());
```

## Best Practices

### 1. Use for Complex Tasks

Thinking mode is most beneficial for:
- Multi-step reasoning
- Mathematical problems
- Code analysis
- Complex decision making

```rust
// Good: Complex reasoning task
.messages(json!([
    {"role": "user", "content": "Solve this puzzle: A father is 4 times as old as his son. In 20 years, he will be only twice as old. How old are they now?"}
]))

// Less beneficial: Simple query
.messages(json!([
    {"role": "user", "content": "What is 2 + 2?"}
]))
```

### 2. Display Reasoning Selectively

You may want to hide reasoning in production but show it for debugging:

```rust
let show_reasoning = std::env::var("SHOW_REASONING").is_ok();

while let Some(event) = stream.next().await {
    match event {
        StreamEvent::Reasoning(delta) => {
            if show_reasoning {
                eprintln!("[thinking] {}", delta);
            }
        }
        StreamEvent::Content(delta) => print!("{}", delta),
        _ => {}
    }
}
```

### 3. Combine with System Prompts

Guide the thinking process with system prompts:

```rust
.messages(json!([
    {
        "role": "system", 
        "content": "Think through problems step by step. Consider multiple approaches before settling on an answer."
    },
    {"role": "user", "content": problem}
]))
```

### 4. Adjust Max Tokens

Thinking mode uses more tokens. Adjust accordingly:

```rust
.max_tokens(4096)  // Account for both reasoning and answer
```

## Troubleshooting

### No Reasoning Content

If you don't see reasoning content:

1. Ensure thinking mode is enabled in `extra` parameters
2. Verify the model supports thinking mode
3. Check vLLM server configuration

```bash
# Check vLLM server logs for any issues
```

### Incomplete Streaming

If streaming seems incomplete:

```rust
// Ensure you handle all event types
while let Some(event) = stream.next().await {
    match event {
        StreamEvent::Reasoning(delta) => { /* handle */ },
        StreamEvent::Content(delta) => { /* handle */ },
        StreamEvent::Done => break,
        StreamEvent::Error(e) => {
            eprintln!("Error: {}", e);
            break;
        }
        _ => {}  // Don't forget other events
    }
}
```

## See Also

- [Streaming API](../api/streaming.md) - Streaming response documentation
- [Examples](../examples.md) - More usage examples
- [Advanced Topics](../advanced.md) - Other advanced features