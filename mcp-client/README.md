# MCP Client CLI Tool

A command-line tool for interacting with Ollama LLM models and MCP server. It provides seamless integration between language models and system tools through the Model Context Protocol (MCP).

## Features

- List available MCP tools
- Call MCP tools with arguments
- List available Ollama models
- Ask questions to Ollama models
- Interactive chat mode with automatic tool usage
- Streaming responses from Ollama models
- Automatic JSON parsing and formatting
- Rich error handling and logging

## Usage

### List MCP Tools
Lists all available tools from the MCP server:
```bash
mcp-client list-tools
```

### Call an MCP Tool
Directly call a specific MCP tool with arguments:
```bash
mcp-client call-tool --name tool_name --args '{"key": "value"}'
```

### List Ollama Models
Shows all available models from your Ollama server:
```bash
mcp-client list-models
```

### Ask a Question
Send a one-off question to an Ollama model:
```bash
mcp-client ask --model llama3 --prompt "What is the meaning of life?"
```

### Interactive Chat with Tool Usage
Chat with a model and let it use MCP tools as needed:
```bash
mcp-client chat --model llama3 --prompt "Can you check my system's CPU usage?"
```

The chat mode allows the model to:
- Understand available tools and their capabilities
- Automatically format tool calls in correct JSON format
- Execute tools and interpret their results
- Provide human-friendly explanations of tool outputs

## Configuration

Command-line options can be used to customize the behavior:

- `--ollama-url`: URL of the Ollama server (default: http://localhost:11434)
- `--mcp-url`: URL of the MCP server (default: http://localhost:3001)
- `--log-level`: Logging level (default: info)

Examples:
```bash
# Use custom server URLs
mcp-client --ollama-url http://custom:11434 --mcp-url http://custom:3001 chat --model llama3 --prompt "Hello"

# Enable debug logging
mcp-client --log-level debug list-tools
```

## Requirements

- Rust 1.70 or later
- Running Ollama server
- Running MCP server
- Docker (if using container-related tools)

### Docker Permissions

If you plan to use Docker-related tools, ensure your user has the proper permissions:

1. Add your user to the docker group:
```bash
sudo usermod -aG docker $USER
```

2. Log out and back in, or run:
```bash
newgrp docker
```

## Building and Installation

1. Clone the repository:
```bash
git clone [repository-url]
cd mcp-client
```

2. Build the project:
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