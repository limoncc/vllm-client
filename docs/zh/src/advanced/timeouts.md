# 超时与重试

本页介绍超时配置和重试策略，用于构建健壮的生产应用程序。

## 设置超时

### 客户端级别超时

创建客户端时设置超时：

```rust
use vllm_client::VllmClient;

// 简单超时
let client = VllmClient::new("http://localhost:8000/v1")
    .timeout_secs(120);

// 使用构建器
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .timeout_secs(300)  // 5 分钟
    .build();
```

### 选择合适的超时时间

| 使用场景 | 推荐超时时间 |
|----------|-------------|
| 简单查询 | 30-60 秒 |
| 代码生成 | 2-3 分钟 |
| 长文档生成 | 5-10 分钟 |
| 复杂推理任务 | 10+ 分钟 |

### 请求耗时因素

请求所需时间取决于：

1. **提示词长度** - 更长的提示词需要更多处理时间
2. **输出 token 数** - 更多 token = 更长生成时间
3. **模型大小** - 更大的模型更慢
4. **服务器负载** - 繁忙的服务器响应更慢

## 超时错误

### 处理超时

```rust
use vllm_client::{VllmClient, json, VllmError};

async fn chat_with_timeout(prompt: &str) -> Result<String, VllmError> {
    let client = VllmClient::new("http://localhost:8000/v1")
        .timeout_secs(60);

    let result = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": prompt}]))
        .send()
        .await;

    match result {
        Ok(response) => Ok(response.content.unwrap_or_default()),
        Err(VllmError::Timeout) => {
            eprintln!("请求在 60 秒后超时");
            Err(VllmError::Timeout)
        }
        Err(e) => Err(e),
    }
}
```

## 重试策略

### 基础重试

使用指数退避重试失败的请求：

```rust
use vllm_client::{VllmClient, json, VllmError};
use std::time::Duration;
use tokio::time::sleep;

async fn send_with_retry(
    client: &VllmClient,
    prompt: &str,
    max_retries: u32,
) -> Result<String, VllmError> {
    let mut attempts = 0;

    loop {
        match client
            .chat
            .completions()
            .create()
            .model("Qwen/Qwen2.5-7B-Instruct")
            .messages(json!([{"role": "user", "content": prompt}]))
            .send()
            .await
        {
            Ok(response) => {
                return Ok(response.content.unwrap_or_default());
            }
            Err(e) if e.is_retryable() && attempts < max_retries => {
                attempts += 1;
                let delay = Duration::from_millis(100 * 2u64.pow(attempts - 1));
                eprintln!("第 {} 次重试，等待 {:?}: {}", attempts, delay, e);
                sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 带抖动的重试

添加抖动以防止惊群效应：

```rust
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

fn backoff_with_jitter(attempt: u32, base_ms: u64, max_ms: u64) -> Duration {
    let exponential = base_ms * 2u64.pow(attempt);
    let jitter = rand::thread_rng().gen_range(0..base_ms);
    let delay = (exponential + jitter).min(max_ms);
    Duration::from_millis(delay)
}

