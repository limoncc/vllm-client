# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-01-XX

### Added

- Initial release of vLLM Client
- `VllmClient` for connecting to vLLM servers
- Chat completions API (`client.chat.completions()`)
- Streaming response support with `MessageStream`
- Tool/function calling support
- Reasoning/thinking mode support for compatible models
- Error handling with `VllmError` enum
- Builder pattern for client configuration
- Request builder pattern for chat completions
- Support for vLLM-specific parameters via `extra()`
- Token usage tracking in responses
- Timeout configuration
- API key authentication

### Features

#### Client

- `VllmClient::new(base_url)` - Create a new client
- `VllmClient::builder()` - Create a client with builder pattern
- `with_api_key()` - Set API key for authentication
- `timeout_secs()` - Set request timeout

#### Chat Completions

- `model()` - Set model name
- `messages()` - Set conversation messages
- `temperature()` - Set sampling temperature
- `max_tokens()` - Set maximum output tokens
- `top_p()` - Set nucleus sampling parameter
- `top_k()` - Set top-k sampling (vLLM extension)
- `stop()` - Set stop sequences
- `stream()` - Enable streaming mode
- `tools()` - Define available tools
- `tool_choice()` - Control tool selection
- `extra()` - Pass vLLM-specific parameters

#### Streaming

- `StreamEvent::Content` - Content tokens
- `StreamEvent::Reasoning` - Reasoning content (thinking models)
- `StreamEvent::ToolCallDelta` - Streaming tool call updates
- `StreamEvent::ToolCallComplete` - Complete tool call
- `StreamEvent::Usage` - Token usage statistics
- `StreamEvent::Done` - Stream completion
- `StreamEvent::Error` - Error events

#### Response Types

- `ChatCompletionResponse` - Chat completion response
- `ToolCall` - Tool call data with parsing methods
- `Usage` - Token usage statistics

### Dependencies

- `reqwest` - HTTP client
- `serde` / `serde_json` - JSON serialization
- `tokio` - Async runtime
- `thiserror` - Error handling

---

## [Unreleased]

### Planned

- [ ] Custom HTTP headers support
- [ ] Connection pooling configuration
- [ ] Request/response logging
- [ ] Retry middleware
- [ ] Multi-modal input helpers
- [ ] Async iterator for batch processing
- [ ] OpenTelemetry integration
- [ ] WebSocket transport

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| 0.1.0 | 2024-01 | Initial release |

---

[0.1.0]: https://github.com/limoncc/vllm-client/releases/tag/v0.1.0