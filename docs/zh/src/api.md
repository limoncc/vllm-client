# API 参考

本文档提供 vLLM Client API 的完整参考。

## 目录

- [客户端](#客户端)
- [聊天补全](#聊天补全)
- [流式响应](#流式响应)
- [工具调用](#工具调用)
- [类型定义](#类型定义)
- [错误处理](#错误处理)

## 客户端

### `VllmClient`

与 vLLM API 交互的主要客户端。

```rust
use vllm_client::VllmClient;

// 创建新客户端
let client = VllmClient::new("http://localhost:8000/v1");

// 带API密钥
let client = VllmClient::new("http://localhost:8000/v1")
    .with_api_key("your-api-key");

// 带自定义超时
let client = VllmClient::new("http://localhost:8000/v1")
    .with_timeout(std::time::Duration::from_secs(60));
```

#### 方法

| 方法 | 描述 |
|------|------|
| `new(base_url: &str)` | 使用给定的基础URL创建新客户端 |
| `with_api_key(key: &str)` | 设置用于认证的API密钥 |
| `with_timeout(duration)` | 设置请求超时时间 |
| `chat` | 访问聊天补全API |

---

## 聊天补全

### 创建补全请求

```rust
use vllm_client::{VllmClient, json};

let client = VllmClient::new("http://localhost:8000/v1");

let response = client
    .chat
    .completions()
    .create()
    .model("llama-3-70b")
    .messages(json!([
        {"role": "system", "content": "你是一个有帮助的助手。"},
        {"role": "user", "content": "你好！"}
    ]))
    .temperature(0.7)
    .max_tokens(1000)
    .send()
    .await?;
```

### 构建器方法

| 方法 | 类型 | 描述 |
|------|------|------|
| `model(name)` | `&str` | 使用的模型名称 |
| `messages(msgs)` | `Value` | 聊天消息数组 |
| `temperature(temp)` | `f32` | 采样温度 (0.0-2.0) |
| `max_tokens(tokens)` | `u32` | 最大生成token数 |
| `top_p(p)` | `f32` | 核采样参数 |
| `top_k(k)` | `u32` | Top-k采样参数 |
| `stream(enable)` | `bool` | 启用流式响应 |
| `tools(tools)` | `Value` | 函数调用的工具定义 |
| `extra(json)` | `Value` | 额外参数（厂商特定） |

### 响应结构

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

## 流式响应

### 流式补全

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
        {"role": "user", "content": "写一首诗"}
    ]))
    .stream(true)
    .send_stream()
    .await?;

while let Some(event) = stream.next().await {
    match &event {
        StreamEvent::Reasoning(delta) => {
            // 推理内容（用于思考模型）
            print!("{}", delta);
        }
        StreamEvent::Content(delta) => {
            // 常规内容
            print!("{}", delta);
        }
        StreamEvent::ToolCallDelta { tool_call_id, delta } => {
            // 工具调用流式更新
        }
        StreamEvent::ToolCallComplete(tool_call) => {
            // 完整的工具调用
        }
        StreamEvent::Usage(usage) => {
            // Token使用信息
        }
        StreamEvent::Done => {
            // 流式完成
            break;
        }
        StreamEvent::Error(e) => {
            eprintln!("错误: {}", e);
        }
    }
}
```

### StreamEvent 类型

| 变体 | 描述 |
|------|------|
| `Reasoning(String)` | 推理/思考内容 |
| `Content(String)` | 常规内容增量 |
| `ToolCallDelta { tool_call_id, delta }` | 流式工具调用 |
| `ToolCallComplete(ToolCall)` | 完整工具调用 |
| `Usage(Usage)` | Token使用统计 |
| `Done` | 流式结束 |
| `Error(VllmError)` | 发生错误 |

---

## 工具调用

### 定义工具

```rust
use vllm_client::json;

let tools = json!([
    {
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "获取指定位置的当前天气",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "城市名称"
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
        {"role": "user", "content": "东京的天气怎么样？"}
    ]))
    .tools(tools)
    .send()
    .await?;

// 处理工具调用
if let Some(tool_calls) = response.choices[0].message.tool_calls {
    for tool_call in tool_calls {
        println!("函数: {}", tool_call.function.name);
        println!("参数: {}", tool_call.function.arguments);
    }
}
```

### ToolCall 结构

```rust
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: FunctionCall,
}

pub struct FunctionCall {
    pub name: String,
    pub arguments: String, // JSON字符串
}
```

### 返回工具结果

```rust
// 执行工具后，返回结果
let response = client
    .chat
    .completions()
    .create()
    .model("llama-3-70b")
    .messages(json!([
        {"role": "user", "content": "东京的天气怎么样？"},
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

## 类型定义

### 消息类型

```rust
// 系统消息
json!({"role": "system", "content": "你是一个有帮助的助手。"})

// 用户消息
json!({"role": "user", "content": "你好！"})

// 助手消息
json!({"role": "assistant", "content": "你好！"})

// 工具结果消息
json!({
    "role": "tool",
    "tool_call_id": "call_123",
    "content": "结果"
})
```

### vLLM 特定参数

使用 `.extra()` 传递 vLLM 特定参数：

```rust
client
    .chat
    .completions()
    .create()
    .model("qwen-3")
    .messages(json!([{"role": "user", "content": "思考一下这个问题"}]))
    .extra(json!({
        "chat_template_kwargs": {
            "enable_thinking": true
        }
    }))
    .send()
    .await?;
```

---

## 错误处理

### VllmError

```rust
use vllm_client::VllmError;

match client.chat.completions().create().send().await {
    Ok(response) => { /* ... */ },
    Err(VllmError::HttpError(e)) => {
        eprintln!("HTTP错误: {}", e);
    }
    Err(VllmError::ApiError { message, code }) => {
        eprintln!("API错误 ({}): {}", code, message);
    }
    Err(VllmError::StreamError(e)) => {
        eprintln!("流式错误: {}", e);
    }
    Err(VllmError::ParseError(e)) => {
        eprintln!("解析错误: {}", e);
    }
    Err(e) => {
        eprintln!("其他错误: {}", e);
    }
}
```

### 错误类型

| 变体 | 描述 |
|------|------|
| `HttpError` | HTTP请求/响应错误 |
| `ApiError` | API级别错误（限流等） |
| `StreamError` | 流式特定错误 |
| `ParseError` | JSON解析错误 |
| `IoError` | I/O错误 |

---

## 完整示例

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1")
        .with_api_key("your-api-key");

    // 流式示例
    let mut stream = client
        .chat
        .completions()
        .create()
        .model("llama-3-70b")
        .messages(json!([
            {"role": "user", "content": "写一首关于编程的俳句"}
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
            StreamEvent::Error(e) => eprintln!("错误: {}", e),
            _ => {}
        }
    }

    println!();
    Ok(())
}
```
