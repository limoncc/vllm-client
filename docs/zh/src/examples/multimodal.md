# 多模态示例

多模态功能允许您将图像和其他媒体类型与文本一起发送给模型。

## 概述

vLLM 通过 OpenAI 兼容的 API 支持多模态输入。您可以使用 base64 编码或 URL 在聊天消息中包含图像。

## 基础图像输入（Base64）

发送 base64 编码的图像：

```rust
use vllm_client::{VllmClient, json};
use base64::{Engine as _, engine::general_purpose};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    // 读取并编码图像
    let image_data = std::fs::read("image.png")?;
    let base64_image = general_purpose::STANDARD.encode(&image_data);

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2-VL-7B-Instruct")  // 视觉模型
        .messages(json!([
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "这张图片里有什么？"
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", base64_image)
                        }
                    }
                ]
            }
        ]))
        .max_tokens(512)
        .send()
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

## 使用 URL 引用图像

通过 URL 引用图像：

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2-VL-7B-Instruct")
        .messages(json!([
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "详细描述这张图片。"
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": "https://example.com/image.jpg"
                        }
                    }
                ]
            }
        ]))
        .send()
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

## 图像消息辅助函数

创建可复用的图像消息辅助函数：

```rust
use vllm_client::{VllmClient, json};
use serde_json::Value;

fn image_message(text: &str, image_path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    use base64::{Engine as _, engine::general_purpose};

    let image_data = std::fs::read(image_path)?;
    let base64_image = general_purpose::STANDARD.encode(&image_data);

    // 根据扩展名检测图像类型
    let mime_type = match image_path.to_lowercase().rsplit('.').next() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/png",
    };

    Ok(json!({
        "role": "user",
        "content": [
            {
                "type": "text",
                "text": text
            },
            {
                "type": "image_url",
                "image_url": {
                    "url": format!("data:{};base64,{}", mime_type, base64_image)
                }
            }
        ]
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let user_msg = image_message("这张图片里有什么？", "photo.jpg")?;

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2-VL-7B-Instruct")
        .messages(json!([user_msg]))
        .max_tokens(1024)
        .send()
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

## 多图像处理

在单个请求中发送多张图像：

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    // 读取并编码多张图像
    let image1 = encode_image("image1.png")?;
    let image2 = encode_image("image2.png")?;

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2-VL-7B-Instruct")
        .messages(json!([
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "比较这两张图片。它们有什么不同？"
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", image1)
                        }
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", image2)
                        }
                    }
                ]
            }
        ]))
        .max_tokens(1024)
        .send()
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}

fn encode_image(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    use base64::{Engine as _, engine::general_purpose};
    let data = std::fs::read(path)?;
    Ok(general_purpose::STANDARD.encode(&data))
}
```

## 带图像的流式响应

对图像查询进行流式响应：

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let base64_image = encode_image("chart.png")?;

    let mut stream = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2-VL-7B-Instruct")
        .messages(json!([
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "分析这个图表并解释趋势。"
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", base64_image)
                        }
                    }
                ]
            }
        ]))
        .stream(true)
        .send_stream()
        .await?;

    while let Some(event) = stream.next().await {
        if let StreamEvent::Content(delta) = event {
            print!("{}", delta);
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
    }

    println!();
    Ok(())
}
```

## 带图像的多轮对话

