# Tool Calling API

Tool calling (also known as function calling) allows the model to call external functions during generation. This enables integration with external APIs, databases, and custom logic.

## Overview

The vLLM Client supports OpenAI-compatible tool calling:

```rust
use vllm_client::{VllmClient, json};

let client = VllmClient::new("http://localhost:8000/v1");

let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "What's the weather in Tokyo?"}
    ]))
    .tools(tools)
    .send()
    .await?;
```

## Defining Tools

### Basic Tool Definition

Tools are defined as JSON following the OpenAI schema:

```rust
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
                        "description": "The city name, e.g., Tokyo"
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
```

### Multiple Tools

```rust
let tools = json!([
    {
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "Get weather information",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                },
                "required": ["location"]
            }
        }
    },
    {
        "type": "function",
        "function": {
            "name": "search_web",
            "description": "Search the web for information",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "limit": {"type": "integer"}
                },
                "required": ["query"]
            }
        }
    }
]);
```

## Tool Choice

Control how the model selects tools:

```rust
// Let the model decide (default)
.tool_choice(json!("auto"))

// Prevent tool use
.tool_choice(json!("none"))

// Force tool use
.tool_choice(json!("required"))

// Force a specific tool
.tool_choice(json!({
    "type": "function",
    "function": {"name": "get_weather"}
}))
```

## Handling Tool Calls

### Checking for Tool Calls

```rust
use vllm_client::{VllmClient, json, VllmError};

let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "What's the weather in Tokyo?"}
    ]))
    .tools(tools)
    .send()
    .await?;

// Check if the response contains tool calls
if response.has_tool_calls() {
    if let Some(tool_calls) = &response.tool_calls {
        for tool_call in tool_calls {
            println!("Function: {}", tool_call.name);
            println!("Arguments: {}", tool_call.arguments);
        }
    }
}
```

### ToolCall Structure

```rust
pub struct ToolCall {
    pub id: String,           // Unique identifier for the call
    pub name: String,         // Function name
    pub arguments: String,    // JSON string of arguments
}
```

### Parsing Arguments

Parse the arguments string into typed data:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct WeatherArgs {
    location: String,
    unit: Option<String>,
}

if let Some(tool_call) = response.first_tool_call() {
    // Parse as a specific type
    match tool_call.parse_args_as::<WeatherArgs>() {
        Ok(args) => {
            println!("Location: {}", args.location);
            if let Some(unit) = args.unit {
                println!("Unit: {}", unit);
            }
        }
        Err(e) => {
            eprintln!("Failed to parse arguments: {}", e);
        }
    }
    
    // Or parse as generic JSON
    let args: Value = tool_call.parse_args()?;
}
```

### Tool Result Method

Create a tool result message:

```rust
// Create a tool result message
let tool_result = tool_call.result(json!({
    "temperature": 25,
    "condition": "sunny",
    "humidity": 60
}));

// Returns a JSON object ready to be added to messages
// {
//     "role": "tool",
//     "tool_call_id": "...",
//     "content": "{\"temperature\": 25, ...}"
// }
```

## Complete Tool Calling Flow

```rust
use vllm_client::{VllmClient, json, ToolCall};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct WeatherArgs {
    location: String,
}

#[derive(Serialize)]
struct WeatherResult {
    temperature: f32,
    condition: String,
}

// Simulate weather API
fn get_weather(location: &str) -> WeatherResult {
    WeatherResult {
        temperature: 25.0,
        condition: "sunny".to_string(),
    }
}

