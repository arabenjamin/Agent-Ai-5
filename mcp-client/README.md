# MCP Client CLI Tool

A command-line tool for interacting with Ollama LLM models and MCP (Model Context Protocol) servers. It provides seamless integration between language models and system tools, enabling AI assistants to perform complex tasks through tool execution.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
- [Testing](#testing)
- [Development](#development)
- [Configuration](#configuration)
- [Project Structure](#project-structure)
- [Contributing](#contributing)
- [License](#license)

## Features

- **MCP Integration**: List and call tools from MCP servers
- **Ollama Support**: Interact with local Ollama language models
- **Tool-Assisted Chat**: Enable AI models to use system tools intelligently
- **Direct Queries**: Ask simple questions without tool integration
- **Robust Error Handling**: Detailed error messages and logging
- **Flexible Configuration**: Customizable server URLs and logging levels
- **Comprehensive Testing**: Unit and integration tests for reliability

## Installation

### From Source

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd mcp-client
   ```

3. **Build the project**:
   ```bash
   cargo build --release
   ```

4. **Install globally** (optional):
   ```bash
   cargo install --path .
   ```

### Prerequisites

- **Rust 1.70+** and Cargo
- **MCP Server** running (default: http://localhost:3001)
- **Ollama Server** running (default: http://localhost:11434)

## Quick Start

1. **Check available tools**:
   ```bash
   mcp-client list-tools
   ```

2. **List Ollama models**:
   ```bash
   mcp-client list-models
   ```

3. **Ask a simple question**:
   ```bash
   mcp-client ask --model llama2 --prompt "What is the capital of France?"
   ```

4. **Use tools via chat**:
   ```bash
   mcp-client chat --model llama2 --prompt "What's my system's memory usage?"
   ```

## Usage

### Available Commands

#### 1. List MCP Tools
```bash
# List all available tools from MCP server
mcp-client list-tools

# With custom MCP server URL
mcp-client --mcp-url http://custom:3001 list-tools
```

#### 2. Call Specific Tool
```bash
# Call a tool with arguments
mcp-client call-tool --name system_info --args '{"action": "memory"}'

# Call tool without arguments
mcp-client call-tool --name list_processes
```

#### 3. List Ollama Models
```bash
# List available models
mcp-client list-models

# With custom Ollama server URL
mcp-client --ollama-url http://custom:11434 list-models
```

#### 4. Ask Command (Simple Queries)
```bash
# Direct question to model
mcp-client ask --model llama2 --prompt "Explain REST APIs"

# Code-related queries
mcp-client ask --model codellama --prompt "How to handle errors in Rust?"
```

**Use `ask` when you want**:
- Quick, simple questions
- Direct model responses without tools
- Educational or explanatory content
- No system interaction required

#### 5. Chat Command (Tool-Assisted)
```bash
# System monitoring
mcp-client chat --model llama2 --prompt "Check my CPU usage"

# Complex analysis
mcp-client chat --model codellama --prompt "Find processes using high memory"
```

**Use `chat` when you need**:
- Access to system tools and information
- Complex multi-step tasks
- System monitoring and management
- Automated task execution

The chat command workflow:
1. Loads available MCP tools
2. Creates context-aware system prompts
3. Interprets model responses for tool usage
4. Executes tools when requested by the model
5. Provides model interpretation of tool results

### Global Options

All commands support these options:
- `--ollama-url`: Ollama server URL (default: http://localhost:11434)
- `--mcp-url`: MCP server URL (default: http://localhost:3001)
- `--log-level`: Logging level - debug, info, warn, error (default: info)

### Examples

**System Information**:
```bash
# Get memory usage through tools
mcp-client chat --model llama2 --prompt "What's my current memory usage?"

# Direct tool call
mcp-client call-tool --name system_info --args '{"type": "memory"}'
```

**Development Assistance**:
```bash
# Ask for code explanation
mcp-client ask --model codellama --prompt "Explain async/await in Rust"

# System analysis
mcp-client chat --model llama2 --prompt "Check if Docker containers are running efficiently"
```

**Tool Discovery**:
```bash
# See what tools are available
mcp-client list-tools

# Ask model about available tools
mcp-client chat --model llama2 --prompt "What tools do you have available?"
```

## Testing

The project includes comprehensive test coverage with both unit and integration tests.

### Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_tests

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_list_tools_success
```

### Test Coverage

- **Unit Tests** (37 tests): Test individual modules (mcp.rs, ollama.rs)
  - MCP client: tool listing, calling, error handling
  - Ollama client: model listing, text generation, streaming
  - Serialization/deserialization of data structures

- **Integration Tests** (17 tests): Test CLI functionality end-to-end
  - Command parsing and validation
  - Error handling and logging
  - Mock server interactions
  - Tool execution workflows

### Test Architecture

Tests use `wiremock` for HTTP mocking and `assert_cmd` for CLI testing:
```bash
# Example: Run integration tests with detailed output
RUST_LOG=debug cargo test --test integration_tests -- --nocapture
```

## Development

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Check code formatting
cargo fmt --check

# Run linting
cargo clippy

# Fix formatting
cargo fmt
```

### Development Environment

The project includes VS Code configuration:
- `.vscode/launch.json`: Debug configurations
- `.vscode/tasks.json`: Build and test tasks

### Adding Features

1. **New commands**: Add to `Commands` enum in `main.rs`
2. **New clients**: Create modules following `mcp.rs`/`ollama.rs` patterns
3. **Tests**: Add unit tests in module and integration tests in `tests/`

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- list-tools

# Use VS Code debugger with launch.json configurations
# Or attach to running process for debugging
```

## Configuration

### Environment Variables

```bash
# Set custom log level
export RUST_LOG=debug

# Custom server URLs can be set via CLI args
mcp-client --mcp-url http://192.168.1.100:3001 --ollama-url http://192.168.1.100:11434 list-tools
```

### Server Requirements

**MCP Server**:
- Must implement standard MCP endpoints:
  - `GET /tools` - List available tools
  - `POST /tools/call` - Execute tools
- Response format must match MCP specification

**Ollama Server**:
- Standard Ollama API endpoints:
  - `GET /api/tags` - List models
  - `POST /api/generate` - Generate text
- Streaming responses supported

## Project Structure

```
mcp-client/
├── src/
│   ├── main.rs          # CLI interface, argument parsing, command routing
│   ├── mcp.rs           # MCP client implementation and data structures
│   └── ollama.rs        # Ollama API client and streaming support
├── tests/
│   └── integration_tests.rs  # End-to-end CLI testing with mocks
├── .vscode/             # VS Code development configuration
├── Cargo.toml           # Dependencies and project metadata
└── README.md           # This file
```

### Key Components

- **CLI Interface** (`main.rs`): Command parsing, routing, and user interaction
- **MCP Client** (`mcp.rs`): HTTP client for MCP server communication
- **Ollama Client** (`ollama.rs`): HTTP client with streaming support for Ollama
- **Integration Tests**: Comprehensive CLI testing with mock servers

## Error Handling

The client provides detailed error reporting:

### Common Issues

1. **Connection Errors**:
   ```
   Failed to list tools: Connection refused
   → Check if MCP server is running on http://localhost:3001
   ```

2. **Tool Execution Errors**:
   ```
   Failed to call tool: Invalid JSON arguments
   → Verify JSON format: {"key": "value"}
   ```

3. **Model Errors**:
   ```
   Failed to generate response: Model not found
   → Check available models with: mcp-client list-models
   ```

### Debugging Tips

1. **Enable debug logging**:
   ```bash
   mcp-client --log-level debug list-tools
   ```

2. **Check server connectivity**:
   ```bash
   curl http://localhost:3001/tools    # MCP server
   curl http://localhost:11434/api/tags  # Ollama server
   ```

3. **Validate JSON arguments**:
   ```bash
   echo '{"key": "value"}' | jq .  # Validate JSON
   ```

## Dependencies

### Core Dependencies
- **tokio**: Async runtime for concurrent operations
- **reqwest**: HTTP client with JSON support
- **serde**: JSON serialization/deserialization
- **clap**: Command-line argument parsing
- **anyhow**: Error handling and context
- **tracing**: Structured logging and diagnostics
- **futures-util**: Stream processing utilities

### Development Dependencies
- **wiremock**: HTTP mocking for tests
- **assert_cmd**: CLI testing framework
- **predicates**: Test assertions
- **assert-json-diff**: JSON comparison in tests
- **tokio-test**: Async testing utilities

## Contributing

Contributions are welcome! Please follow these guidelines:

### Development Workflow

1. **Fork and clone**:
   ```bash
   git fork <repository-url>
   git clone <your-fork-url>
   cd mcp-client
   ```

2. **Create feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Develop and test**:
   ```bash
   cargo fmt
   cargo clippy
   cargo test
   ```

4. **Submit pull request** with:
   - Clear description of changes
   - Test coverage for new features
   - Updated documentation if needed

### Code Standards

- Follow Rust conventions and `cargo fmt` formatting
- Add tests for new functionality
- Update documentation for API changes
- Use descriptive commit messages

## License

This project is licensed under the MIT License. See LICENSE file for details.

---

For questions or issues, please open a GitHub issue or submit a pull request.