在对话中保持图像上下文：

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let base64_image = encode_image("screenshot.png")?;

    // 第一条带图像的消息
    let messages = json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "这个截图里有什么？"},
                {
                    "type": "image_url",
                    "image_url": {
                        "url": format!("data:image/png;base64,{}", base64_image)
                    }
                }
            ]
        }
    ]);

    let response1 = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2-VL-7B-Instruct")
        .messages(messages.clone())
        .send()
        .await?;

    println!("第一次响应: {}", response1.content.unwrap_or_default());

    // 继续对话（不需要新图像）
    let messages2 = json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "这个截图里有什么？"},
                {
                    "type": "image_url",
                    "image_url": {
                        "url": format!("data:image/png;base64,{}", base64_image)
                    }
                }
            ]
        },
        {
            "role": "assistant",
            "content": response1.content.unwrap_or_default()
        },
        {
            "role": "user",
            "content": "你能翻译图片中的文本吗？"
        }
    ]);

    let response2 = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2-VL-7B-Instruct")
        .messages(messages2)
        .send()
        .await?;

    println!("\n第二次响应: {}", response2.content.unwrap_or_default());

    Ok(())
}
```

## OCR 和文档分析

使用视觉模型进行 OCR 和文档分析：

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let document_image = encode_image("document.png")?;

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2-VL-7B-Instruct")
        .messages(json!([
            {
                "role": "system",
                "content": "你是一个 OCR 助手。准确提取图像中的文本并正确格式化。"
            },
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "从这个文档图像中提取所有文本。尽可能保留格式。"
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", document_image)
                        }
                    }
                ]
            }
        ]))
        .max_tokens(2048)
        .send()
        .await?;

    println!("提取的文本:\n{}", response.content.unwrap_or_default());
    Ok(())
}
```

## 图像大小考虑

正确处理大图像：

```rust
use vllm_client::{VllmClient, json};

fn encode_and_resize_image(path: &str, max_size: u32) -> Result<String, Box<dyn std::error::Error>> {
    use base64::{Engine as _, engine::general_purpose};
    use image::ImageReader;

    // 加载并调整图像大小
    let img = ImageReader::open(path)?.decode()?;
    let img = img.resize(max_size, max_size, image::imageops::FilterType::Lanczos3);

    // 转换为 PNG
    let mut buffer = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buffer, image::ImageFormat::Png)?;

    Ok(general_purpose::STANDARD.encode(&buffer.into_inner()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    // 调整大小到最大 1024px，保持宽高比
    let base64_image = encode_and_resize_image("large_image.jpg", 1024)?;

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2-VL-7B-Instruct")
        .messages(json!([
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "描述这张图片。"},
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", base64_image)
                        }
                    }
                ]
            }
        ]))
        .send()
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

## 支持的模型

对于多模态输入，请使用支持视觉的模型：

| 模型 | 描述 |
|------|------|
| `Qwen/Qwen2-VL-7B-Instruct` | Qwen2 视觉语言模型 |
| `Qwen/Qwen2-VL-72B-Instruct` | Qwen2 视觉语言大模型 |
| `meta-llama/Llama-3.2-11B-Vision-Instruct` | Llama 3.2 视觉模型 |
| `openai/clip-vit-large-patch14` | CLIP 模型 |

使用以下命令检查 vLLM 服务器的可用模型：

```bash
curl http://localhost:8000/v1/models
```

## 必需的依赖

对于图像处理，添加以下依赖：

```toml
[dependencies]
vllm-client = "0.1"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
base64 = "0.22"
image = "0.25"  # 可选，用于图像处理
```

## 故障排除

### 图像过大

如果遇到图像大小错误，请减小图像尺寸：

```rust
// 发送前调整大小
let img = image::load_from_memory(&image_data)?;
let resized = img.resize(1024, 1024, image::imageops::FilterType::Lanczos3);
```

### 不支持的格式

将图像转换为支持的格式：

```rust
// 转换为 PNG
let img = image::load_from_memory(&image_data)?;
let mut output = Vec::new();
img.write_to(&mut std::io::Cursor::new(&mut output), image::ImageFormat::Png)?;
```

### 模型不支持视觉

确保使用支持视觉的模型。非视觉模型会忽略图像输入。

## 相关链接

- [基础聊天](./basic-chat.md) - 纯文本示例
- [流式聊天](./streaming-chat.md) - 流式响应
- [API 参考](../api/chat-completions.md) - 完整 API 文档