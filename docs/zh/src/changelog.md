# 更新日志

本文件记录了项目的所有重要更改。

格式基于 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，
本项目遵循 [语义化版本](https://semver.org/spec/v2.0.0.html)。

## [0.1.0] - 2024-01-XX

### 新增

- vLLM Client 初始版本发布
- `VllmClient` 用于连接 vLLM 服务器
- 聊天补全 API (`client.chat.completions()`)
- 使用 `MessageStream` 的流式响应支持
- 工具/函数调用支持
- 兼容模型的推理/思考模式支持
- 使用 `VllmError` 枚举的错误处理
- 客户端配置的构建器模式
- 聊天补全的请求构建器模式
- 通过 `extra()` 支持 vLLM 特定参数
- 响应中的 token 使用追踪
- 超时配置
- API Key 认证

### 功能

#### 客户端

- `VllmClient::new(base_url)` - 创建新客户端
- `VllmClient::builder()` - 使用构建器模式创建客户端
- `with_api_key()` - 设置用于认证的 API Key
- `timeout_secs()` - 设置请求超时

#### 聊天补全

- `model()` - 设置模型名称
- `messages()` - 设置对话消息
- `temperature()` - 设置采样温度
- `max_tokens()` - 设置最大输出 token 数
- `top_p()` - 设置核采样参数
- `top_k()` - 设置 top-k 采样（vLLM 扩展）
- `stop()` - 设置停止序列
- `stream()` - 启用流式模式
- `tools()` - 定义可用工具
- `tool_choice()` - 控制工具选择
- `extra()` - 传递 vLLM 特定参数

#### 流式响应

- `StreamEvent::Content` - 内容 token
- `StreamEvent::Reasoning` - 推理内容（思考模型）
- `StreamEvent::ToolCallDelta` - 流式工具调用更新
- `StreamEvent::ToolCallComplete` - 完整的工具调用
- `StreamEvent::Usage` - Token 使用统计
- `StreamEvent::Done` - 流式完成
- `StreamEvent::Error` - 错误事件

#### 响应类型

- `ChatCompletionResponse` - 聊天补全响应
- `ToolCall` - 带解析方法的工具调用数据
- `Usage` - Token 使用统计

### 依赖项

- `reqwest` - HTTP 客户端
- `serde` / `serde_json` - JSON 序列化
- `tokio` - 异步运行时
- `thiserror` - 错误处理

---

## [未发布]

### 计划中

- [ ] 自定义 HTTP 请求头支持
- [ ] 连接池配置
- [ ] 请求/响应日志
- [ ] 重试中间件
- [ ] 多模态输入辅助工具
- [ ] 批量处理的异步迭代器
- [ ] OpenTelemetry 集成
- [ ] WebSocket 传输

---

## 版本历史

| 版本 | 日期 | 亮点 |
|------|------|------|
| 0.1.0 | 2024-01 | 初始版本 |

---

[0.1.0]: https://github.com/limoncc/vllm-client/releases/tag/v0.1.0