async fn retry_with_jitter<F, T, E>(
    mut f: F,
    max_retries: u32,
) -> Result<T, E>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    E: std::fmt::Debug,
{
    let mut attempts = 0;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                let delay = backoff_with_jitter(attempts, 100, 10_000);
                eprintln!("第 {} 次重试，等待 {:?}: {:?}", attempts, delay, e);
                sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 仅重试可重试错误

并非所有错误都应该重试：

```rust
use vllm_client::{VllmClient, json, VllmError};

async fn smart_retry(
    client: &VllmClient,
    prompt: &str,
) -> Result<String, VllmError> {
    let mut attempts = 0;
    let max_retries = 3;

    loop {
        let result = client
            .chat
            .completions()
            .create()
            .model("Qwen/Qwen2.5-7B-Instruct")
            .messages(json!([{"role": "user", "content": prompt}]))
            .send()
            .await;

        match result {
            Ok(response) => return Ok(response.content.unwrap_or_default()),
            Err(e) => {
                // 检查错误是否可重试
                if !e.is_retryable() {
                    return Err(e);
                }

                if attempts >= max_retries {
                    return Err(e);
                }

                attempts += 1;
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempts))).await;
            }
        }
    }
}
```

### 可重试错误

| 错误 | 可重试 | 原因 |
|------|--------|------|
| `Timeout` | 是 | 服务器可能较慢 |
| `429 频率限制` | 是 | 等待后重试 |
| `500 服务器错误` | 是 | 临时服务器问题 |
| `502 网关错误` | 是 | 服务器可能正在重启 |
| `503 服务不可用` | 是 | 临时过载 |
| `504 网关超时` | 是 | 服务器错误 |
| `400 请求错误` | 否 | 客户端错误 |
| `401 未授权` | 否 | 认证问题 |
| `404 未找到` | 否 | 资源不存在 |

## 断路器模式

使用断路器防止级联故障：

```rust
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use std::sync::Mutex;

struct CircuitBreaker {
    failures: AtomicU32,
    last_failure: Mutex<Option<Instant>>,
    threshold: u32,
    reset_duration: Duration,
}

impl CircuitBreaker {
    fn new(threshold: u32, reset_duration: Duration) -> Self {
        Self {
            failures: AtomicU32::new(0),
            last_failure: Mutex::new(None),
            threshold,
            reset_duration,
        }
    }

    fn can_attempt(&self) -> bool {
        let failures = self.failures.load(Ordering::Relaxed);
        if failures < self.threshold {
            return true;
        }

        let last = self.last_failure.lock().unwrap();
        if let Some(time) = *last {
            if time.elapsed() > self.reset_duration {
                // 重置断路器
                self.failures.store(0, Ordering::Relaxed);
                return true;
            }
        }

        false
    }

    fn record_success(&self) {
        self.failures.store(0, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        self.failures.fetch_add(1, Ordering::Relaxed);
        *self.last_failure.lock().unwrap() = Some(Instant::now());
    }
}
```

## 流式响应超时

处理流式响应过程中的超时：

```rust
use vllm_client::{VllmClient, json, StreamEvent};
use futures::StreamExt;
use tokio::time::{timeout, Duration};

async fn stream_with_timeout(
    client: &VllmClient,
    prompt: &str,
    per_event_timeout: Duration,
) -> Result<String, vllm_client::VllmError> {
    let mut stream = client
        .chat
        .completions()
        .create()
        .model("Qwen/Qwen2.5-7B-Instruct")
        .messages(json!([{"role": "user", "content": prompt}]))
        .stream(true)
        .send_stream()
        .await?;

    let mut content = String::new();

    loop {
        match timeout(per_event_timeout, stream.next()).await {
            Ok(Some(event)) => {
                match event {
                    StreamEvent::Content(delta) => content.push_str(&delta),
                    StreamEvent::Done => break,
                    StreamEvent::Error(e) => return Err(e),
                    _ => {}
                }
            }
            Ok(None) => break,
            Err(_) => {
                return Err(vllm_client::VllmError::Timeout);
            }
        }
    }

    Ok(content)
}
```

## 速率限制

实现客户端速率限制：

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

struct RateLimitedClient {
    client: vllm_client::VllmClient,
    semaphore: Arc<Semaphore>,
}

impl RateLimitedClient {
    fn new(base_url: &str, max_concurrent: usize) -> Self {
        Self {
            client: vllm_client::VllmClient::new(base_url),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    async fn chat(&self, prompt: &str) -> Result<String, vllm_client::VllmError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        
        self.client
            .chat
            .completions()
            .create()
            .model("Qwen/Qwen2.5-7B-Instruct")
            .messages(vllm_client::json!([{"role": "user", "content": prompt}]))
            .send()
            .await
            .map(|r| r.content.unwrap_or_default())
    }
}
```

## 生产环境配置

### 完整示例

```rust
use vllm_client::{VllmClient, json, VllmError};
use std::time::Duration;
use tokio::time::sleep;

struct RobustClient {
    client: VllmClient,
    max_retries: u32,
    base_backoff_ms: u64,
    max_backoff_ms: u64,
}

impl RobustClient {
    fn new(base_url: &str, timeout_secs: u64) -> Self {
        Self {
            client: VllmClient::builder()
                .base_url(base_url)
                .timeout_secs(timeout_secs)
                .build(),
            max_retries: 3,
            base_backoff_ms: 100,
            max_backoff_ms: 10_000,
        }
    }

    async fn chat(&self, prompt: &str) -> Result<String, VllmError> {
        let mut attempts = 0;

        loop {
            match self.send_request(prompt).await {
                Ok(response) => return Ok(response),
                Err(e) if self.should_retry(&e, attempts) => {
                    attempts += 1;
                    let delay = self.calculate_backoff(attempts);
                    eprintln!("第 {} 次重试，等待 {:?}: {}", attempts, delay, e);
                    sleep(delay).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn send_request(&self, prompt: &str) -> Result<String, VllmError> {
        self.client
            .chat
            .completions()
            .create()
            .model("Qwen/Qwen2.5-7B-Instruct")
            .messages(json!([{"role": "user", "content": prompt}]))
            .send()
            .await
            .map(|r| r.content.unwrap_or_default())
    }

    fn should_retry(&self, error: &VllmError, attempts: u32) -> bool {
        attempts < self.max_retries && error.is_retryable()
    }

    fn calculate_backoff(&self, attempt: u32) -> Duration {
        let delay = self.base_backoff_ms * 2u64.pow(attempt);
        Duration::from_millis(delay.min(self.max_backoff_ms))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RobustClient::new("http://localhost:8000/v1", 300);

    match client.chat("你好！").await {
        Ok(response) => println!("响应: {}", response),
        Err(e) => eprintln!("重试后仍然失败: {}", e),
    }

    Ok(())
}
```

## 最佳实践

1. **根据预期响应时间设置适当的超时**
2. **使用指数退避**以避免压垮服务器
3. **添加抖动**以防止惊群效应问题
4. **仅重试可重试错误** - 不要重试客户端错误
5. **为生产系统实现断路器**
6. **记录重试尝试**用于调试和监控
7. **设置最大重试次数**以避免无限循环

## 相关链接

- [错误处理](../api/error-handling.md) - 错误类型和处理
- [流式响应](../api/streaming.md) - 流式 API
- [配置](../getting-started/configuration.md) - 客户端配置