# 基础聊天示例

本页演示 vLLM Client 的基础聊天补全使用模式。

## 简单聊天

发送聊天消息的最简单方式：

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
            {"role": "user", "content": "你好，你好吗？"}
        ]))
        .send()
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

## 带系统消息

添加系统消息来控制助手的行为：

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
            {"role": "system", "content": "你是一个有帮助的编程助手。你编写整洁、文档完善的代码。"},
            {"role": "user", "content": "用 Rust 写一个检查数字是否为质数的函数"}
        ]))
        .send()
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

## 多轮对话

在多轮消息中保持上下文：

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    // 构建对话历史
    let mut messages = vec![
        json!({"role": "system", "content": "你是一个有帮助的助手。"}),
    ];

    // 第一轮
    messages.push(json!({"role": "user", "content": "我叫小明"}));
    
    let response1 = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!(messages.clone()))
        .send()
        .await?;

    let assistant_reply = response1.content.unwrap_or_default();
    println!("助手: {}", assistant_reply);

    // 将助手回复添加到历史
    messages.push(json!({"role": "assistant", "content": assistant_reply}));

    // 第二轮
    messages.push(json!({"role": "user", "content": "我叫什么名字？"}));

    let response2 = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!(messages))
        .send()
        .await?;

    println!("助手: {}", response2.content.unwrap_or_default());
    Ok(())
}
```

## 对话辅助工具

一个可复用的对话构建辅助工具：

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
                json!({"role": "system", "content": "你是一个有帮助的助手。"})
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
        .with_system("你是一个数学辅导员。简单地解释概念。");

    println!("用户: 2 + 2 等于几？");
    let reply = conv.send("2 + 2 等于几？").await?;
    println!("助手: {}", reply);

    println!("\n用户: 那乘以 3 等于几？");
    let reply = conv.send("那乘以 3 等于几？").await?;
    println!("助手: {}", reply);

    Ok(())
}
```

## 使用采样参数

通过采样参数控制生成：

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
            {"role": "user", "content": "写一个关于机器人的创意故事"}
        ]))
        .temperature(1.2)      // 更高的温度增加创意性
        .top_p(0.95)           // 核采样
        .top_k(50)             // vLLM 扩展参数
        .max_tokens(512)       // 限制输出长度
        .send()
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

## 确定性输出

要获得可重复的结果，将温度设置为 0：

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
            {"role": "user", "content": "2 + 2 等于几？"}
        ]))
        .temperature(0.0)      // 确定性输出
        .send()
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

## 使用停止序列

在特定序列处停止生成：

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
            {"role": "user", "content": "列出三种水果，每行一个"}
        ]))
        .stop(json!(["\n\n", "END"]))  // 在双换行或 END 处停止
        .send()
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

## Token 使用追踪

追踪 token 使用情况以监控成本：

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
            {"role": "user", "content": "解释量子计算"}
        ]))
        .send()
        .await?;

    println!("响应: {}", response.content.unwrap_or_default());

    if let Some(usage) = response.usage {
        println!("\n--- Token 使用统计 ---");
        println!("提示词 tokens: {}", usage.prompt_tokens);
        println!("补全 tokens: {}", usage.completion_tokens);
        println!("总 tokens: {}", usage.total_tokens);
    }

    Ok(())
}
```

## 批量处理

高效处理多个提示：

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
        "Rust 是什么？",
        "Python 是什么？",
        "Go 是什么？",
    ];

    let results = process_prompts(&client, &prompts).await;

    for (prompt, result) in prompts.iter().zip(results.iter()) {
        match result {
            Ok(response) => println!("问: {}\n答: {}\n", prompt, response),
            Err(e) => eprintln!("'{}' 出错: {}", prompt, e),
        }
    }

    Ok(())
}
```

## 错误处理

生产代码的正确错误处理：

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
        .map_err(|e| format!("请求失败: {}", e))?;

    response.content.ok_or_else(|| "响应中无内容".to_string())
}

#[tokio::main]
async fn main() {
    match safe_chat("你好！").await {
        Ok(text) => println!("响应: {}", text),
        Err(e) => eprintln!("错误: {}", e),
    }
}
```

## 相关链接

- [流式聊天](./streaming-chat.md) - 实时响应流
- [工具调用](./tool-calling.md) - 函数调用示例
- [API 参考](../api/chat-completions.md) - 完整 API 文档