# MCP Server

A Model Context Protocol (MCP) server implementation in Rust, designed to integrate with the Ollama n8n stack and provide useful tools for AI agents.

## Requirements

1. **Rust** - The server is written in Rust. Install it via [rustup](https://rustup.rs/):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Ollama** - Must be installed and running locally. Install from [ollama.ai](https://ollama.ai):
   ```bash
   curl -fsSL https://ollama.ai/install.sh | sh
   ```
   After installation, ensure the Ollama service is running:
   ```bash
   systemctl --user start ollama
   ```

## Features

### Built-in Tools

1. **System Information Tool** (`system_info`)
   - Get CPU, memory, disk usage information
   - View running Docker containers
   - System monitoring capabilities

2. **HTTP Request Tool** (`http_request`)
   - Make HTTP requests to external APIs
   - Support for GET, POST, PUT, DELETE, PATCH methods
   - Configurable headers and timeouts
   - Request/response logging

3. **Home Assistant Integration** (`homeassistant`)
   - Get entity states from Home Assistant
   - Call Home Assistant services
   - List available services
   - Full Home Assistant API integration

## Usage

### Running in Docker (Recommended)

The MCP server is configured as a service in the docker-compose stack:

```bash
# Build and start the MCP server
docker compose up mcp-server -d

# View logs
docker compose logs -f mcp-server

# Interactive testing
docker compose exec mcp-server /app/mcp-server --stdio
```

### Running Locally

```bash
# Install Rust and build
cd mcp-server
cargo build --release

# Run in STDIO mode (default)
./target/release/mcp-server --stdio

# Run in HTTP mode (development)
./target/release/mcp-server --port 8080
```

## Configuration

### Logging

The MCP server supports different log levels through either:

1. Command line argument: `--log-level=debug` (defaults to "debug")
2. Environment variable: `RUST_LOG=debug`

Available log levels:
- `error`: Only show errors
- `warn`: Show warnings and errors
- `info`: Show general information plus warnings and errors
- `debug`: Show detailed debug information (default)
- `trace`: Show very detailed trace information

Example using environment variable:
```bash
RUST_LOG=debug ./target/release/mcp-server
```

Example using command line:
```bash
./target/release/mcp-server --log-level=debug
```

### Environment Variables

- `RUST_LOG`: Set logging level (debug, info, warn, error)
- `HOMEASSISTANT_URL`: Home Assistant base URL (default: http://localhost:8123)
- `HOMEASSISTANT_TOKEN`: Home Assistant API token (required for HA integration)

### Docker Environment

The service is configured with:
- Host networking for easy service communication
- Docker socket access for container monitoring
- Data persistence in `./data/mcp-server`
- Non-root user for security

## Tool Examples

### System Information

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "system_info",
    "arguments": {
      "info_type": "all"
    }
  }
}
```

### HTTP Request

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "http_request",
    "arguments": {
      "method": "GET",
      "url": "https://api.github.com/users/octocat",
      "timeout": 10
    }
  }
}
```

### Home Assistant

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "homeassistant",
    "arguments": {
      "action": "get_states"
    }
  }
}
```

## Architecture

The MCP server follows the Model Context Protocol specification:

- **JSON-RPC 2.0** for communication
- **STDIO** transport (default) for security
- **Async/await** architecture with Tokio
- **Tool registry** pattern for extensibility
- **Error handling** with proper JSON-RPC error responses

## Security

- Runs as non-root user in container
- Limited Docker socket access (read-only)
- Input validation on all tool calls
- Secure token handling for Home Assistant
- Network isolation via Docker networking