async fn chat_with_tools(client: &VllmClient, user_message: &str) -> Result<String, Box<dyn std::error::Error>> {
    let tools = json!([
        {
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get current weather",
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

    // First request
    let response = client.chat.completions().create()
        .model("Qwen/Qwen2.5-72B-Instruct")
        .messages(json!([
            {"role": "user", "content": user_message}
        ]))
        .tools(tools.clone())
        .send()
        .await?;

    // Check if model wants to call a tool
    if response.has_tool_calls() {
        let mut messages = vec![
            json!({"role": "user", "content": user_message})
        ];

        // Add assistant's tool calls to messages
        if let Some(tool_calls) = &response.tool_calls {
            let assistant_msg = response.assistant_message();
            messages.push(assistant_msg);

            // Execute each tool and add results
            for tool_call in tool_calls {
                if tool_call.name == "get_weather" {
                    let args: WeatherArgs = tool_call.parse_args_as()?;
                    let result = get_weather(&args.location);
                    messages.push(tool_call.result(json!(result)));
                }
            }
        }

        // Continue conversation with tool results
        let final_response = client.chat.completions().create()
            .model("Qwen/Qwen2.5-72B-Instruct")
            .messages(json!(messages))
            .tools(tools)
            .send()
            .await?;

        return Ok(final_response.content.unwrap_or_default());
    }

    Ok(response.content.unwrap_or_default())
}
```

## Streaming Tool Calls

Tool calls are streamed incrementally during streaming responses:

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

let mut stream = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "What's the weather in Tokyo and Paris?"}
    ]))
    .tools(tools)
    .stream(true)
    .send_stream()
    .await?;

let mut tool_calls: Vec<ToolCall> = Vec::new();
let mut content = String::new();

while let Some(event) = stream.next().await {
    match event {
        StreamEvent::Content(delta) => {
            content.push_str(&delta);
            print!("{}", delta);
        }
        StreamEvent::ToolCallDelta { index, id, name, arguments } => {
            println!("[Tool delta {}] {}({})", index, name, arguments);
        }
        StreamEvent::ToolCallComplete(tool_call) => {
            println!("[Tool complete] {}({})", tool_call.name, tool_call.arguments);
            tool_calls.push(tool_call);
        }
        StreamEvent::Done => break,
        _ => {}
    }
}

// Execute all collected tool calls
for tool_call in tool_calls {
    // Execute and return results...
}
```

## Tool Calling with Multiple Rounds

```rust
async fn multi_round_tool_calling(
    client: &VllmClient,
    user_message: &str,
    max_rounds: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut messages = vec![
        json!({"role": "user", "content": user_message})
    ];

    for _ in 0..max_rounds {
        let response = client.chat.completions().create()
            .model("Qwen/Qwen2.5-72B-Instruct")
            .messages(json!(&messages))
            .tools(tools.clone())
            .send()
            .await?;

        if response.has_tool_calls() {
            // Add assistant message with tool calls
            messages.push(response.assistant_message());

            // Execute tools and add results
            if let Some(tool_calls) = &response.tool_calls {
                for tool_call in tool_calls {
                    let result = execute_tool(&tool_call.name, &tool_call.arguments);
                    messages.push(tool_call.result(result));
                }
            }
        } else {
            // No more tool calls, return the content
            return Ok(response.content.unwrap_or_default());
        }
    }

    Err("Max rounds exceeded".into())
}
```

## Best Practices

### Clear Tool Descriptions

Write clear, detailed descriptions:

```rust
// Good
"description": "Get the current weather conditions for a specific city. Returns temperature, humidity, and weather condition."

// Avoid
"description": "Get weather"
```

### Precise Parameter Schemas

Define accurate JSON schemas:

```rust
"parameters": {
    "type": "object",
    "properties": {
        "location": {
            "type": "string",
            "description": "City name or coordinates"
        },
        "days": {
            "type": "integer",
            "minimum": 1,
            "maximum": 7,
            "description": "Number of days for forecast"
        }
    },
    "required": ["location"]
}
```

### Error Handling

Handle tool execution errors gracefully:

```rust
let tool_result = match execute_tool(&tool_call.name, &tool_call.arguments) {
    Ok(result) => json!({"success": true, "data": result}),
    Err(e) => json!({"success": false, "error": e.to_string()}),
};
messages.push(tool_call.result(tool_result));
```

## See Also

- [Chat Completions](./chat-completions.md) - Base chat API
- [Streaming](./streaming.md) - Streaming responses
- [Examples](../examples/tool-calling.md) - More tool calling examples