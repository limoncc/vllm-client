# Multi-modal Examples

Multi-modal capabilities allow you to send images and other media types along with text to the model.

## Overview

vLLM supports multi-modal inputs through the OpenAI-compatible API. You can include images in your chat messages using base64 encoding or URLs.

## Basic Image Input (Base64)

Send an image encoded as base64:

```rust
use vllm_client::{VllmClient, json};
use base64::{Engine as _, engine::general_purpose};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    // Read and encode image
    let image_data = std::fs::read("image.png")?;
    let base64_image = general_purpose::STANDARD.encode(&image_data);

    let response = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2-VL-7B-Instruct")  // Vision model
        .messages(json!([
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "What's in this image?"
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

## Image from URL

Reference an image by URL:

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
                        "text": "Describe this image in detail."
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

## Helper Function for Images

Create a reusable helper for image messages:

```rust
use vllm_client::{VllmClient, json};
use serde_json::Value;

fn image_message(text: &str, image_path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    use base64::{Engine as _, engine::general_purpose};

    let image_data = std::fs::read(image_path)?;
    let base64_image = general_purpose::STANDARD.encode(&image_data);

    // Detect image type from extension
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

    let user_msg = image_message("What do you see in this image?", "photo.jpg")?;

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

## Multiple Images

Send multiple images in a single request:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    // Read and encode multiple images
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
                        "text": "Compare these two images. What are the differences?"
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

## Streaming with Images

Stream responses for image-based queries:

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
                        "text": "Analyze this chart and explain the trends."
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

## Multi-turn with Images

Maintain conversation context with images:

```rust
use vllm_client::{VllmClient, json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    let base64_image = encode_image("screenshot.png")?;

    // First message with image
    let messages = json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "What's in this screenshot?"},
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

    println!("First response: {}", response1.content.unwrap_or_default());

    // Continue conversation (no new image needed)
    let messages2 = json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "What's in this screenshot?"},
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
            "content": "Can you translate any text you see in the image?"
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

    println!("\nSecond response: {}", response2.content.unwrap_or_default());

    Ok(())
}
```

## OCR and Document Analysis

Use vision models for OCR and document analysis:

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
                "content": "You are an OCR assistant. Extract text from images accurately and format it properly."
            },
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "Extract all text from this document image. Preserve the formatting as much as possible."
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

    println!("Extracted Text:\n{}", response.content.unwrap_or_default());
    Ok(())
}
```

## Image Size Considerations

Handle large images appropriately:

```rust
use vllm_client::{VllmClient, json};

fn encode_and_resize_image(path: &str, max_size: u32) -> Result<String, Box<dyn std::error::Error>> {
    use base64::{Engine as _, engine::general_purpose};
    use image::ImageReader;

    // Load and resize image
    let img = ImageReader::open(path)?.decode()?;
    let img = img.resize(max_size, max_size, image::imageops::FilterType::Lanczos3);

    // Convert to PNG
    let mut buffer = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buffer, image::ImageFormat::Png)?;

    Ok(general_purpose::STANDARD.encode(&buffer.into_inner()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://localhost:8000/v1");

    // Resize to max 1024px while maintaining aspect ratio
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
                    {"type": "text", "text": "Describe this image."},
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

## Supported Models

For multi-modal inputs, use models that support vision:

| Model | Description |
|-------|-------------|
| `Qwen/Qwen2-VL-7B-Instruct` | Qwen2 Vision Language |
| `Qwen/Qwen2-VL-72B-Instruct` | Qwen2 VL Large |
| `meta-llama/Llama-3.2-11B-Vision-Instruct` | Llama 3.2 Vision |
| `openai/clip-vit-large-patch14` | CLIP model |

Check your vLLM server's available models with:

```bash
curl http://localhost:8000/v1/models
```

## Required Dependencies

For image handling, add these dependencies:

```toml
[dependencies]
vllm-client = "0.1"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
base64 = "0.22"
image = "0.25"  # Optional, for image processing
```

## Troubleshooting

### Image Too Large

If you get errors about image size, reduce the image dimensions:

```rust
// Resize before sending
let img = image::load_from_memory(&image_data)?;
let resized = img.resize(1024, 1024, image::imageops::FilterType::Lanczos3);
```

### Unsupported Format

Convert images to supported formats:

```rust
// Convert to PNG
let img = image::load_from_memory(&image_data)?;
let mut output = Vec::new();
img.write_to(&mut std::io::Cursor::new(&mut output), image::ImageFormat::Png)?;
```

### Model Doesn't Support Vision

Ensure you're using a vision-capable model. Non-vision models will ignore image inputs.

## See Also

- [Basic Chat](./basic-chat.md) - Text-only examples
- [Streaming Chat](./streaming-chat.md) - Streaming responses
- [API Reference](../api/chat-completions.md) - Complete API docs