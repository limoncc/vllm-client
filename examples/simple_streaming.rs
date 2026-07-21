//! 最简单的流式聊天示例
//!
//! 环境变量:
//!   VLLM_BASE_URL  - API 地址（必填）
//!   VLLM_API_KEY   - API 密钥（可选）
//!   VLLM_MODEL     - 模型名称（可选，默认 inclusionAI/Ling-mini-2.0）
//!
//! 运行: cargo run --example simple_streaming

use std::env;
use vllm_client::{json, StreamEvent, VllmClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let base_url = env::var("VLLM_BASE_URL")
        .expect("请设置 VLLM_BASE_URL 环境变量，或创建 .env 文件");
    let api_key = env::var("VLLM_API_KEY").ok();
    let model = env::var("VLLM_MODEL").unwrap_or_else(|_| "inclusionAI/Ling-mini-2.0".into());

    let mut client = VllmClient::new(&base_url);
    if let Some(key) = &api_key {
        client = client.with_api_key(key);
    }

    let mut stream = client
        .chat
        .completions()
        .create()
        .model(&model)
        .messages(json!([{"role": "user", "content": "写一首关于春天的诗"}]))
        .stream(true)
        .send_stream()
        .await?;

    while let Some(event) = stream.next().await {
        match &event {
            StreamEvent::Reasoning(delta) => print!("{}", delta),
            StreamEvent::Content(delta) => print!("{}", delta),
            _ => {}
        }
    }
    println!();
    Ok(())
}
