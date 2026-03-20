# 工具调用示例

本示例演示如何在 vLLM Client 中使用工具调用（函数调用）。

## 基础工具调用

定义工具，让模型决定何时调用它们：

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    // 定义可用工具
    let tools = json!([
        {
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "获取指定地点的当前天气",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "城市名称，如：东京、纽约"
                        },
                        "unit": {
                            "type": "string",
                            "enum": ["celsius", "fahrenheit"],
                            "description": "温度单位"
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
            {"role": "user", "content": "东京的天气怎么样？"}
        ]))
        .tools(tools)
        .send()
        .await?;

    // 检查模型是否要调用工具
    if response.has_tool_calls() {
        if let Some(tool_calls) = &response.tool_calls {
            for tool_call in tool_calls {
                println!("函数: {}", tool_call.name);
                println!("参数: {}", tool_call.arguments);
            }
        }
    } else {
        println!("响应: {}", response.content.unwrap_or_default());
    }

    Ok(())
}
```

## 完整工具调用流程

执行工具并返回结果以继续对话：

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

// 模拟天气函数
fn get_weather(location: &str, unit: Option<&str>) -> WeatherResult {
    // 实际代码中，调用真实的天气 API
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
        condition: "晴朗".to_string(),
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
                "description": "获取指定地点的当前天气",
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

    let user_message = "东京和纽约的天气怎么样？";

    // 第一次请求 - 模型可能调用工具
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
        // 构建消息历史
        let mut messages = vec![
            json!({"role": "user", "content": user_message})
        ];

        // 添加助手的工具调用
        messages.push(response.assistant_message());

        // 执行每个工具并添加结果
        if let Some(tool_calls) = &response.tool_calls {
            for tool_call in tool_calls {
                if tool_call.name == "get_weather" {
                    let args: WeatherArgs = tool_call.parse_args_as()?;
                    let result = get_weather(&args.location, args.unit.as_deref());
                    messages.push(tool_call.result(json!(result)));
                }
            }
        }

        // 使用工具结果继续对话
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

## 多个工具

为不同目的定义多个工具：

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
                "description": "在网络上搜索信息",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "搜索查询"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "最大结果数"
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
                "description": "执行数学计算",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "要计算的数学表达式，如 '2 + 2 * 3'"
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
            {"role": "user", "content": "搜索 Rust 编程语言并计算 42 * 17"}
        ]))
        .tools(tools)
        .send()
        .await?;

    if let Some(tool_calls) = &response.tool_calls {
        for tool_call in tool_calls {
            match tool_call.name.as_str() {
                "web_search" => {
                    let args: SearchArgs = tool_call.parse_args_as()?;
                    println!("搜索: {} (限制: {:?})", args.query, args.limit);
                }
                "calculate" => {
                    let args: CalcArgs = tool_call.parse_args_as()?;
                    println!("计算: {}", args.expression);
                }
                _ => println!("未知工具: {}", tool_call.name),
            }
        }
    }

    Ok(())
}
```

## 流式工具调用

