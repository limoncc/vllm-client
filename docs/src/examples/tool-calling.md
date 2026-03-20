# Tool Calling Examples

This example demonstrates how to use tool calling (function calling) with vLLM Client.

## Basic Tool Calling

Define tools and let the model decide when to call them:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    // Define available tools
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
                            "description": "City name, e.g., Tokyo, New York"
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
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "What's the weather like in Tokyo?"}
        ]))
        .tools(tools)
        .send()
        .await?;

    // Check if the model wants to call a tool
    if response.has_tool_calls() {
        if let Some(tool_calls) = &response.tool_calls {
            for tool_call in tool_calls {
                println!("Function: {}", tool_call.name);
                println!("Arguments: {}", tool_call.arguments);
            }
        }
    } else {
        println!("Response: {}", response.content.unwrap_or_default());
    }

    Ok(())
}
```

## Complete Tool Calling Flow

Execute tools and return results to continue the conversation:

```rust
use vllm_client::{VllmClient, json, ToolCall};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct WeatherArgs {
    location: String,
    unit: Option<String>,
}

#[derive(Serialize)]
struct WeatherResult {
    temperature: f32,
    condition: String,
    humidity: u32,
}

// Simulated weather function
fn get_weather(location: &str, unit: Option<&str>) -> WeatherResult {
    // In real code, call an actual weather API
    let temp = match location {
        "Tokyo" => 25.0,
        "New York" => 20.0,
        "London" => 15.0,
        _ => 22.0,
    };

    WeatherResult {
        temperature: if unit == Some("fahrenheit") {
            temp * 9.0 / 5.0 + 32.0
        } else {
            temp
        },
        condition: "sunny".to_string(),
        humidity: 60,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let tools = json!([
        {
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get current weather for a location",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"},
                        "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}
                    },
                    "required": ["location"]
                }
            }
        }
    ]);

    let user_message = "What's the weather like in Tokyo and New York?";

    // First request - model may call tools
    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": user_message}
        ]))
        .tools(tools.clone())
        .send()
        .await?;

    if response.has_tool_calls() {
        // Build message history
        let mut messages = vec![
            json!({"role": "user", "content": user_message})
        ];

        // Add assistant's tool calls
        messages.push(response.assistant_message());

        // Execute each tool and add results
        if let Some(tool_calls) = &response.tool_calls {
            for tool_call in tool_calls {
                if tool_call.name == "get_weather" {
                    let args: WeatherArgs = tool_call.parse_args_as()?;
                    let result = get_weather(&args.location, args.unit.as_deref());
                    messages.push(tool_call.result(json!(result)));
                }
            }
        }

        // Continue conversation with tool results
        let final_response = client
            .chat
            .completions()
            .create()
            .model("Qwen/Qwen2.5-7B-Instruct")
            .messages(json!(messages))
            .tools(tools)
            .send()
            .await?;

        println!("{}", final_response.content.unwrap_or_default());
    } else {
        println!("{}", response.content.unwrap_or_default());
    }

    Ok(())
}
```

## Multiple Tools

Define multiple tools for different purposes:

```rust
use vllm_client::{VllmClient, json};
use serde::Deserialize;

#[derive(Deserialize)]
struct SearchArgs {
    query: String,
    limit: Option<u32>,
}

#[derive(Deserialize)]
struct CalcArgs {
    expression: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let tools = json!([
        {
            "type": "function",
            "function": {
                "name": "web_search",
                "description": "Search the web for information",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results"
                        }
                    },
                    "required": ["query"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "calculate",
                "description": "Perform mathematical calculations",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "Math expression to evaluate, e.g., '2 + 2 * 3'"
                        }
                    },
                    "required": ["expression"]
                }
            }
        }
    ]);

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "Search for Rust programming language and calculate 42 * 17"}
        ]))
        .tools(tools)
        .send()
        .await?;

    if let Some(tool_calls) = &response.tool_calls {
        for tool_call in tool_calls {
            match tool_call.name.as_str() {
                "web_search" => {
                    let args: SearchArgs = tool_call.parse_args_as()?;
                    println!("Searching for: {} (limit: {:?})", args.query, args.limit);
                }
                "calculate" => {
                    let args: CalcArgs = tool_call.parse_args_as()?;
                    println!("Calculating: {}", args.expression);
                }
                _ => println!("Unknown tool: {}", tool_call.name),
            }
        }
    }

    Ok(())
}
```

## Streaming Tool Calls

Stream tool call updates in real-time:

```rust
use vllm_client::{VllmClient, json, StreamEvent, ToolCall};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

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
            {"role": "user", "content": "What's the weather in Tokyo, Paris, and London?"}
        ]))
        .tools(tools)
        .stream(true)
        .send_stream()
        .await?;

    let mut tool_calls: Vec<ToolCall> = Vec::new();
    let mut content = String::new();

    println!("Streaming response:\n");

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Content(delta) => {
                content.push_str(&delta);
                print!("{}", delta);
            }
            StreamEvent::ToolCallDelta { index, id, name, arguments } => {
                println!("[Tool {}] {} - partial args: {}", index, name, arguments);
            }
            StreamEvent::ToolCallComplete(tool_call) => {
                println!("[Tool Complete] {}({})", tool_call.name, tool_call.arguments);
                tool_calls.push(tool_call);
            }
            StreamEvent::Done => {
                println!("\n--- Stream Complete ---");
                break;
            }
            StreamEvent::Error(e) => {
                eprintln!("\nError: {}", e);
                break;
            }
            _ => {}
        }
    }

    println!("\nCollected {} tool calls", tool_calls.len());
    for (i, tc) in tool_calls.iter().enumerate() {
        println!("  {}. {}({})", i + 1, tc.name, tc.arguments);
    }

    Ok(())
}
```

## Multi-Round Tool Calling

Handle multiple rounds of tool calls:

```rust
use vllm_client::{VllmClient, json, VllmError};
use serde_json::Value;

