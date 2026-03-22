# Streaming Completions Example

This example demonstrates streaming completions using the legacy `/v1/completions` API.

## Basic Streaming Completions

```rust
use vllm_client::{VllmClient, json, CompletionStreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    println!("=== Streaming Completions ===\n");
    println!("Model: Qwen/Qwen2.5-7B-Instruct\n");
    println!("Prompt: What is machine learning?");
    println!("\nGenerated text: ");

    let mut stream = client
        .completions
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .prompt("What is machine learning?")
        .max_tokens(500)
        .temperature(0.7)
        .stream(true)
        .send_stream()
        .await?;

    // Process streaming events
    while let Some(event) = stream.next().await {
        match event {
            CompletionStreamEvent::Text(delta) => {
                // Print text delta (real-time output)
                print!("{}", delta);
                // Flush buffer for real-time display
                std::io::stdout().flush().ok();
            }
            CompletionStreamEvent::FinishReason(reason) => {
                println!("\n\n--- Finish reason: {} ---", reason);
            }
            CompletionStreamEvent::Usage(usage) => {
                // Output token usage statistics at the end
                println!("\n\n--- Token Usage ---");
                println!("Prompt tokens: {}", usage.prompt_tokens);
                println!("Completion tokens: {}", usage.completion_tokens);
                println!("Total tokens: {}", usage.total_tokens);
            }
            CompletionStreamEvent::Done => {
                println!("\n\n=== Generation Complete ===");
                break;
            }
            CompletionStreamEvent::Error(e) => {
                eprintln!("\nError: {}", e);
                return Err(e.into());
            }
        }
    }

    Ok(())
}
```

## Key Differences from Chat Streaming

| Aspect | Chat Completions | Completions |
|--------|-----------------|-------------|
| Event type | `StreamEvent` | `CompletionStreamEvent` |
| Content variant | `Content(String)` | `Text(String)` |
| Additional event | `Reasoning`, `ToolCall` | `FinishReason` |
| Use case | Conversation-based | Single prompt |

## When to Use Completions API

- Simple text generation with a single prompt
- Legacy compatibility with OpenAI API
- Situations where chat messages format is not needed

For new projects, we recommend using the Chat Completions API (`client.chat.completions()`) which provides more flexibility and better message formatting.

## Related Links

- [Streaming](./streaming-chat.md) - Chat streaming examples
- [API Streaming](../api/streaming.md) - Streaming API reference
- [Basic Chat](./basic-chat.md) - Non-streaming completions example
