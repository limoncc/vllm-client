# 贡献指南

感谢您有兴趣为 vLLM Client 做贡献！本文档提供了贡献的指南和说明。

## 目录

- [行为准则](#行为准则)
- [入门指南](#入门指南)
- [开发环境设置](#开发环境设置)
- [进行更改](#进行更改)
- [测试](#测试)
- [文档](#文档)
- [Pull Request 流程](#pull-request-流程)
- [编码标准](#编码标准)

## 行为准则

请保持尊重和包容。我们欢迎所有人的贡献。

## 入门指南

1. 在 GitHub 上 Fork 仓库
2. 克隆您的 Fork 到本地
3. 为您的更改创建分支

```bash
git clone https://github.com/YOUR_USERNAME/vllm-client.git
cd vllm-client
git checkout -b my-feature
```

## 开发环境设置

### 前提条件

- Rust 1.70 或更高版本
- Cargo（随 Rust 一起安装）
- 用于集成测试的 vLLM 服务器（可选）

### 构建

```bash
# 构建库
cargo build

# 构建所有功能
cargo build --all-features
```

### 运行测试

```bash
# 运行单元测试
cargo test

# 运行测试并显示输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_name

# 运行集成测试（需要 vLLM 服务器）
cargo test --test integration
```

## 进行更改

### 分支命名

使用描述性的分支名称：

- `feature/add-new-feature` - 用于新功能
- `fix/bug-description` - 用于 bug 修复
- `docs/documentation-update` - 用于文档更改
- `refactor/code-cleanup` - 用于重构

### 提交消息

遵循约定式提交格式：

```
类型(范围): 描述

[可选正文]

[可选页脚]
```

类型：
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更改
- `style`: 代码风格更改（格式化等）
- `refactor`: 代码重构
- `test`: 添加或更新测试
- `chore`: 维护任务

示例：
```
feat(client): 添加连接池支持

fix(streaming): 正确处理空数据块

docs(api): 更新流式文档
```

## 测试

### 单元测试

所有新功能都应该有单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature() {
        // 测试实现
    }
}
```

### 集成测试

集成测试放在 `tests/` 目录中：

```rust
// tests/integration_test.rs
use vllm_client::{VllmClient, json};

#[tokio::test]
async fn test_chat_completion() {
    let client = VllmClient::new("http://localhost:8000/v1");
    // ... 测试代码
}
```

### 测试覆盖率

我们追求良好的测试覆盖率。运行覆盖率报告：

```bash
cargo tarpaulin --out Html
```

## 文档

### 代码文档

使用文档注释记录所有公共 API：

```rust
/// 创建新的聊天补全请求。
///
/// # 参数
///
/// * `model` - 用于生成的模型名称
///
/// # 返回
///
/// 新的 `ChatCompletionsRequest` 构建器
///
/// # 示例
///
/// ```rust
/// use vllm_client::{VllmClient, json};
///
/// let client = VllmClient::new("http://localhost:8000/v1");
/// let response = client.chat.completions().create()
///     .model("Qwen/Qwen2.5-7B-Instruct")
///     .messages(json!([{"role": "user", "content": "你好"}]))
///     .send()
///     .await?;
/// ```
pub fn create(&self) -> ChatCompletionsRequest {
    // 实现
}
```

### 更新文档

添加新功能时：

1. 更新内联文档
2. 更新 `docs/src/api/` 中的 API 参考
3. 在 `docs/src/examples/` 中添加示例
4. 更新变更日志

### 构建文档

```bash
# 构建并预览文档
cd docs && mdbook serve --open
```

## Pull Request 流程

1. **更新文档**：确保文档反映您的更改
2. **添加测试**：为新功能包含测试
3. **运行测试**：确保所有测试通过
4. **格式化代码**：运行 `cargo fmt`
5. **检查 Lint**：运行 `cargo clippy`
6. **更新 CHANGELOG**：在变更日志中添加条目

### PR 前检查清单

```bash
# 格式化代码
cargo fmt

# 检查 lint
cargo clippy -- -D warnings

# 运行所有测试
cargo test

# 构建文档
mdbook build docs
mdbook build docs/zh
```

### 提交 PR

1. 将您的分支推送到您的 Fork
2. 向 `main` 分支发起 PR
3. 填写 PR 模板
4. 等待审查

### PR 模板

```markdown
## 描述

更改的简要描述

## 更改类型

- [ ] Bug 修复
- [ ] 新功能
- [ ] 破坏性更改
- [ ] 文档更新

## 测试

- [ ] 单元测试已添加/更新
- [ ] 集成测试已添加/更新
- [ ] 已完成手动测试

## 检查清单

- [ ] 代码已用 `cargo fmt` 格式化
- [ ] 无 clippy 警告
- [ ] 文档已更新
- [ ] 变更日志已更新
```

## 编码标准

### Rust 风格

遵循标准 Rust 约定：

- 使用 `cargo fmt` 进行格式化
- 解决所有 `clippy` 警告
- 遵循 [Rust API 指南](https://rust-lang.github.io/api-guidelines/)

### 命名约定

- **类型**：PascalCase（`ChatCompletionResponse`）
- **函数/方法**：snake_case（`send_stream`）
- **常量**：SCREAMING_SNAKE_CASE（`MAX_RETRIES`）
- **模块**：snake_case（`chat`，`completions`）

### 错误处理

对所有错误使用 `VllmError`：

```rust
// 好
pub fn parse_response(data: &str) -> Result<Response, VllmError> {
    serde_json::from_str(data).map_err(VllmError::Json)
}

// 避免
pub fn parse_response(data: &str) -> Result<Response, String> {
    // ...
}
```

### 异步代码

对所有异步操作使用 `async/await`：

```rust
// 好
pub async fn send(&self) -> Result<Response, VllmError> {
    let response = self.http.post(&url).send().await?;
    // ...
}

// 避免在异步上下文中阻塞
pub async fn bad_example(&self) -> Result<Response, VllmError> {
    std::thread::sleep(Duration::from_secs(1)); // 不要这样做
    // ...
}
```

## 项目结构

```
vllm-client/
├── src/
│   ├── lib.rs         # 库入口点
│   ├── client.rs      # 客户端实现
│   ├── chat.rs        # 聊天 API
│   ├── completions.rs # 传统补全
│   ├── types.rs       # 类型定义
│   └── error.rs       # 错误类型
├── tests/
│   └── integration/   # 集成测试
├── docs/
│   ├── src/           # 英文文档
│   └── zh/src/        # 中文文档
├── examples/
│   └── *.rs           # 示例程序
└── Cargo.toml
```

## 获取帮助

- 对于 bug 或功能请求，请提交 issue
- 对于问题，请发起讨论
- 创建新 issue 前请先检查现有 issue

## 许可证

通过贡献，您同意您的贡献将根据 MIT OR Apache-2.0 许可证授权。

## 致谢

贡献者将在我们的 README 和发布说明中得到认可。

感谢您为 vLLM Client 做贡献！