async fn run_agent(
    client: &VllmClient,
    user_message: &str,
    tools: &Value,
    max_rounds: usize,
) -> Result<String, VllmError> {
    let mut messages = vec![
        json!({"role": "user", "content": user_message})
    ];

    for round in 0..max_rounds {
        println!("--- Round {} ---", round + 1);

        let response = client
            .chat
            .completions()
            .create()
            .model("Qwen/Qwen2.5-7B-Instruct")
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
                    println!("Calling: {}({})", tool_call.name, tool_call.arguments);

                    // Execute the tool
                    let result = execute_tool(&tool_call.name, &tool_call.arguments);
                    println!("Result: {}", result);

                    // Add tool result to messages
                    messages.push(tool_call.result(result));
                }
            }
        } else {
            // No more tool calls, return the final response
            return Ok(response.content.unwrap_or_default());
        }
    }

    Err(VllmError::Other("Max rounds exceeded".to_string()))
}

fn execute_tool(name: &str, args: &str) -> Value {
    // Your tool execution logic here
    match name {
        "get_weather" => json!({"temperature": 22, "condition": "sunny"}),
        "web_search" => json!({"results": ["result1", "result2"]}),
        _ => json!({"error": "Unknown tool"}),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

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
        },
        {
            "type": "function",
            "function": {
                "name": "web_search",
                "description": "Search the web",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"}
                    },
                    "required": ["query"]
                }
            }
        }
    ]);

    let result = run_agent(
        &client,
        "What's the weather in Tokyo and find information about cherry blossoms?",
        &tools,
        5
    ).await?;

    println!("\nFinal Answer: {}", result);

    Ok(())
}
```

## Tool Choice Options

Control tool selection behavior:

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

    // Option 1: Let the model decide (default)
    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "Hello!"}
        ]))
        .tools(tools.clone())
        .tool_choice(json!("auto"))
        .send()
        .await?;

    // Option 2: Prevent tool use
    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "What's the weather in Tokyo?"}
        ]))
        .tools(tools.clone())
        .tool_choice(json!("none"))
        .send()
        .await?;

    // Option 3: Force tool use
    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "I need weather info"}
        ]))
        .tools(tools.clone())
        .tool_choice(json!("required"))
        .send()
        .await?;

    // Option 4: Force specific tool
    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "Check Tokyo weather"}
        ]))
        .tools(tools.clone())
        .tool_choice(json!({
            "type": "function",
            "function": {"name": "get_weather"}
        }))
        .send()
        .await?;

    Ok(())
}
```

## Error Handling

Handle tool execution errors gracefully:

```rust
use vllm_client::{VllmClient, json, ToolCall};
use serde_json::Value;

fn execute_tool_safely(tool_call: &ToolCall) -> Value {
    match tool_call.name.as_str() {
        "get_weather" => {
            // Parse arguments safely
            match tool_call.parse_args() {
                Ok(args) => {
                    // Execute tool
                    match get_weather_internal(&args) {
                        Ok(result) => json!({"success": true, "data": result}),
                        Err(e) => json!({"success": false, "error": e.to_string()}),
                    }
                }
                Err(e) => json!({
                    "success": false,
                    "error": format!("Invalid arguments: {}", e)
                }),
            }
        }
        _ => json!({
            "success": false,
            "error": format!("Unknown tool: {}", tool_call.name)
        }),
    }
}

fn get_weather_internal(args: &Value) -> Result<Value, String> {
    let location = args["location"].as_str()
        .ok_or("location is required")?;

    // Simulate API call
    Ok(json!({
        "location": location,
        "temperature": 22,
        "condition": "sunny"
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

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

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "What's the weather?"}
        ]))
        .tools(tools)
        .send()
        .await?;

    if let Some(tool_calls) = &response.tool_calls {
        for tool_call in tool_calls {
            let result = execute_tool_safely(tool_call);
            println!("Tool result: {}", result);
        }
    }

    Ok(())
}
```

## See Also

- [API: Tool Calling](../api/tool-calling.md) - Tool calling API reference
- [Streaming Chat](./streaming-chat.md) - Streaming responses
- [Basic Chat](./basic-chat.md) - Basic chat completion