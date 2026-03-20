# Basic Chat Examples

This page demonstrates basic chat completion usage patterns with vLLM Client.

## Simple Chat

The simplest way to send a chat message:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "Hello, how are you?"}
        ]))
        .send()
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

## With System Message

Add a system message to control the assistant's behavior:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "system", "content": "You are a helpful coding assistant. You write clean, well-documented code."},
            {"role": "user", "content": "Write a function to check if a number is prime in Rust"}
        ]))
        .send()
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

## Multi-turn Conversation

Maintain context across multiple messages:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    // Build conversation history
    let mut messages = vec![
        json!({"role": "system", "content": "You are a helpful assistant."}),
    ];

    // First turn
    messages.push(json!({"role": "user", "content": "My name is Alice"}));
    
    let response1 = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!(messages.clone()))
        .send()
        .await?;

    let assistant_reply = response1.content.unwrap_or_default();
    println!("Assistant: {}", assistant_reply);

    // Add assistant reply to history
    messages.push(json!({"role": "assistant", "content": assistant_reply}));

    // Second turn
    messages.push(json!({"role": "user", "content": "What's my name?"}));

    let response2 = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!(messages))
        .send()
        .await?;

    println!("Assistant: {}", response2.content.unwrap_or_default());
    Ok(())
}
```

## Conversation Helper

A reusable helper for building conversations:

```rust
use vllm_client::{VllmClient, json, VllmError};
use serde_json::Value;

struct Conversation {
    client: VllmClient,
    model: String,
    messages: Vec<Value>,
}

impl Conversation {
    fn new(client: VllmClient, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
            messages: vec![
                json!({"role": "system", "content": "You are a helpful assistant."})
            ],
        }
    }

    fn with_system(mut self, content: &str) -> Self {
        self.messages[0] = json!({"role": "system", "content": content});
        self
    }

    async fn send(&mut self, user_message: &str) -> Result<String, VllmError> {
        self.messages.push(json!({
            "role": "user",
            "content": user_message
        }));

        let response = self.client
            .chat
            .completions()
            .create()
            .model(&self.model)
            .messages(json!(&self.messages))
            .send()
            .await?;

        let content = response.content.unwrap_or_default();
        self.messages.push(json!({
            "role": "assistant",
            "content": &content
        }));

        Ok(content)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let mut conv = Conversation::new(client, "Qwen/Qwen2.5-7B-Instruct")
        .with_system("You are a math tutor. Explain concepts simply.");

    println!("User: What is 2 + 2?");
    let reply = conv.send("What is 2 + 2?").await?;
    println!("Assistant: {}", reply);

    println!("\nUser: And what is that multiplied by 3?");
    let reply = conv.send("And what is that multiplied by 3?").await?;
    println!("Assistant: {}", reply);

    Ok(())
}
```

## With Sampling Parameters

Control the generation with sampling parameters:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "Write a creative story about a robot"}
        ]))
        .temperature(1.2)      // Higher temperature for more creativity
        .top_p(0.95)           // Nucleus sampling
        .top_k(50)             // vLLM extension
        .max_tokens(512)       // Limit output length
        .send()
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

## Deterministic Output

For reproducible results, set temperature to 0:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "What is 2 + 2?"}
        ]))
        .temperature(0.0)      // Deterministic output
        .send()
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

## With Stop Sequences

Stop generation at specific sequences:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "List three fruits, one per line"}
        ]))
        .stop(json!(["\n\n", "END"]))  // Stop at double newline or END
        .send()
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

## Token Usage Tracking

Track token usage for cost monitoring:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "Explain quantum computing"}
        ]))
        .send()
        .await?;

    println!("Response: {}", response.content.unwrap_or_default());

    if let Some(usage) = response.usage {
        println!("\n--- Token Usage ---");
        println!("Prompt tokens: {}", usage.prompt_tokens);
        println!("Completion tokens: {}", usage.completion_tokens);
        println!("Total tokens: {}", usage.total_tokens);
    }

    Ok(())
}
```

## Batch Processing

Process multiple prompts efficiently:

```rust
use vllm_client::{VllmClient, json, VllmError};

async fn process_prompts(
    client: &VllmClient,
    prompts: &[&str],
) -> Vec<Result<String, VllmError>> {
    let mut results = Vec::new();

    for prompt in prompts {
        let result = client
            .chat
            .completions()
            .create()
            .model("Qwen/Qwen2.5-7B-Instruct")
            .messages(json!([{"role": "user", "content": prompt}]))
            .send()
            .await
            .map(|r| r.content.unwrap_or_default());

        results.push(result);
    }

    results
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1")
        .timeout_secs(120);

    let prompts = [
        "What is Rust?",
        "What is Python?",
        "What is Go?",
    ];

    let results = process_prompts(&client, &prompts).await;

    for (prompt, result) in prompts.iter().zip(results.iter()) {
        match result {
            Ok(response) => println!("Q: {}\nA: {}\n", prompt, response),
            Err(e) => eprintln!("Error for '{}': {}", prompt, e),
        }
    }

    Ok(())
}
```

## Error Handling

Proper error handling for production code:

```rust
use vllm_client::{VllmClient, json, VllmError};

async fn safe_chat(prompt: &str) -> Result<String, String> {
    let client = VllmClient::new("http://localhost:8000/v1")
        .timeout_secs(60);

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": prompt}]))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    response.content.ok_or_else(|| "No content in response".to_string())
}

#[tokio::main]
async fn main() {
    match safe_chat("Hello!").await {
        Ok(text) => println!("Response: {}", text),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## See Also

- [Streaming Chat](./streaming-chat.md) - Real-time response streaming
- [Tool Calling](./tool-calling.md) - Function calling examples
- [API Reference](../api/chat-completions.md) - Complete API documentation