实时流式传输工具调用更新：

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
                "description": "获取指定地点的天气",
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
            {"role": "user", "content": "东京、巴黎和伦敦的天气怎么样？"}
        ]))
        .tools(tools)
        .stream(true)
        .send_stream()
        .await?;

    let mut tool_calls: Vec<ToolCall> = Vec::new();
    let mut content = String::new();

    println!("流式响应:\n");

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::Content(delta) => {
                content.push_str(&delta);
                print!("{}", delta);
            }
            StreamEvent::ToolCallDelta { index, id, name, arguments } => {
                println!("[工具 {}] {} - 部分参数: {}", index, name, arguments);
            }
            StreamEvent::ToolCallComplete(tool_call) => {
                println!("[工具完成] {}({})", tool_call.name, tool_call.arguments);
                tool_calls.push(tool_call);
            }
            StreamEvent::Done => {
                println!("\n--- 流式完成 ---");
                break;
            }
            StreamEvent::Error(e) => {
                eprintln!("\n错误: {}", e);
                break;
            }
            _ => {}
        }
    }

    println!("\n收集到 {} 个工具调用", tool_calls.len());
    for (i, tc) in tool_calls.iter().enumerate() {
        println!("  {}. {}({})", i + 1, tc.name, tc.arguments);
    }

    Ok(())
}
```

## 多轮工具调用

处理多轮工具调用：

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
        println!("--- 第 {} 轮 ---", round + 1);

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
            // 添加包含工具调用的助手消息
            messages.push(response.assistant_message());

            // 执行工具并添加结果
            if let Some(tool_calls) = &response.tool_calls {
                for tool_call in tool_calls {
                    println!("调用: {}({})", tool_call.name, tool_call.arguments);

                    // 执行工具
                    let result = execute_tool(&tool_call.name, &tool_call.arguments);
                    println!("结果: {}", result);

                    // 将工具结果添加到消息
                    messages.push(tool_call.result(result));
                }
            }
        } else {
            // 没有更多工具调用，返回最终响应
            return Ok(response.content.unwrap_or_default());
        }
    }

    Err(VllmError::Other("超过最大轮数".to_string()))
}

fn execute_tool(name: &str, args: &str) -> Value {
    // 在这里实现工具执行逻辑
    match name {
        "get_weather" => json!({"temperature": 22, "condition": "晴朗"}),
        "web_search" => json!({"results": ["结果1", "结果2"]}),
        _ => json!({"error": "未知工具"}),
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
                "description": "获取指定地点的天气",
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
                "description": "在网络上搜索",
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
        "东京的天气怎么样？并查找关于樱花的信息",
        &tools,
        5
    ).await?;

    println!("\n最终答案: {}", result);

    Ok(())
}
```

## 工具选择选项

控制工具选择行为：

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
                "description": "获取指定地点的天气",
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

    // 选项 1: 让模型决定（默认）
    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "你好！"}
        ]))
        .tools(tools.clone())
        .tool_choice(json!("auto"))
        .send()
        .await?;

    // 选项 2: 禁止工具使用
    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "东京的天气怎么样？"}
        ]))
        .tools(tools.clone())
        .tool_choice(json!("none"))
        .send()
        .await?;

    // 选项 3: 强制使用工具
    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "我需要天气信息"}
        ]))
        .tools(tools.clone())
        .tool_choice(json!("required"))
        .send()
        .await?;

    // 选项 4: 强制使用特定工具
    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([
            {"role": "user", "content": "查看东京天气"}
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

## 错误处理

优雅地处理工具执行错误：

```rust
use vllm_client::{VllmClient, json, ToolCall};
use serde_json::Value;

fn execute_tool_safely(tool_call: &ToolCall) -> Value {
    match tool_call.name.as_str() {
        "get_weather" => {
            // 安全地解析参数
            match tool_call.parse_args() {
                Ok(args) => {
                    // 执行工具
                    match get_weather_internal(&args) {
                        Ok(result) => json!({"success": true, "data": result}),
                        Err(e) => json!({"success": false, "error": e.to_string()}),
                    }
                }
                Err(e) => json!({
                    "success": false,
                    "error": format!("无效参数: {}", e)
                }),
            }
        }
        _ => json!({
            "success": false,
            "error": format!("未知工具: {}", tool_call.name)
        }),
    }
}

fn get_weather_internal(args: &Value) -> Result<Value, String> {
    let location = args["location"].as_str()
        .ok_or("location 是必需的")?;

    // 模拟 API 调用
    Ok(json!({
        "location": location,
        "temperature": 22,
        "condition": "晴朗"
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
                "description": "获取指定地点的天气",
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
            {"role": "user", "content": "天气怎么样？"}
        ]))
        .tools(tools)
        .send()
        .await?;

    if let Some(tool_calls) = &response.tool_calls {
        for tool_call in tool_calls {
            let result = execute_tool_safely(tool_call);
            println!("工具结果: {}", result);
        }
    }

    Ok(())
}
```

## 相关链接

- [API: 工具调用](../api/tool-calling.md) - 工具调用 API 参考
- [流式聊天](./streaming-chat.md) - 流式响应
- [基础聊天](./basic-chat.md) - 基础聊天补全