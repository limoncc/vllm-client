# Contributing to vLLM Client

Thank you for your interest in contributing to vLLM Client! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)

## Code of Conduct

Be respectful and inclusive. We welcome contributions from everyone.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Create a branch for your changes

```bash
git clone https://github.com/YOUR_USERNAME/vllm-client.git
cd vllm-client
git checkout -b my-feature
```

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)
- A vLLM server for integration testing (optional)

### Building

```bash
# Build the library
cargo build

# Build with all features
cargo build --all-features
```

### Running Tests

```bash
# Run unit tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run integration tests (requires vLLM server)
cargo test --test integration
```

## Making Changes

### Branch Naming

Use descriptive branch names:

- `feature/add-new-feature` - for new features
- `fix/bug-description` - for bug fixes
- `docs/documentation-update` - for documentation changes
- `refactor/code-cleanup` - for refactoring

### Commit Messages

Follow conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(client): add connection pooling support

fix(streaming): handle empty chunks correctly

docs(api): update streaming documentation
```

## Testing

### Unit Tests

All new functionality should have unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature() {
        // Test implementation
    }
}
```

### Integration Tests

Integration tests go in the `tests/` directory:

```rust
// tests/integration_test.rs
use vllm_client::{VllmClient, json};

#[tokio::test]
async fn test_chat_completion() {
    let client = VllmClient::new("http://localhost:8000/v1");
    // ... test code
}
```

### Test Coverage

We aim for good test coverage. Run coverage reports:

```bash
cargo tarpaulin --out Html
```

## Documentation

### Code Documentation

Document all public APIs with doc comments:

```rust
/// Creates a new chat completion request.
///
/// # Arguments
///
/// * `model` - The model name to use for generation
///
/// # Returns
///
/// A new `ChatCompletionsRequest` builder
///
/// # Example
///
/// ```rust
/// use vllm_client::{VllmClient, json};
///
/// let client = VllmClient::new("http://localhost:8000/v1");
/// let response = client.chat.completions().create()
///     .model("Qwen/Qwen2.5-7B-Instruct")
///     .messages(json!([{"role": "user", "content": "Hello"}]))
///     .send()
///     .await?;
/// ```
pub fn create(&self) -> ChatCompletionsRequest {
    // Implementation
}
```

### Updating Documentation

When adding new features:

1. Update inline documentation
2. Update API reference in `docs/src/api/`
3. Add examples to `docs/src/examples/`
4. Update the changelog

### Building Documentation

```bash
# Build and preview documentation
cd docs && mdbook serve --open
```

## Pull Request Process

1. **Update Documentation**: Ensure documentation reflects your changes
2. **Add Tests**: Include tests for new functionality
3. **Run Tests**: Make sure all tests pass
4. **Format Code**: Run `cargo fmt`
5. **Check Lints**: Run `cargo clippy`
6. **Update CHANGELOG**: Add entry to changelog

### Pre-PR Checklist

```bash
# Format code
cargo fmt

# Check for lints
cargo clippy -- -D warnings

# Run all tests
cargo test

# Build documentation
mdbook build docs
mdbook build docs/zh
```

### Submitting the PR

1. Push your branch to your fork
2. Open a PR against the `main` branch
3. Fill in the PR template
4. Wait for review

### PR Template

```markdown
## Description

Brief description of changes

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing

- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing done

## Checklist

- [ ] Code formatted with `cargo fmt`
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Changelog updated
```

## Coding Standards

### Rust Style

Follow standard Rust conventions:

- Use `cargo fmt` for formatting
- Address all `clippy` warnings
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### Naming Conventions

- **Types**: PascalCase (`ChatCompletionResponse`)
- **Functions/Methods**: snake_case (`send_stream`)
- **Constants**: SCREAMING_SNAKE_CASE (`MAX_RETRIES`)
- **Modules**: snake_case (`chat`, `completions`)

### Error Handling

Use `VllmError` for all errors:

```rust
// Good
pub fn parse_response(data: &str) -> Result<Response, VllmError> {
    serde_json::from_str(data).map_err(VllmError::Json)
}

// Avoid
pub fn parse_response(data: &str) -> Result<Response, String> {
    // ...
}
```

### Async Code

Use `async/await` for all async operations:

```rust
// Good
pub async fn send(&self) -> Result<Response, VllmError> {
    let response = self.http.post(&url).send().await?;
    // ...
}

// Avoid blocking in async context
pub async fn bad_example(&self) -> Result<Response, VllmError> {
    std::thread::sleep(Duration::from_secs(1)); // Don't do this
    // ...
}
```

## Project Structure

```
vllm-client/
├── src/
│   ├── lib.rs         # Library entry point
│   ├── client.rs      # Client implementation
│   ├── chat.rs        # Chat API
│   ├── completions.rs # Legacy completions
│   ├── types.rs       # Type definitions
│   └── error.rs       # Error types
├── tests/
│   └── integration/   # Integration tests
├── docs/
│   ├── src/           # English documentation
│   └── zh/src/        # Chinese documentation
├── examples/
│   └── *.rs           # Example programs
└── Cargo.toml
```

## Getting Help

- Open an issue for bugs or feature requests
- Start a discussion for questions
- Check existing issues before creating new ones

## License

By contributing, you agree that your contributions will be licensed under the MIT OR Apache-2.0 license.

## Recognition

Contributors are recognized in our README and release notes.

Thank you for contributing to vLLM Client!