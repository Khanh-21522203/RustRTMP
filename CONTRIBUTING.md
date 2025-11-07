# Contributing to RustRTMP

Thank you for your interest in contributing to RustRTMP! This document provides guidelines and instructions for contributing.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/your-username/RustRTMP.git
   cd RustRTMP
   ```
3. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/my-new-feature
   ```

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- Cargo
- Git
- (Optional) FFmpeg for testing

### Building

```bash
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_test
```

### Running Examples

```bash
# List all examples
ls examples/

# Run an example
cargo run --example simple_server

# Run with logging
RUST_LOG=debug cargo run --example simple_server
```

## Code Guidelines

### Style

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/)
- Use `rustfmt` to format code:
  ```bash
  cargo fmt
  ```
- Use `clippy` to catch common mistakes:
  ```bash
  cargo clippy
  ```

### Code Structure

- **Keep modules focused** - Each module should have a single responsibility
- **Use meaningful names** - Variable and function names should be descriptive
- **Document public APIs** - All public functions, structs, and modules should have documentation
- **Handle errors properly** - Use the `Result` type and proper error handling

### Documentation

All public APIs must be documented with:

```rust
/// Brief description of what this function does
///
/// # Arguments
///
/// * `param1` - Description of param1
/// * `param2` - Description of param2
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// Description of when/why this might error
///
/// # Examples
///
/// ```
/// use rtmp::SomeFunction;
/// let result = some_function(arg);
/// ```
pub fn some_function(param1: Type1, param2: Type2) -> Result<ReturnType> {
    // implementation
}
```

### Testing

- **Unit tests** should be in the same file as the code they test
- **Integration tests** go in the `tests/` directory
- **Test coverage** - Aim for at least 70% code coverage
- **Test naming** - Use descriptive test names: `test_<what>_<when>_<expected>`

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_creation_with_valid_data_succeeds() {
        let header = RtmpHeader::new(0, 100, 8, 1, 3);
        let payload = vec![1, 2, 3];
        let packet = RtmpPacket::new(header, payload);
        
        assert_eq!(packet.header().message_length, 3);
    }

    #[tokio::test]
    async fn test_connection_with_invalid_url_returns_error() {
        let mut client = RtmpClient::new();
        let result = client.connect("invalid-url").await;
        assert!(result.is_err());
    }
}
```

## Commit Guidelines

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(client): add auto-reconnect functionality

Implement automatic reconnection when connection is lost.
Includes exponential backoff and max retry limits.

Closes #123
```

```
fix(server): resolve memory leak in connection pool

Fix issue where connections were not being properly cleaned up
after disconnect, leading to memory leak over time.

Fixes #456
```

### Commit Best Practices

- Keep commits focused and atomic
- Write clear, descriptive commit messages
- Reference issues in commit messages
- Don't commit commented-out code
- Don't commit debug print statements

## Pull Request Process

1. **Update your branch** with the latest changes from main:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Ensure all tests pass**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt -- --check
   ```

3. **Update documentation** if you changed APIs

4. **Add tests** for new functionality

5. **Create a Pull Request** with:
   - Clear title describing the change
   - Description of what changed and why
   - Reference to related issues
   - Screenshots/examples if applicable

### PR Title Format

Use the same format as commit messages:
```
feat(server): add connection pooling
fix(client): resolve handshake timeout issue
docs: update installation instructions
```

### PR Description Template

```markdown
## Description
Brief description of changes

## Motivation
Why are these changes needed?

## Changes
- Change 1
- Change 2
- Change 3

## Testing
How was this tested?

## Related Issues
Closes #123
Related to #456

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Code formatted with rustfmt
- [ ] No clippy warnings
- [ ] All tests passing
```

## Reporting Bugs

When reporting bugs, include:

1. **Description** - Clear description of the bug
2. **Steps to Reproduce** - Exact steps to reproduce the issue
3. **Expected Behavior** - What should happen
4. **Actual Behavior** - What actually happens
5. **Environment**:
   - OS and version
   - Rust version
   - RustRTMP version
6. **Logs** - Relevant log output (with `RUST_LOG=debug`)

### Bug Report Template

```markdown
**Description**
Brief description of the bug

**Steps to Reproduce**
1. Step 1
2. Step 2
3. Step 3

**Expected Behavior**
What should happen

**Actual Behavior**
What actually happens

**Environment**
- OS: Ubuntu 22.04
- Rust: 1.70.0
- RustRTMP: 0.1.0

**Logs**
```
paste logs here
```

**Additional Context**
Any other relevant information
```

## Suggesting Features

When suggesting features, include:

1. **Use Case** - Why is this feature needed?
2. **Proposed Solution** - How should it work?
3. **Alternatives** - Other approaches considered
4. **Additional Context** - Examples, mockups, etc.

## Code Review Process

All submissions require review. We aim to review PRs within:
- **Bug fixes**: 1-2 days
- **Features**: 3-5 days
- **Large changes**: 1 week

### What We Look For

- **Correctness** - Does it work as intended?
- **Tests** - Are there adequate tests?
- **Documentation** - Is it well documented?
- **Code Quality** - Is it readable and maintainable?
- **Performance** - Are there any performance concerns?
- **Breaking Changes** - Are breaking changes justified and documented?

## Development Workflow

### Typical Development Cycle

1. **Pick an issue** or create one
2. **Discuss approach** in the issue before major changes
3. **Create a branch** from main
4. **Implement changes** with tests
5. **Run tests locally**
6. **Submit PR**
7. **Address review feedback**
8. **Merge** after approval

### Working on Issues

- Comment on issues you want to work on
- Ask questions if requirements are unclear
- Keep the issue updated with progress
- Link PRs to issues

## Communication

- **GitHub Issues** - Bug reports, feature requests
- **Pull Requests** - Code review discussions
- **Discussions** - General questions, ideas

## Recognition

Contributors will be:
- Listed in the CONTRIBUTORS file
- Credited in release notes
- Mentioned in the README (for significant contributions)

## Questions?

If you have questions about contributing, feel free to:
- Open a Discussion on GitHub
- Comment on an existing issue
- Reach out to maintainers

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).

---

Thank you for contributing to RustRTMP! 🚀
