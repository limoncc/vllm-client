# 安装

## 环境要求

- **Rust**: 1.70 及以上版本
- **Cargo**: 安装 Rust 时会自动安装

## 引入项目

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
vllm-client = "0.1"
```

或直接运行：

```bash
cargo add vllm-client
```

## 依赖说明

本库依赖 tokio 异步运行时，请在 `Cargo.toml` 中添加：

```toml
[dependencies]
vllm-client = "0.1"
tokio = { version = "1", features = ["full"] }
```

为方便使用，库内重新导出了 `serde_json::json`，你可以选择添加：

```toml
[dependencies]
vllm-client = "0.1"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

## 特性开关

目前 `vllm-client` 暂无额外特性开关，所有功能默认启用。

## 验证安装

写一段简单代码验证安装是否成功：

```rust
use vllm_client::VllmClient;

fn main() {
    let client = VllmClient::new("http://localhost:8000/v1");
    println!("客户端创建成功，地址: {}", client.base_url());
}
```

运行：

```bash
cargo run
```

## 启动 vLLM 服务

使用本客户端前，需要先启动 vLLM 服务：

```bash
# 安装 vLLM
pip install vllm

# 启动服务并加载模型
vllm serve Qwen/Qwen2.5-7B-Instruct --port 8000
```

服务启动后会在 `http://localhost:8000/v1` 提供接口。

## 常见问题

### 连接失败

遇到连接错误时，请检查：

1. vLLM 服务是否正常运行
2. 服务地址是否正确（默认 `http://localhost:8000/v1`）
3. 防火墙是否阻止了端口访问

### TLS/SSL 报错

如果 vLLM 服务使用了自签名 HTTPS 证书，需要在代码中处理证书验证问题。

### 请求超时

请求耗时时长较大时，可以调大超时时间：

```rust
let client = VllmClient::new("http://localhost:8000/v1")
    .timeout_secs(300); // 5 分钟
```

## 下一步

- [快速上手](./quick-start.md) - 开发第一个示例
- [配置说明](./configuration.md) - 了解配置选项