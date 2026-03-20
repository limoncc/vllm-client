# Installation

## Requirements

- **Rust**: 1.70 or later
- **Cargo**: Comes with Rust installation

## Adding to Your Project

Add `vllm-client` to your `Cargo.toml`:

```toml
[dependencies]
vllm-client = "0.1"
```

Or use cargo add:

```bash
cargo add vllm-client
```

## Required Dependencies

The library requires tokio for async runtime. Add it to your `Cargo.toml`:

```toml
[dependencies]
vllm-client = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Optional Dependencies

For convenience, the library re-exports `serde_json::json`:

```toml
[dependencies]
vllm-client = "0.1"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

## Feature Flags

Currently, `vllm-client` does not have additional feature flags. All functionality is included by default.

## Verifying Installation

Create a simple test to verify the installation:

```rust
use vllm_client::VllmClient;

fn main() {
    let client = VllmClient::new("http://localhost:8000/v1");
    println!("Client created with base URL: {}", client.base_url());
}
```

Run with:

```bash
cargo run
```

## vLLM Server Setup

To use this client, you need a vLLM server running. Install and start vLLM:

```bash
# Install vLLM
pip install vllm

# Start vLLM server with a model
vllm serve Qwen/Qwen2.5-7B-Instruct --port 8000
```

The server will be available at `http://localhost:8000/v1`.

## Troubleshooting

### Connection Refused

If you see connection errors, ensure:

1. The vLLM server is running
2. The server URL is correct (default: `http://localhost:8000/v1`)
3. The port is not blocked by firewall

### TLS/SSL Issues

If your vLLM server uses HTTPS with a self-signed certificate, you may need to handle certificate validation in your application.

### Timeout Errors

For long-running requests, configure a longer timeout:

```rust
let client = VllmClient::new("http://localhost:8000/v1")
    .timeout_secs(300); // 5 minutes
```

## Next Steps

- [Quick Start](./quick-start.md) - Learn basic usage
- [Configuration](./configuration.md) - Configure the client