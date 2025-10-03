# MCP Client CLI Tool

A command-line tool for interacting with Ollama LLM models and MCP server. It provides seamless integration between language models and system tools through the Model Context Protocol (MCP).

## Features

- List available MCP tools from the server
- Call specific MCP tools with JSON arguments
- List available Ollama models
- Ask one-off questions to Ollama models
- Automatic JSON parsing and formatting
- Detailed error reporting and logging

## Development

### Prerequisites
- Rust 1.70 or later
- Cargo (Rust's package manager)
- Running MCP server (default: http://localhost:3001)
- Running Ollama server (default: http://localhost:11434)

### Building
```bash
# Check code formatting
cargo fmt --check

# Run code linting
cargo clippy

# Build in debug mode
cargo build

# Build optimized release version
cargo build --release

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- --help
```

### IDE Setup
The repository includes VS Code configurations in `.vscode/`:
- `launch.json`: Debug configurations
- `tasks.json`: Build tasks

## Usage

### Available Commands

1. List MCP Tools
```bash
# List all available tools
cargo run -- list-tools

# With custom MCP server URL
cargo run -- --mcp-url http://custom:3001 list-tools
```

2. Call a Specific Tool
```bash
# Call a tool with arguments
cargo run -- call-tool --name system_info --args '{"action": "get_system_info"}'

# Call tool with no arguments
cargo run -- call-tool --name list_tools
```

3. List Ollama Models
```bash
# List available models
cargo run -- list-models

# With custom Ollama server URL
cargo run -- --ollama-url http://custom:11434 list-models
```

### Interacting with Models

The client provides two distinct ways to interact with Ollama models:

#### Ask Command
The `ask` command provides direct, simple interaction with an Ollama model:
```bash
# Direct question to model
cargo run -- ask --model llama2 --prompt "What is REST API?"
```
Use `ask` when you want:
- Quick, simple questions
- Direct model responses
- No need for system tools
- Learning or testing model capabilities
- Documentation or explanation queries

#### Chat Command
The `chat` command enables complex interactions with tool integration:
```bash
# Tool-assisted interaction
cargo run -- chat --model llama2 --prompt "Check my system's CPU usage"

# Complex task with tool usage
cargo run -- chat --model codellama --prompt "Monitor Docker containers and show their status"
```
Use `chat` when you need:
- Access to system tools and information
- Complex multi-step tasks
- System monitoring and management
- Tool-assisted analysis
- Automated task execution

The chat command:
1. Loads available MCP tools
2. Creates a context-aware system prompt
3. Interprets model responses for tool usage
4. Executes tools when requested
5. Gets model interpretation of tool results

### Command-Line Arguments

Every command supports these global options:
- `--ollama-url`: Ollama server URL (default: http://localhost:11434)
- `--mcp-url`: MCP server URL (default: http://localhost:3001)
- `--log-level`: Logging level (default: info)

### Examples

Direct Question (Ask):
```bash
# Simple coding question
cargo run -- ask --model codellama --prompt "Explain how to use Result in Rust"

# General knowledge query
cargo run -- ask --model llama2 --prompt "What are the SOLID principles?"
```

Tool Integration (Chat):
```bash
# System monitoring
cargo run -- chat --model llama2 --prompt "Check if any containers are using too much memory"

# Complex analysis
cargo run -- chat --model codellama --prompt "Analyze my system performance and suggest improvements"
```

## Project Structure

```
mcp-client/
├── src/
│   ├── main.rs      # CLI interface and command handling
│   ├── mcp.rs       # MCP client implementation
│   └── ollama.rs    # Ollama API client
├── .vscode/         # VS Code configurations
│   ├── launch.json  # Debug configurations
│   └── tasks.json   # Build tasks
└── Cargo.toml       # Project dependencies and metadata
```

## Error Handling

The client provides detailed error messages for common issues:

1. Connection Errors
   - Check if MCP server is running and accessible
   - Verify Ollama server is running and responding
   - Check network connectivity and URLs

2. Tool Execution Errors
   - Validate JSON argument format
   - Ensure tool exists and is available
   - Check tool-specific requirements (e.g., Docker for container operations)

3. Model Errors
   - Verify model is downloaded and available
   - Check model compatibility with query
   - Monitor resource usage during model execution

## Contributing

1. Fork and clone the repository
2. Create a new branch for your feature
3. Run tests and formatting:
   ```bash
   cargo fmt
   cargo clippy
   cargo test
   ```
4. Submit a pull request

## License

This project is licensed under the MIT License.
```bash
cargo build --release
```

The binary will be available at `target/release/mcp-client`.

## Examples

1. Get system information:
```bash
mcp-client chat --model llama3 --prompt "What's my CPU usage?"
```

2. List and use available tools:
```bash
# First list available tools
mcp-client list-tools

# Then use a tool through chat
mcp-client chat --model llama3 --prompt "Can you use the system_info tool to check my memory usage?"
```

3. Direct tool usage:
```bash
mcp-client call-tool --name system_info --args '{"info_type": "memory"}'
```

## Error Handling

The client provides detailed error messages for common issues:
- Connection errors to Ollama/MCP servers
- Invalid JSON in tool arguments
- Tool execution failures
- Permission issues (e.g., Docker access)

## Dependencies

- tokio - Async runtime
- reqwest - HTTP client
- serde - JSON serialization
- clap - Command-line argument parsing
- tracing - Logging and diagnostics
- futures-util - Async utilities
- anyhow - Error handling

## Contributing

Contributions are welcome! Please feel free to submit pull requests.