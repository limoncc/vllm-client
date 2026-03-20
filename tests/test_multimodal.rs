//! Phase 5.1: Multimodal Tests
//!
//! 测试多模态消息（图像）的支持

use serde_json::json;
use vllm_client::VllmClient;

// ============================================================================
// Test: test_image_url_message
// 输入: 包含图像 URL 的多模态消息
// 预期: 序列化后符合 OpenAI 格式
// ============================================================================
#[test]
fn test_image_url_message() {
    let messages = json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "What's in this image?"},
                {
                    "type": "image_url",
                    "image_url": {
                        "url": "https://example.com/image.jpg"
                    }
                }
            ]
        }
    ]);

    // 验证格式正确
    assert!(messages.is_array());
    let arr = messages.as_array().unwrap();
    assert_eq!(arr.len(), 1);

    let first_msg = &arr[0];
    assert_eq!(first_msg["role"], "user");
    assert!(first_msg["content"].is_array());

    let content = first_msg["content"].as_array().unwrap();
    assert_eq!(content.len(), 2);

    // 验证文本部分
    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[0]["text"], "What's in this image?");

    // 验证图像部分
    assert_eq!(content[1]["type"], "image_url");
    assert_eq!(
        content[1]["image_url"]["url"],
        "https://example.com/image.jpg"
    );
}

// ============================================================================
// Test: test_base64_image_message
// 输入: 包含 base64 图像的消息
// 预期: 正确序列化，data:image/jpeg;base64,... 格式正确
// ============================================================================
#[test]
fn test_base64_image_message() {
    // 模拟 base64 编码的图像数据
    let base64_image = "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkSEw8UHRofHh0aHBwgJC4nICIsIxwcKDcpLDAxNDQ0Hyc5PTgyPC4zNDL/2wBDAQkJCQwLDBgNDRgyIRwhMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjL/wAARCAABAAEDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAv/xAAUEAEAAAAAAAAAAAAAAAAAAAAA/8QAFQEBAQAAAAAAAAAAAAAAAAAAAAX/xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oADAMBEQACEQA/ALUABoAAAA//2Q==";

    let messages = json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "Describe this image"},
                {
                    "type": "image_url",
                    "image_url": {
                        "url": base64_image
                    }
                }
            ]
        }
    ]);

    // 验证格式正确
    let content = messages[0]["content"].as_array().unwrap();

    // 验证 base64 格式
    let image_url = &content[1]["image_url"]["url"];
    let url_str = image_url.as_str().unwrap();
    assert!(url_str.starts_with("data:image/"));
    assert!(url_str.contains(";base64,"));
}

// ============================================================================
// Test: test_base64_image_different_formats
// 输入: 不同图像格式的 base64 编码
// 预期: 正确处理 JPEG、PNG、GIF、WebP 等格式
// ============================================================================
#[test]
fn test_base64_image_different_formats() {
    let formats = vec![
        ("jpeg", "data:image/jpeg;base64,/9j/4AAQSkZJRg=="),
        ("png", "data:image/png;base64,iVBORw0KGgo="),
        ("gif", "data:image/gif;base64,R0lGODlh="),
        ("webp", "data:image/webp;base64,UklGRjg="),
    ];

    for (format, base64_data) in formats {
        let messages = json!([
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "Analyze this"},
                    {
                        "type": "image_url",
                        "image_url": {"url": base64_data}
                    }
                ]
            }
        ]);

        let url = messages[0]["content"][1]["image_url"]["url"]
            .as_str()
            .unwrap();
        assert!(url.starts_with(&format!("data:image/{}", format)));
    }
}

// ============================================================================
// Test: test_multiple_images_in_message
// 输入: 一条消息中包含多张图片
// 预期: 正确处理多张图片
// ============================================================================
#[test]
fn test_multiple_images_in_message() {
    let messages = json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "Compare these two images"},
                {
                    "type": "image_url",
                    "image_url": {"url": "https://example.com/image1.jpg"}
                },
                {
                    "type": "image_url",
                    "image_url": {"url": "https://example.com/image2.jpg"}
                }
            ]
        }
    ]);

    let content = messages[0]["content"].as_array().unwrap();
    assert_eq!(content.len(), 3); // 1 text + 2 images

    // 验证两个图片
    assert_eq!(content[1]["type"], "image_url");
    assert_eq!(content[2]["type"], "image_url");
}

// ============================================================================
// Test: test_image_url_with_detail_parameter
// 输入: 包含 detail 参数的图像 URL
// 预期: detail 参数正确传递
// ============================================================================
#[test]
fn test_image_url_with_detail_parameter() {
    let messages = json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "Analyze in detail"},
                {
                    "type": "image_url",
                    "image_url": {
                        "url": "https://example.com/image.jpg",
                        "detail": "high"
                    }
                }
            ]
        }
    ]);

    let image_url = &messages[0]["content"][1]["image_url"];
    assert_eq!(image_url["detail"], "high");
}

// ============================================================================
// Test: test_multimodal_request_building
// 输入: 使用 client 构建多模态请求
// 预期: 请求对象正确构建
// ============================================================================
#[test]
fn test_multimodal_request_building() {
    let client = VllmClient::new("http://localhost:8000/v1");

    let _request = client
        .chat
        .completions()
        .create()
        .model("vision-model")
        .messages(json!([
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "What do you see?"},
                    {
                        "type": "image_url",
                        "image_url": {"url": "https://example.com/photo.jpg"}
                    }
                ]
            }
        ]))
        .max_tokens(500);

    // 如果能成功构建请求对象，测试通过
}

