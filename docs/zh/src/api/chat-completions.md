# 对话补全 API

对话补全 API 是生成文本响应的主要接口。

## 概述

通过 `client.chat.completions()` 访问对话补全 API：

```rust
use vllm_client::{VllmClient, json};

let client = VllmClient::new("http://localhost:8000/v1");

let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "你好！"}
    ]))
    .send()
    .await?;
```

## 请求构建器

### 必需参数

#### `model(name: impl Into<String>)`

设置生成使用的模型名称。

```rust
.model("Qwen/Qwen2.5-72B-Instruct")
// 或
.model("meta-llama/Llama-3-70b")
```

#### `messages(messages: Value)`

设置对话消息，格式为 JSON 数组。

```rust
.messages(json!([
    {"role": "system", "content": "你是一个有帮助的助手。"},
    {"role": "user", "content": "Rust 是什么？"}
]))
```

### 消息类型

| 角色 | 说明 |
|------|------|
| `system` | 设置助手行为 |
| `user` | 用户输入 |
| `assistant` | 助手回复（多轮对话时使用） |
| `tool` | 工具结果（函数调用时使用） |

### 采样参数

#### `temperature(temp: f32)`

控制随机性。范围：`0.0` 到 `2.0`。

```rust
.temperature(0.7)  // 常规行为
.temperature(0.0)  // 确定性输出
.temperature(1.5)  // 更有创意
```

#### `max_tokens(tokens: u32)`

最大生成 token 数。

```rust
.max_tokens(1024)
.max_tokens(4096)
```

#### `top_p(p: f32)`

核采样阈值。范围：`0.0` 到 `1.0`。

```rust
.top_p(0.9)
```

#### `top_k(k: i32)`

Top-K 采样（vLLM 扩展）。限制为 top K 个 token。

```rust
.top_k(50)
```

#### `stop(sequences: Value)`

遇到这些序列时停止生成。

```rust
// 多个停止序列
.stop(json!(["END", "STOP", "\n\n"]))

// 单个停止序列
.stop(json!("---"))
```

### 工具调用参数

#### `tools(tools: Value)`

定义模型可调用的工具/函数。

```rust
.tools(json!([
    {
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "获取某地的天气",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                },
                "required": ["location"]
            }
        }
    }
]))
```

#### `tool_choice(choice: Value)`

控制工具选择行为。

```rust
.tool_choice(json!("auto"))       // 模型决定
.tool_choice(json!("none"))       // 不使用工具
.tool_choice(json!("required"))   // 强制使用工具
.tool_choice(json!({
    "type": "function",
    "function": {"name": "get_weather"}
}))
```

### 高级参数

#### `stream(enable: bool)`

启用流式响应。

```rust
.stream(true)
```

#### `extra(params: Value)`

传入 vLLM 特有或其他额外参数。

```rust
.extra(json!({
    "chat_template_kwargs": {
        "think_mode": true
    },
    "reasoning_effort": "high"
}))
```

## 发送请求

### `send()` - 同步响应

一次性返回完整响应。

```rust
let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "你好！"}]))
    .send()
    .await?;
```

### `send_stream()` - 流式响应

返回流式数据，实现实时输出。

```rust
let mut stream = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([{"role": "user", "content": "你好！"}]))
    .stream(true)
    .send_stream()
    .await?;
```

详见[流式响应](./streaming.md)。

## 响应结构

### `ChatCompletionResponse`

| 字段 | 类型 | 说明 |
|-------|------|------|
| `raw` | `Value` | 原始 JSON 响应 |
| `id` | `String` | 响应 ID |
| `object` | `String` | 对象类型 |
| `model` | `String` | 使用的模型 |
| `content` | `Option<String>` | 生成的内容 |
| `reasoning_content` | `Option<String>` | 推理内容（思考模型） |
| `tool_calls` | `Option<Vec<ToolCall>>` | 工具调用 |
| `finish_reason` | `Option<String>` | 停止原因 |
| `usage` | `Option<Usage>` | Token 使用统计 |

### 使用示例

```rust
let response = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "2+2 等于几？"}
    ]))
    .send()
    .await?;

// 获取内容
println!("内容: {}", response.content.unwrap_or_default());

// 检查推理内容（思考模型）
if let Some(reasoning) = response.reasoning_content {
    println!("推理: {}", reasoning);
}

// 检查停止原因
match response.finish_reason.as_deref() {
    Some("stop") => println!("自然结束"),
    Some("length") => println!("达到最大 token 数"),
    Some("tool_calls") => println!("进行了工具调用"),
    _ => {}
}

// Token 使用统计
if let Some(usage) = response.usage {
    println!("提示词 tokens: {}", usage.prompt_tokens);
    println!("补全 tokens: {}", usage.completion_tokens);
    println!("总 tokens: {}", usage.total_tokens);
}
```

## 完整示例

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client.chat.completions().create()
        .model("Qwen/Qwen2.5-72B-Instruct")
        .messages(json!([
            {"role": "system", "content": "你是一个编程助手。"},
            {"role": "user", "content": "用 Rust 写一个反转字符串的函数"}
        ]))
        .temperature(0.7)
        .max_tokens(1024)
        .top_p(0.9)
        .send()
        .await?;

    if let Some(content) = response.content {
        println!("{}", content);
    }

    Ok(())
}
```

## 多轮对话

```rust
use vllm_client::{VllmClient, json};

let client = VllmClient::new("http://localhost:8000/v1");

// 第一轮
let response1 = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "我叫小明"}
    ]))
    .send()
    .await?;

// 继续对话
let response2 = client.chat.completions().create()
    .model("Qwen/Qwen2.5-72B-Instruct")
    .messages(json!([
        {"role": "user", "content": "我叫小明"},
        {"role": "assistant", "content": response1.content.unwrap()},
        {"role": "user", "content": "我叫什么名字？"}
    ]))
    .send()
    .await?;
```

## 相关链接

- [流式响应](./streaming.md) - 流式响应处理
- [工具调用](./tool-calling.md) - 函数调用
- [客户端](./client.md) - 客户端配置