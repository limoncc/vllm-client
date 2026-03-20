# Timeouts & Retries

This page covers timeout configuration and retry strategies for robust production applications.

## Setting Timeouts

### Client-Level Timeout

Set a timeout when creating the client:

```rust
use vllm_client::VllmClient;

// Simple timeout
let client = VllmClient::new("http://localhost:8000/v1")
    .timeout_secs(120);

// Using builder
let client = VllmClient::builder()
    .base_url("http://localhost:8000/v1")
    .timeout_secs(300)  // 5 minutes
    .build();
```

### Choosing the Right Timeout

| Use Case | Recommended Timeout |
|----------|---------------------|
| Simple queries | 30-60 seconds |
| Code generation | 2-3 minutes |
| Long document generation | 5-10 minutes |
| Complex reasoning tasks | 10+ minutes |

### Request Duration Factors

The time a request takes depends on:

1. **Prompt length** - Longer prompts take more time to process
2. **Output tokens** - More tokens = longer generation time
3. **Model size** - Larger models are slower
4. **Server load** - Busy servers respond slower

## Timeout Errors

### Handling Timeout

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
            eprintln!("Request timed out after 60 seconds");
            Err(VllmError::Timeout)
        }
        Err(e) => Err(e),
    }
}
```

## Retry Strategies

### Basic Retry

Retry failed requests with exponential backoff:

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
                eprintln!("Retry {} after {:?}: {}", attempts, delay, e);
                sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Retry with Jitter

Add jitter to prevent thundering herd:

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
                eprintln!("Retry {} after {:?}: {:?}", attempts, delay, e);
                sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Retry Only Retryable Errors

Not all errors should be retried:

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
                // Check if error is retryable
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

### Retryable Errors

| Error | Retryable | Reason |
|-------|-----------|--------|
| `Timeout` | Yes | Server may be slow |
| `429 Rate Limited` | Yes | Wait and retry |
| `500 Server Error` | Yes | Temporary server issue |
| `502 Bad Gateway` | Yes | Server may restart |
| `503 Unavailable` | Yes | Temporary overload |
| `504 Gateway Timeout` | Yes | Server error |
| `429 Rate Limited` | Yes | Should wait |
| `500 Server Error` | Yes | Temporary issue |
| `502/503/504` | Yes | Gateway errors |
| `400 Bad Request` | No | Client error |
| `401 Unauthorized` | No | Authentication issue |
| `404 Not Found` | No | Resource doesn't exist |

## Circuit Breaker Pattern

Prevent cascading failures with a circuit breaker:

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
                // Reset circuit breaker
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

## Streaming Timeout

Handle timeouts during streaming:

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

## Rate Limiting

Implement client-side rate limiting:

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

## Production Configuration

### Complete Example

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
                    eprintln!("Retry {} after {:?}: {}", attempts, delay, e);
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

    match client.chat("Hello!").await {
        Ok(response) => println!("Response: {}", response),
        Err(e) => eprintln!("Failed after retries: {}", e),
    }

    Ok(())
}
```

## Best Practices

1. **Set appropriate timeouts** based on expected response times
2. **Use exponential backoff** to avoid overwhelming the server
3. **Add jitter** to prevent thundering herd problems
4. **Only retry retryable errors** - don't retry client errors
5. **Implement circuit breakers** for production systems
6. **Log retry attempts** for debugging and monitoring
7. **Set a maximum retry count** to avoid infinite loops

## See Also

- [Error Handling](../api/error-handling.md) - Error types and handling
- [Streaming](../api/streaming.md) - Streaming API
- [Configuration](../getting-started/configuration.md) - Client configuration