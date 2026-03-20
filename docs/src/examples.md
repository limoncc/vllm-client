# Examples

This section contains various code examples for using the vLLM Client.

## Table of Contents

- [Basic Chat](#basic-chat)
- [Streaming Chat](#streaming-chat)
- [With System Prompt](#with-system-prompt)
- [Multiple Turns](#multiple-turns)
- [Tool Calling](#tool-calling)
- [Thinking Mode](#thinking-mode)
- [Custom Parameters](#custom-parameters)
- [Error Handling](#error-handling)

---

## Basic Chat

A simple chat completion request:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let response = client
        .chat
        .completions()
        .create()
        .model("llama-3-70b")
        .messages(json!([
            {"role": "user", "content": "Hello, how are you?"}
        ]))
        .send()
        .await?;
    
    println!("{}", response.choices[0].message.content.as_ref().unwrap());
    Ok(())
}
```

---

## Streaming Chat

Stream the response token by token:

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
        .model("llama-3-70b")
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

---

## With System Prompt

Include a system prompt to set the assistant's behavior:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let response = client
        .chat
        .completions()
        .create()
        .model("llama-3-70b")
        .messages(json!([
            {"role": "system", "content": "You are a helpful coding assistant. Be concise and provide code examples when appropriate."},
            {"role": "user", "content": "How do I read a file in Rust?"}
        ]))
        .send()
        .await?;
    
    println!("{}", response.choices[0].message.content.as_ref().unwrap());
    Ok(())
}
```

---

## Multiple Turns

Maintain a conversation history:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let messages = json!([
        {"role": "user", "content": "My name is Alice."},
        {"role": "assistant", "content": "Hello Alice! Nice to meet you. How can I help you today?"},
        {"role": "user", "content": "What's my name?"}
    ]);
    
    let response = client
        .chat
        .completions()
        .create()
        .model("llama-3-70b")
        .messages(messages)
        .send()
        .await?;
    
    println!("{}", response.choices[0].message.content.as_ref().unwrap());
    Ok(())
}
```

---

## Tool Calling

Define and use tools for function calling:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let tools = json!([
        {
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get the current weather for a location",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "City name, e.g., Tokyo"
                        },
                        "unit": {
                            "type": "string",
                            "enum": ["celsius", "fahrenheit"],
                            "description": "Temperature unit"
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
            {"role": "user", "content": "What's the weather like in Tokyo?"}
        ]))
        .tools(tools)
        .send()
        .await?;
    
    if let Some(tool_calls) = &response.choices[0].message.tool_calls {
        for tool_call in tool_calls {
            println!("Function: {}", tool_call.function.name);
            println!("Arguments: {}", tool_call.function.arguments);
            
            // Execute the function and get the result
            let result = execute_weather_function(&tool_call.function.arguments);
            
            // Return the result back to the model (next turn)
            // ...
        }
    }
    
    Ok(())
}

fn execute_weather_function(args: &str) -> String {
    // Your function implementation here
    "{\"temperature\": 22, \"condition\": \"sunny\"}".to_string()
}
```

---

## Thinking Mode

For models that support reasoning/thinking (like Qwen with thinking enabled):

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
        .model("qwen-3")
        .messages(json!([
            {"role": "user", "content": "Solve: What is 15 * 23 + 42?"}
        ]))
        .extra(json!({
            "chat_template_kwargs": {
                "enable_thinking": true
            }
        }))
        .stream(true)
        .send_stream()
        .await?;
    
    print!("Thinking: ");
    while let Some(event) = stream.next().await {
        match &event {
            StreamEvent::Reasoning(delta) => {
                // Reasoning/thinking content
                print!("{}", delta);
            }
            StreamEvent::Content(delta) => {
                // Final answer
                print!("{}", delta);
            }
            StreamEvent::Done => break,
            _ => {}
        }
    }
    
    println!();
    Ok(())
}
```

---

## Custom Parameters

Use various generation parameters:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let response = client
        .chat
        .completions()
        .create()
        .model("llama-3-70b")
        .messages(json!([
            {"role": "user", "content": "Generate a creative story opening."}
        ]))
        .temperature(0.9)           // Higher = more creative
        .top_p(0.95)                // Nucleus sampling
        .top_k(50)                  // Top-k sampling
        .max_tokens(500)            // Maximum response length
        .extra(json!({
            "repetition_penalty": 1.1,
            "frequency_penalty": 0.5
        }))
        .send()
        .await?;
    
    println!("{}", response.choices[0].message.content.as_ref().unwrap());
    
    // Print token usage
    println!("\n--- Token Usage ---");
    println!("Prompt tokens: {}", response.usage.prompt_tokens);
    println!("Completion tokens: {}", response.usage.completion_tokens);
    println!("Total tokens: {}", response.usage.total_tokens);
    
    Ok(())
}
```

---

## Error Handling

Handle various error scenarios:

```rust
use vllm_client::{VllmClient, json, VllmError};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1")
        .with_timeout(Duration::from_secs(30));
    
    match client
        .chat
        .completions()
        .create()
        .model("llama-3-70b")
        .messages(json!([
            {"role": "user", "content": "Hello"}
        ]))
        .send()
        .await
    {
        Ok(response) => {
            println!("Response: {}", response.choices[0].message.content.as_ref().unwrap());
        }
        Err(VllmError::HttpError(e)) => {
            eprintln!("HTTP error (connection issue?): {}", e);
        }
        Err(VllmError::ApiError { message, code }) => {
            eprintln!("API error (code {:?}): {}", code, message);
        }
        Err(VllmError::ParseError(e)) => {
            eprintln!("Failed to parse response: {}", e);
        }
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
        }
    }
    
    Ok(())
}
```

---

## Complete Example with Retry Logic

A more robust example with retry logic:

```rust
use vllm_client::{VllmClient, json, VllmError};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1")
        .with_timeout(Duration::from_secs(60));
    
    let mut retries = 0;
    let max_retries = 3;
    
    loop {
        match client
            .chat
            .completions()
            .create()
            .model("llama-3-70b")
            .messages(json!([
                {"role": "user", "content": "Hello"}
            ]))
            .send()
            .await
        {
            Ok(response) => {
                println!("{}", response.choices[0].message.content.as_ref().unwrap());
                break;
            }
            Err(VllmError::HttpError(_)) if retries < max_retries => {
                retries += 1;
                println!("Request failed, retrying... ({}/{})", retries, max_retries);
                sleep(Duration::from_secs(2_u64.pow(retries))).await;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(e.into());
            }
        }
    }
    
    Ok(())
}
```

---

## More Examples

See the `examples/` directory in the repository for complete runnable examples:

- `simple.rs` - Basic chat completion
- `simple_streaming.rs` - Streaming chat
- `streaming_chat.rs` - Streaming with thinking mode
- `tool_calling.rs` - Tool calling example