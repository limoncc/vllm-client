//! 最简单的流式聊天示例
//!
//! 运行: cargo run --example simple_streaming

use vllm_client::{json, StreamEvent, VllmClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VllmClient::new("http://23.99.0.1:18120/v1")
        .with_api_key("sk-f58fe51e-c5f2-4510-3364-36f9c4e0f697");

    let mut stream = client
        .chat
        .completions()
        .create()
        .model("Qwen3.5-35B-A3B")
        .messages(json!([{"role": "user", "content": "写一首关于春天的诗"}]))
        .stream(true)
        .extra(json!({"chat_template_kwargs": {"enable_thinking": false}}))
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
