//! VLLM Client - A Rust client for vLLM OpenAI-compatible API
//!
//! ## Example
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
pub mod types;

pub use chat::Chat;
pub use client::VllmClient;
pub use completions::Completions;
pub use error::VllmError;
pub use types::*;

// Re-export serde_json for convenience
pub use serde_json::json;