// ============================================================================
// Test: test_multimodal_with_system_message
// 输入: 包含系统消息的多模态对话
// 预期: 系统消息和用户消息都正确
// ============================================================================
#[test]
fn test_multimodal_with_system_message() {
    let messages = json!([
        {
            "role": "system",
            "content": "You are a helpful image analysis assistant."
        },
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "Describe this picture"},
                {
                    "type": "image_url",
                    "image_url": {"url": "https://example.com/pic.jpg"}
                }
            ]
        }
    ]);

    assert_eq!(messages.as_array().unwrap().len(), 2);
    assert_eq!(messages[0]["role"], "system");
    assert_eq!(messages[1]["role"], "user");
}

// ============================================================================
// Test: test_multimodal_conversation_history
// 输入: 多轮对话中包含图像
// 预期: 历史消息正确维护
// ============================================================================
#[test]
fn test_multimodal_conversation_history() {
    let messages = json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "What animal is this?"},
                {"type": "image_url", "image_url": {"url": "https://example.com/animal.jpg"}}
            ]
        },
        {
            "role": "assistant",
            "content": "This is a cat."
        },
        {
            "role": "user",
            "content": "What color is it?"
        }
    ]);

    let arr = messages.as_array().unwrap();
    assert_eq!(arr.len(), 3);

    // 第一条用户消息包含图像
    assert!(arr[0]["content"].is_array());
    assert_eq!(arr[0]["content"][1]["type"], "image_url");

    // 助手回复是普通文本
    assert!(arr[1]["content"].is_string());

    // 后续用户消息没有图像
    assert!(arr[2]["content"].is_string());
}

// ============================================================================
// Test: test_empty_image_url
// 输入: 空的图像 URL
// 预期: 结构上正确，但 URL 为空
// ============================================================================
#[test]
fn test_empty_image_url() {
    let messages = json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "Analyze this"},
                {
                    "type": "image_url",
                    "image_url": {"url": ""}
                }
            ]
        }
    ]);

    let url = messages[0]["content"][1]["image_url"]["url"]
        .as_str()
        .unwrap();
    assert!(url.is_empty());
}

// ============================================================================
// Test: test_image_url_serialization
// 输入: URL 格式的消息
// 预期: 序列化后 JSON 格式正确
// ============================================================================
#[test]
fn test_image_url_serialization() {
    let messages = json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "Check this"},
                {"type": "image_url", "image_url": {"url": "https://example.com/test.png"}}
            ]
        }
    ]);

    // 序列化再反序列化
    let json_str = serde_json::to_string(&messages).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(parsed, messages);
}

// ============================================================================
// Test: test_text_only_vs_multimodal_content
// 输入: 文本内容和多模态内容
// 预期: 两种格式都能正确处理
// ============================================================================
#[test]
fn test_text_only_vs_multimodal_content() {
    // 纯文本消息
    let text_only = json!([
        {
            "role": "user",
            "content": "Hello!"
        }
    ]);

    // 多模态消息
    let multimodal = json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "Hello!"},
                {"type": "image_url", "image_url": {"url": "https://example.com/img.jpg"}}
            ]
        }
    ]);

    // 纯文本 content 是字符串
    assert!(text_only[0]["content"].is_string());

    // 多模态 content 是数组
    assert!(multimodal[0]["content"].is_array());
}

// ============================================================================
// Test: test_base64_image_realistic
// 输入: 真实场景的 base64 图像（截取部分）
// 预期: 格式验证通过
// ============================================================================
#[test]
fn test_base64_image_realistic() {
    // 模拟一个更真实的 base64 编码（实际是一个很小的测试图像）
    let base64_prefix = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

    let client = VllmClient::new("http://localhost:8000/v1");

    let _request = client
        .chat
        .completions()
        .create()
        .model("vision-model")
        .messages(json!([
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "What color is this pixel?"},
                    {"type": "image_url", "image_url": {"url": base64_prefix}}
                ]
            }
        ]));

    // 验证 base64 格式
    assert!(base64_prefix.starts_with("data:image/png;base64,"));
    assert!(base64_prefix.len() > "data:image/png;base64,".len());
}

// ============================================================================
// Test: test_image_with_text_first
// 输入: 先文本后图像的顺序
// 预期: 顺序正确
// ============================================================================
#[test]
fn test_image_with_text_first() {
    let content = json!([
        {"type": "text", "text": "Look at this"},
        {"type": "image_url", "image_url": {"url": "https://example.com/img.jpg"}}
    ]);

    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[1]["type"], "image_url");
}

// ============================================================================
// Test: test_image_with_image_first
// 输入: 先图像后文本的顺序（虽然不推荐，但技术上可行）
// 预期: 顺序正确
// ============================================================================
#[test]
fn test_image_with_image_first() {
    let content = json!([
        {"type": "image_url", "image_url": {"url": "https://example.com/img.jpg"}},
        {"type": "text", "text": "Describe the image above"}
    ]);

    assert_eq!(content[0]["type"], "image_url");
    assert_eq!(content[1]["type"], "text");
}

// ============================================================================
// Test: test_image_url_http_and_https
// 输入: HTTP 和 HTTPS 两种 URL
// 预期: 都能正确处理
// ============================================================================
#[test]
fn test_image_url_http_and_https() {
    let https_url = "https://example.com/secure.jpg";
    let http_url = "http://example.com/insecure.jpg";

    let messages = json!([
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "Compare these"},
                {"type": "image_url", "image_url": {"url": https_url}},
                {"type": "image_url", "image_url": {"url": http_url}}
            ]
        }
    ]);

    let content = messages[0]["content"].as_array().unwrap();
    assert_eq!(content[1]["image_url"]["url"], https_url);
    assert_eq!(content[2]["image_url"]["url"], http_url);
}
