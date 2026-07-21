//! 流式聊天示例
//!
//! 演示如何使用 vllm-client 进行流式聊天
//!
//! 环境变量:
//!   VLLM_BASE_URL  - API 地址（必填）
//!   VLLM_API_KEY   - API 密钥（可选）
//!   VLLM_MODEL     - 模型名称（可选，默认 Ring-2.6-1T）
//!   VLLM_TIMEOUT   - 超时秒数（可选，默认 120）
//!
//! 运行方式:
//! ```bash
//! cp .env.example .env    # 首次运行前配置
//! cargo run --example streaming_chat
//! ```

use std::env;
use std::io::Write;
use vllm_client::{json, StreamEvent, VllmClient};

// ANSI 颜色码
const COLOR_GRAY: &str = "\x1b[90m"; // 灰色（用于思考内容）
const COLOR_RESET: &str = "\x1b[0m"; // 重置颜色

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let base_url =
        env::var("VLLM_BASE_URL").expect("请设置 VLLM_BASE_URL 环境变量，或创建 .env 文件");
    let api_key = env::var("VLLM_API_KEY").ok();
    let model = env::var("VLLM_MODEL").unwrap_or_else(|_| "Ring-2.6-1T".into());
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

    println!("=== 流式聊天示例 ===\n");
    println!("模型: {model}\n");
    println!("用户: 你好，请介绍一下你自己");
    println!("\n助手: ");

    // 创建流式聊天请求
    let mut stream = client
        .chat
        .completions()
        .create()
        .model(&model)
        .messages(json!([
            {"role": "system", "content": "你是一个友好的AI助手, 你思考和说话以简洁著称。很少废话，你使用中文思考和回复"},
            {"role": "user", "content": "什么是机器学习,一句话说明即可。"}
        ]))
        .extra(json!({"chat_template_kwargs": {"enable_thinking": false}}))
        // .extra(json!({"reasoning": {"effort": "high"}}))
        .temperature(0.7)
        .max_tokens(2000)
        .stream(true)
        .send_stream()
        .await?;

    // 用于跟踪是否已经开始输出思考内容和普通内容
    let mut in_reasoning = false;
    let mut in_content = false;

    // 处理流式事件
    while let Some(event) = stream.next().await {
        // println!("{:?}", event);
        match event {
            StreamEvent::Content(delta) => {
                // 如果之前在输出思考内容，现在切换到普通内容
                if in_reasoning {
                    // 先重置颜色，然后换行
                    println!("{}", COLOR_RESET);
                    println!();
                    in_reasoning = false;
                }

                // 标记正在输出普通内容
                in_content = true;

                // 打印内容增量（实时输出）
                print!("{}", delta);
                // 强制刷新缓冲区，实现实时显示
                std::io::stdout().flush().ok();
            }
            StreamEvent::Reasoning(delta) => {
                // 如果之前在输出普通内容，现在切换到思考内容
                if in_content {
                    println!();
                    in_content = false;
                }

                // 第一次输出思考内容时打印标记并设置灰色
                if !in_reasoning {
                    print!("{}[思考]\n", COLOR_GRAY);
                    std::io::stdout().flush().ok();
                    in_reasoning = true;
                }

                // 打印思考内容增量（灰色）
                print!("{}", delta);
                std::io::stdout().flush().ok();
            }
            StreamEvent::Usage(usage) => {
                // 打印 usage 统计（只在 Done 前发一次）
                if in_reasoning {
                    print!("{}", COLOR_RESET);
                    std::io::stdout().flush().ok();
                }
                println!("\n\n--- Token 使用统计 ---");
                println!("提示词 tokens: {}", usage.prompt_tokens);
                println!("生成 tokens: {}", usage.completion_tokens);
                println!("总计 tokens: {}", usage.total_tokens);
            }
            StreamEvent::Done => {
                println!("\n=== 聊天完成 ===");
                break;
            }
            StreamEvent::Error(e) => {
                // 确保重置颜色
                if in_reasoning {
                    print!("{}", COLOR_RESET);
                    std::io::stdout().flush().ok();
                }
                eprintln!("\n错误: {}", e);
                return Err(e.into());
            }
            _ => {}
        }
    }

    Ok(())
}
