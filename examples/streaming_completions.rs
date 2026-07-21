//! 流式 Completions 示例
//!
//! 演示如何使用 vllm-client 进行流式 Completions (旧版 API)
//!
//! 环境变量:
//!   VLLM_BASE_URL  - API 地址（必填）
//!   VLLM_API_KEY   - API 密钥（可选）
//!   VLLM_MODEL     - 模型名称（可选，默认 Qwen3.5-35B-A3B）
//!   VLLM_TIMEOUT   - 超时秒数（可选，默认 120）
//!
//! 运行方式:
//! ```bash
//! cp .env.example .env    # 首次运行前配置
//! cargo run --example streaming_completions
//! ```

use std::io::Write;
use std::env;
use vllm_client::{json, CompletionStreamEvent, VllmClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let base_url = env::var("VLLM_BASE_URL")
        .expect("请设置 VLLM_BASE_URL 环境变量，或创建 .env 文件");
    let api_key = env::var("VLLM_API_KEY").ok();
    let model = env::var("VLLM_MODEL").unwrap_or_else(|_| "Qwen3.5-35B-A3B".into());
    let timeout: u64 = env::var("VLLM_TIMEOUT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(120);

    let mut client_builder = VllmClient::builder()
        .base_url(&base_url)
        .timeout_secs(timeout);

    if let Some(key) = &api_key {
        client_builder = client_builder.api_key(key);
    }
    let client = client_builder.build();

    println!("=== 流式 Completions 示例 ===\n");
    println!("模型: {model}\n");
    println!("提示词: 什么是机器学习");
    println!("\n生成文本: ");

    // 创建流式 Completions 请求
    let mut stream = client
        .completions
        .create()
        .model(&model)
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
