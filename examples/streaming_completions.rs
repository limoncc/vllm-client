//! 流式 Completions 示例
//!
//! 演示如何使用 vllm-client 进行流式 Completions (旧版 API)
//!
//! 运行方式:
//! ```bash
//! cargo run --example streaming_completions
//! ```

use std::io::Write;
use vllm_client::{json, CompletionStreamEvent, VllmClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用用户提供的配置创建客户端
    let client = VllmClient::builder()
        .base_url("http://23.99.0.1:18120/v1")
        .api_key("sk-f58fe51e-c5f2-4510-3364-36f9c4e0f697")
        .timeout_secs(120)
        .build();

    println!("=== 流式 Completions 示例 ===\n");
    println!("模型: Qwen3.5-35B-A3B\n");
    println!("提示词: 什么是机器学习");
    println!("\n生成文本: ");

    // 创建流式 Completions 请求
    let mut stream = client
        .completions
        .create()
        .model("Qwen3.5-35B-A3B")
        .prompt(json!("什么是机器学习"))
        .max_tokens(500)
        .temperature(0.7)
        .stream(true)
        .send_stream()
        .await?;

    let mut has_content = false;

    // 处理流式事件
    while let Some(event) = stream.next().await {
        match event {
            CompletionStreamEvent::Text(delta) => {
                has_content = true;
                // 打印文本增量（实时输出）
                print!("{}", delta);
                // 强制刷新缓冲区，实现实时显示
                std::io::stdout().flush().ok();
            }
            CompletionStreamEvent::FinishReason(reason) => {
                println!("\n\n--- 结束原因: {} ---", reason);
            }
            CompletionStreamEvent::Usage(usage) => {
                // 流结束时输出 token 使用统计
                println!("\n\n--- Token 使用统计 ---");
                println!("提示词 tokens: {}", usage.prompt_tokens);
                println!("生成 tokens: {}", usage.completion_tokens);
                println!("总计 tokens: {}", usage.total_tokens);
            }
            CompletionStreamEvent::Done => {
                println!("\n\n=== 生成完成 ===");
                break;
            }
            CompletionStreamEvent::Error(e) => {
                eprintln!("\n错误: {}", e);
                return Err(e.into());
            }
        }
    }

    if !has_content {
        println!("(无内容生成)");
    }

    Ok(())
}
