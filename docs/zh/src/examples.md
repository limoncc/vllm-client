# 示例代码

本节包含各种使用场景的代码示例。

## 目录

- [基础聊天](#基础聊天)
- [流式聊天](#流式聊天)
- [工具调用](#工具调用)
- [多模态](#多模态)
- [思考模式](#思考模式)

## 基础聊天

### 简单对话

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
            {"role": "user", "content": "你好，请介绍一下你自己。"}
        ]))
        .send()
        .await?;
    
    println!("{}", response.choices[0].message.content.unwrap());
    Ok(())
}
```

### 带系统提示的对话

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
            {"role": "system", "content": "你是一个专业的 Rust 编程助手，回答简洁准确。"},
            {"role": "user", "content": "什么是所有权？"}
        ]))
        .temperature(0.7)
        .max_tokens(500)
        .send()
        .await?;
    
    println!("{}", response.choices[0].message.content.unwrap());
    Ok(())
}
```

### 多轮对话

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
            {"role": "user", "content": "我叫张三"},
            {"role": "assistant", "content": "你好，张三！很高兴认识你。有什么我可以帮助你的吗？"},
            {"role": "user", "content": "我叫什么名字？"}
        ]))
        .send()
        .await?;
    
    println!("{}", response.choices[0].message.content.unwrap());
    Ok(())
}
```

---

## 流式聊天

### 基本流式输出

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
            {"role": "user", "content": "写一首关于春天的诗"}
        ]))
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

### 带思考模式的流式输出

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
            {"role": "user", "content": "解释相对论"}
        ]))
        .extra(json!({"chat_template_kwargs": {"enable_thinking": true}}))
        .stream(true)
        .send_stream()
        .await?;
    
    println!("=== 思考过程 ===");
    while let Some(event) = stream.next().await {
        match &event {
            StreamEvent::Reasoning(delta) => {
                // 思考内容
                print!("{}", delta);
            }
            StreamEvent::Content(delta) => {
                // 正式回复内容
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

## 工具调用

### 定义和使用工具

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    // 定义工具
    let tools = json!([
        {
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "获取指定城市的当前天气",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "city": {
                            "type": "string",
                            "description": "城市名称，如：北京、上海"
                        }
                    },
                    "required": ["city"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "get_time",
                "description": "获取指定城市的当前时间",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "city": {
                            "type": "string",
                            "description": "城市名称"
                        }
                    },
                    "required": ["city"]
                }
            }
        }
    ]);
    
    // 发送请求
    let response = client
        .chat
        .completions()
        .create()
        .model("llama-3-70b")
        .messages(json!([
            {"role": "user", "content": "北京现在天气怎么样？"}
        ]))
        .tools(tools)
        .send()
        .await?;
    
    // 检查是否有工具调用
    if let Some(tool_calls) = &response.choices[0].message.tool_calls {
        for tool_call in tool_calls {
            println!("工具: {}", tool_call.function.name);
            println!("参数: {}", tool_call.function.arguments);
            
            // 在这里执行实际的工具调用
            // let result = execute_tool(&tool_call.function.name, &tool_call.function.arguments);
        }
    }
    
    Ok(())
}
```

### 返回工具结果

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
                "description": "获取天气信息",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "city": {"type": "string"}
                    },
                    "required": ["city"]
                }
            }
        }
    ]);
    
    // 模拟对话流程
    let response = client
        .chat
        .completions()
        .create()
        .model("llama-3-70b")
        .messages(json!([
            {"role": "user", "content": "上海天气如何？"},
            {
                "role": "assistant",
                "tool_calls": [{
                    "id": "call_001",
                    "type": "function",
                    "function": {
                        "name": "get_weather",
                        "arguments": "{\"city\": \"上海\"}"
                    }
                }]
            },
            {
                "role": "tool",
                "tool_call_id": "call_001",
                "content": "{\"temperature\": 28, \"condition\": \"多云\", \"humidity\": 65}"
            }
        ]))
        .tools(tools)
        .send()
        .await?;
    
    println!("{}", response.choices[0].message.content.unwrap());
    Ok(())
}
```

---

## 多模态

### 图像理解

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    // 使用 base64 编码的图像
    let image_base64 = "data:image/png;base64,iVBORw0KGgo...";
    
    let response = client
        .chat
        .completions()
        .create()
        .model("llava-v1.6")
        .messages(json!([
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "这张图片里有什么？"},
                    {
                        "type": "image_url",
                        "image_url": {"url": image_base64}
                    }
                ]
            }
        ]))
        .max_tokens(500)
        .send()
        .await?;
    
    println!("{}", response.choices[0].message.content.unwrap());
    Ok(())
}
```

### 使用图像 URL

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let response = client
        .chat
        .completions()
        .create()
        .model("llava-v1.6")
        .messages(json!([
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "描述这张图片"},
                    {
                        "type": "image_url",
                        "image_url": {"url": "https://example.com/image.jpg"}
                    }
                ]
            }
        ]))
        .send()
        .await?;
    
    println!("{}", response.choices[0].message.content.unwrap());
    Ok(())
}
```

---

## 思考模式

### 启用思考模式

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
            {"role": "system", "content": "你是一个善于深度思考的AI助手。"},
            {"role": "user", "content": "为什么天空是蓝色的？"}
        ]))
        .extra(json!({
            "chat_template_kwargs": {
                "enable_thinking": true
            }
        }))
        .stream(true)
        .send_stream()
        .await?;
    
    let mut reasoning = String::new();
    let mut content = String::new();
    
    while let Some(event) = stream.next().await {
        match &event {
            StreamEvent::Reasoning(delta) => reasoning.push_str(delta),
            StreamEvent::Content(delta) => content.push_str(delta),
            StreamEvent::Done => break,
            _ => {}
        }
    }
    
    println!("=== 思考过程 ===");
    println!("{}", reasoning);
    println!("\n=== 回答 ===");
    println!("{}", content);
    
    Ok(())
}
```

### 禁用思考模式

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");
    
    let response = client
        .chat
        .completions()
        .create()
        .model("qwen-3")
        .messages(json!([
            {"role": "user", "content": "你好"}
        ]))
        .extra(json!({
            "chat_template_kwargs": {
                "enable_thinking": false
            }
        }))
        .send()
        .await?;
    
    println!("{}", response.choices[0].message.content.unwrap());
    Ok(())
}
```

---

## 更多示例

完整的示例代码可以在项目的 `examples/` 目录中找到：

- `simple.rs` - 基础聊天示例
- `simple_streaming.rs` - 流式聊天示例
- `streaming_chat.rs` - 带思考模式的流式聊天
- `tool_calling.rs` - 工具调用示例