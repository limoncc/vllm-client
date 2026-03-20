# 工具调用 API

工具调用（也称为函数调用）允许模型在生成过程中调用外部函数，实现与外部 API、数据库和自定义逻辑的集成。

## 概述

vLLM Client 支持 OpenAI 兼容的工具调用：

```rust
use vllm_client::{VllmClient, json};

let client = VllmClient::new("http://localhost:8000/v1");

let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "东京的天气怎么样？"}
    ]))
    .tools(tools)
    .send()
    .await?;
```

## 定义工具

### 基础工具定义

工具使用遵循 OpenAI 规范的 JSON 格式定义：

```rust
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
                        "description": "城市名称，如东京"
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
```

### 多个工具

```rust
let tools = json!([
    {
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "获取天气信息",
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
            "description": "搜索网页信息",
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

## 工具选择

控制模型如何选择工具：

```rust
// 让模型自行决定（默认）
.tool_choice(json!("auto"))

// 禁止使用工具
.tool_choice(json!("none"))

// 强制使用工具
.tool_choice(json!("required"))

// 强制使用特定工具
.tool_choice(json!({
    "type": "function",
    "function": {"name": "get_weather"}
}))
```

## 处理工具调用

### 检查工具调用

```rust
use vllm_client::{VllmClient, json, VllmError};

let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "东京的天气怎么样？"}
    ]))
    .tools(tools)
    .send()
    .await?;

// 检查响应是否包含工具调用
if response.has_tool_calls() {
    if let Some(tool_calls) = &response.tool_calls {
        for tool_call in tool_calls {
            println!("函数: {}", tool_call.name);
            println!("参数: {}", tool_call.arguments);
        }
    }
}
```

### ToolCall 结构

```rust
pub struct ToolCall {
    pub id: String,           // 调用的唯一标识
    pub name: String,         // 函数名称
    pub arguments: String,    // 参数的 JSON 字符串
}
```

### 解析参数

将参数字符串解析为类型化数据：

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct WeatherArgs {
    location: String,
    unit: Option<String>,
}

if let Some(tool_call) = response.first_tool_call() {
    // 解析为特定类型
    match tool_call.parse_args_as::<WeatherArgs>() {
        Ok(args) => {
            println!("地点: {}", args.location);
            if let Some(unit) = args.unit {
                println!("单位: {}", unit);
            }
        }
        Err(e) => {
            eprintln!("解析参数失败: {}", e);
        }
    }
    
    // 或解析为通用 JSON
    let args: Value = tool_call.parse_args()?;
}
```

### 工具结果方法

创建工具结果消息：

```rust
// 创建工具结果消息
let tool_result = tool_call.result(json!({
    "temperature": 25,
    "condition": "sunny",
    "humidity": 60
}));

// 返回一个可直接加入消息的 JSON 对象
// {
//     "role": "tool",
//     "tool_call_id": "...",
//     "content": "{\"temperature\": 25, ...}"
// }
```

## 完整工具调用流程

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

// 模拟天气 API
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
                "description": "获取当前天气",
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

    // 第一次请求
    let response = client.chat.completions().create()
        .model("Qwen/Qwen2.5-72B-Instruct")
        .messages(json!([
            {"role": "user", "content": user_message}
        ]))
        .tools(tools.clone())
        .send()
        .await?;

    // 检查模型是否要调用工具
    if response.has_tool_calls() {
        let mut messages = vec![
            json!({"role": "user", "content": user_message})
        ];

        // 将助手的工具调用加入消息
        if let Some(tool_calls) = &response.tool_calls {
            let assistant_msg = response.assistant_message();
            messages.push(assistant_msg);

            // 执行每个工具并加入结果
            for tool_call in tool_calls {
                if tool_call.name == "get_weather" {
                    let args: WeatherArgs = tool_call.parse_args_as()?;
                    let result = get_weather(&args.location);
                    messages.push(tool_call.result(json!(result)));
                }
            }
        }

        // 带工具结果继续对话
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

## 流式工具调用

流式响应中，工具调用会增量推送：

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

let mut stream = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "东京和巴黎的天气怎么样？"}
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
            println!("[工具增量 {}] {}({})", index, name, arguments);
        }
        StreamEvent::ToolCallComplete(tool_call) => {
            println!("[工具完成] {}({})", tool_call.name, tool_call.arguments);
            tool_calls.push(tool_call);
        }
        StreamEvent::Done => break,
        _ => {}
    }
}

// 执行所有收集到的工具调用
for tool_call in tool_calls {
    // 执行并返回结果...
}
```

## 多轮工具调用

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
            // 加入带工具调用的助手消息
            messages.push(response.assistant_message());

            // 执行工具并加入结果
            if let Some(tool_calls) = &response.tool_calls {
                for tool_call in tool_calls {
                    let result = execute_tool(&tool_call.name, &tool_call.arguments);
                    messages.push(tool_call.result(result));
                }
            }
        } else {
            // 没有更多工具调用，返回内容
            return Ok(response.content.unwrap_or_default());
        }
    }

    Err("超过最大轮数".into())
}
```

## 最佳实践

### 清晰的工具描述

写清楚、详细的描述：

```rust
// 推荐
"description": "获取指定城市的当前天气状况。返回温度、湿度和天气状况。"

// 避免
"description": "获取天气"
```

### 精确的参数 Schema

定义准确的 JSON Schema：

```rust
"parameters": {
    "type": "object",
    "properties": {
        "location": {
            "type": "string",
            "description": "城市名称或坐标"
        },
        "days": {
            "type": "integer",
            "minimum": 1,
            "maximum": 7,
            "description": "预报天数"
        }
    },
    "required": ["location"]
}
```

### 错误处理

优雅地处理工具执行错误：

```rust
let tool_result = match execute_tool(&tool_call.name, &tool_call.arguments) {
    Ok(result) => json!({"success": true, "data": result}),
    Err(e) => json!({"success": false, "error": e.to_string()}),
};
messages.push(tool_call.result(tool_result));
```

## 相关链接

- [对话补全](./chat-completions.md) - 基础对话 API
- [流式响应](./streaming.md) - 流式响应处理
- [示例代码](../examples/tool-calling.md) - 更多工具调用示例