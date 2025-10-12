# Contributing to Agent-Ai-5

Thank you for your interest in contributing to Agent-Ai-5! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Standards](#code-standards)
- [Testing Guidelines](#testing-guidelines)
- [Submitting Changes](#submitting-changes)
- [CI/CD Pipeline](#cicd-pipeline)

## Code of Conduct

This project adheres to a code of conduct adapted from the [Contributor Covenant](https://www.contributor-covenant.org/). By participating, you are expected to uphold this code.

### Our Standards

- **Be respectful**: Treat all community members with respect and kindness
- **Be inclusive**: Welcome newcomers and help them get started
- **Be collaborative**: Work together and help each other
- **Be constructive**: Provide helpful feedback and suggestions

## Getting Started

### Prerequisites

- **Rust 1.70+** with Cargo
- **Docker** and **Docker Compose**
- **Git**
- **Pre-commit** (recommended): `pip install pre-commit`

### Initial Setup

1. **Fork and clone the repository**:
   ```bash
   git fork https://github.com/arabenjamin/Agent-Ai-5
   git clone https://github.com/YOUR_USERNAME/Agent-Ai-5
   cd Agent-Ai-5
   ```

2. **Set up the development environment**:
   ```bash
   # Install pre-commit hooks
   pre-commit install
   
   # Start Docker services
   docker compose up -d
   
   # Build all Rust projects
   cd mcp-client && cargo build && cd ..
   cd mcp-http-bridge && cargo build && cd ..
   cd mcp-server && cargo build && cd ..
   ```

3. **Verify the setup**:
   ```bash
   # Run tests for all projects
   cd mcp-client && cargo test && cd ..
   cd mcp-http-bridge && cargo test && cd ..
   cd mcp-server && cargo test && cd ..
   ```

## Development Workflow

### Branching Strategy

- **`master`**: Stable production code
- **`develop`**: Integration branch for features
- **`feature/feature-name`**: Individual feature development
- **`bugfix/issue-description`**: Bug fixes
- **`hotfix/critical-fix`**: Critical production fixes

### Workflow Steps

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following the code standards below

3. **Test your changes**:
   ```bash
   # Run pre-commit checks
   pre-commit run --all-files
   
   # Run tests
   cargo test --workspace
   
   # Check formatting and linting
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   ```

4. **Commit your changes**:
   ```bash
   git add .
   git commit -m "feat: add new feature description"
   ```

5. **Push and create a pull request**:
   ```bash
   git push origin feature/your-feature-name
   # Create PR through GitHub interface
   ```

## Code Standards

### Rust Code Style

- **Formatting**: Use `cargo fmt` (enforced by CI)
- **Linting**: Pass `cargo clippy` with the project's lint rules
- **Documentation**: Document all public APIs with `///` comments
- **Error Handling**: Use `anyhow::Result` for error propagation
- **Async Code**: Use `tokio` for async operations

### Code Organization

```rust
// Standard library imports
use std::collections::HashMap;

// External crate imports
use anyhow::Result;
use serde::{Deserialize, Serialize};

// Internal module imports
use crate::config::Config;

// Module structure
pub mod client;
pub mod server;
mod internal;

// Public API
pub use client::Client;
pub use server::Server;
```

### Naming Conventions

- **Modules**: `snake_case`
- **Functions**: `snake_case`
- **Variables**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Types/Structs**: `PascalCase`
- **Traits**: `PascalCase`

### Documentation Standards

```rust
/// Brief description of the function.
///
/// Longer description with more details about what this function does,
/// its behavior, and any important notes.
///
/// # Arguments
///
/// * `param1` - Description of the first parameter
/// * `param2` - Description of the second parameter
///
/// # Returns
///
/// Description of what the function returns.
///
/// # Errors
///
/// Description of when and why this function might return an error.
///
/// # Examples
///
/// ```rust
/// use your_crate::YourFunction;
///
/// let result = your_function("example")?;
/// assert_eq!(result, expected_value);
/// ```
pub fn your_function(param1: &str, param2: i32) -> Result<String> {
    // Implementation
}
```

## Testing Guidelines

### Test Structure

Each project should have comprehensive test coverage:

```
src/
├── lib.rs
├── client.rs
└── server.rs
tests/
├── integration_tests.rs
└── common/
    └── mod.rs
```

### Test Categories

1. **Unit Tests**: Test individual functions and modules
2. **Integration Tests**: Test component interactions
3. **End-to-End Tests**: Test complete workflows

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    
    #[tokio::test]
    async fn test_client_success() {
        // Arrange
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;
        
        // Act
        let result = client_function().await;
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### Test Requirements

- **Coverage**: Aim for >80% test coverage
- **Mocking**: Use `wiremock` for HTTP mocking
- **Async**: Use `tokio::test` for async tests
- **Documentation**: Test documentation examples

## Submitting Changes

### Pull Request Process

1. **Create a descriptive PR title**:
   ```
   feat: add new MCP tool for system monitoring
   fix: resolve connection timeout in HTTP bridge
   docs: update installation instructions
   ```

2. **Write a comprehensive description**:
   ```markdown
   ## Description
   Brief description of changes
   
   ## Changes Made
   - List of specific changes
   - Another change
   
   ## Testing
   - [ ] Unit tests pass
   - [ ] Integration tests pass
   - [ ] Manual testing completed
   
   ## Breaking Changes
   List any breaking changes
   
   ## Related Issues
   Closes #123
   ```

3. **Ensure CI passes**: All GitHub Actions workflows must pass

4. **Request review**: Tag relevant maintainers for review

### Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Formatting changes
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

Examples:
```
feat(client): add support for async tool execution
fix(server): resolve memory leak in connection pool
docs: update README with new installation steps
```

## CI/CD Pipeline

### Automated Checks

Every PR triggers:
- **Code formatting** verification
- **Lint checks** with Clippy
- **Test execution** (unit + integration)
- **Security audit** for vulnerabilities
- **Build verification** across platforms
- **Documentation** generation and testing

### Quality Gates

PRs must pass all checks:
- ✅ All tests passing
- ✅ Code coverage maintained
- ✅ No Clippy warnings
- ✅ Proper formatting
- ✅ No security vulnerabilities
- ✅ Documentation builds successfully

### Release Process

Releases are automated through tags:
1. Update version numbers in `Cargo.toml`
2. Create and push git tag: `git tag v1.0.0`
3. GitHub Actions handles the rest:
   - Binary builds for all platforms
   - Docker image publishing
   - Crate publishing to crates.io
   - Release notes generation

## Project Structure

### Repository Layout

```
ollama-n8n-stack/
├── .github/workflows/     # CI/CD workflows
├── mcp-client/           # CLI tool for development
├── mcp-http-bridge/      # HTTP to JSON-RPC bridge
├── mcp-server/           # Core MCP server
├── OpenWebUiTools/       # Open WebUI integration tools
├── docs/                 # Documentation
├── scripts/              # Utility scripts
└── docker-compose.*.yml  # Docker service definitions
```

### Adding New Components

When adding new Rust projects:

1. Create project directory with standard Rust structure
2. Add to CI/CD matrix in `.github/workflows/`
3. Include in root `deny.toml` configuration
4. Update documentation and examples

## Getting Help

- **Documentation**: Check existing docs and README files
- **Issues**: Search existing GitHub issues
- **Discussions**: Use GitHub Discussions for questions
- **Discord/Chat**: [Add if available]

## Recognition

Contributors are recognized in:
- GitHub contributor graphs
- Release notes for significant contributions
- Project documentation

Thank you for contributing to Agent-Ai-5! 🚀