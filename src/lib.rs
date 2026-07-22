//! VLLM Client - OpenAI 兼容 vLLM API 的 Rust 客户端
//!
//! # 快速开始
//!
//! ```rust,no_run
//! use vllm_client::*;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = VllmClient::new("http://localhost:8000/v1");
//!
//!     let response = client.chat.completions().create()
//!         .model("Qwen/Qwen2.5-72B-Instruct")
//!         .messages(json!([
//!             {"role": "user", "content": "Hello!"}
//!         ]))
//!         .send()
//!         .await?;
//!
//!     println!("{}", response.content.unwrap());
//!     Ok(())
//! }
//! ```

mod chat;
mod client;
mod completions;
mod error;
mod request;
mod types;

pub use chat::{Chat, ChatCompletionRequest, ChatCompletions};
pub use client::{VllmClient, VllmClientBuilder};
pub use completions::{CompletionRequest, Completions};
pub use error::VllmError;
pub use types::*;

/// 便利性 re-export，方便构造 JSON
pub use serde_json::json;
