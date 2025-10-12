# MCP HTTP Bridge

A high-performance HTTP REST API bridge for Model Context Protocol (MCP) servers, providing a web-accessible interface to MCP server functionality.

## 🚀 Features

- **RESTful API** - Clean HTTP endpoints for MCP server communication
- **OpenAPI Documentation** - Auto-generated API documentation at `/openapi.json`
- **Health Monitoring** - Built-in health check endpoint
- **CORS Support** - Cross-origin request handling for web applications
- **Error Handling** - Comprehensive error responses and logging
- **High Performance** - Built with Axum for optimal throughput
- **Docker Ready** - Containerized deployment support

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [API Endpoints](#api-endpoints)
- [Configuration](#configuration)
- [Development](#development)
- [Testing](#testing)
- [Docker Deployment](#docker-deployment)
- [API Documentation](#api-documentation)
- [Error Handling](#error-handling)
- [Contributing](#contributing)

## 🏁 Quick Start

### Prerequisites

- Rust 1.70+ 
- A running MCP server

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd mcp-http-bridge
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Run the bridge**
   ```bash
   cargo run -- --port 3001 --mcp-server-path http://localhost:3002
   ```

The bridge will start on `http://localhost:3001` and connect to your MCP server at `http://localhost:3002`.

## 🔗 API Endpoints

### Health Check
- **GET** `/health`
- Returns service status and version information
- Always returns 200 OK when service is running

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

### List Available Tools
- **GET** `/tools`
- Returns all available tools from the connected MCP server
- Includes tool names, descriptions, and input schemas

**Response:**
```json
{
  "tools": [
    {
      "name": "system_info",
      "description": "Get system information",
      "input_schema": {
        "type": "object",
        "properties": {
          "detailed": {"type": "boolean"}
        }
      }
    }
  ]
}
```

### Call a Tool
- **POST** `/tools/call`
- Execute a specific tool with provided arguments

**Request:**
```json
{
  "tool_name": "system_info",
  "arguments": {
    "detailed": true
  }
}
```

**Response:**
```json
{
  "success": true,
  "content": [
    {
      "type": "text",
      "text": "System: Ubuntu 22.04, CPU: 8 cores, Memory: 16GB"
    }
  ],
  "error": null
}
```

### OpenAPI Documentation
- **GET** `/openapi.json`
- Returns the complete OpenAPI 3.0 specification
- Use with Swagger UI or other API documentation tools

## ⚙️ Configuration

### Command Line Options

```bash
mcp-http-bridge [OPTIONS]

Options:
    --port <PORT>                    Server port [default: 3001]
    --log-level <LEVEL>             Log level [default: info]
    --mcp-server-path <URL>         MCP server URL [default: http://mcp-server:3002]
    -h, --help                      Print help information
```

### Environment Variables

You can also configure the bridge using environment variables:

```bash
export MCP_HTTP_BRIDGE_PORT=3001
export MCP_HTTP_BRIDGE_LOG_LEVEL=debug
export MCP_SERVER_URL=http://localhost:3002
```

### Log Levels

Available log levels (from most to least verbose):
- `trace` - Very detailed debugging information
- `debug` - Debug information 
- `info` - General information (default)
- `warn` - Warning messages
- `error` - Error messages only

## 🛠️ Development

### Project Structure

```
src/
├── main.rs           # Application entry point and CLI
├── lib.rs            # Library exports and core functionality
├── mcp_client.rs     # MCP server communication
├── openapi.rs        # OpenAPI specification generation
└── tests.rs          # Unit tests

tests/
├── integration_tests.rs  # Integration tests
└── common/
    └── mod.rs            # Test utilities
```

### Building from Source

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Development with auto-reload
cargo watch -x run
```

### Code Quality

```bash
# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy

# Check for security issues
cargo audit
```

## 🧪 Testing

The project includes comprehensive test coverage:

### Unit Tests (22 tests)
- Endpoint functionality testing
- Request/response validation
- Error handling verification
- OpenAPI specification validation

```bash
# Run unit tests only
cargo test --lib
```

### Integration Tests (9 tests)
- End-to-end API workflow testing
- MCP server integration scenarios
- Performance and load testing
- CORS and content-type validation

```bash
# Run integration tests only
cargo test --test integration_tests
```

### Test Coverage

```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

## 🐳 Docker Deployment

### Using Docker Compose

The bridge is designed to work with the broader MCP stack via Docker Compose:

```yaml
version: '3.8'
services:
  mcp-http-bridge:
    build: ./mcp-http-bridge
    ports:
      - "3001:3001"
    environment:
      - MCP_SERVER_URL=http://mcp-server:3002
      - LOG_LEVEL=info
    depends_on:
      - mcp-server
    
  mcp-server:
    build: ./mcp-server
    ports:
      - "3002:3002"
```

### Building Docker Image

```bash
# Build the image
docker build -t mcp-http-bridge .

# Run the container
docker run -p 3001:3001 \
  -e MCP_SERVER_URL=http://host.docker.internal:3002 \
  mcp-http-bridge
```

## 📚 API Documentation

### OpenAPI/Swagger Integration

The bridge automatically generates and serves OpenAPI 3.0 documentation:

1. **Access the specification**: `GET http://localhost:3001/openapi.json`
2. **Use with Swagger UI**: Import the JSON into any Swagger UI instance
3. **Generate client SDKs**: Use OpenAPI generators for various languages

### Example with Swagger UI

```bash
# Run Swagger UI with Docker
docker run -p 8080:8080 \
  -e SWAGGER_JSON=/openapi.json \
  -v $(pwd)/openapi.json:/openapi.json \
  swaggerapi/swagger-ui
```

### Client Examples

#### cURL
```bash
# Health check
curl http://localhost:3001/health

# List tools
curl http://localhost:3001/tools

# Call a tool
curl -X POST http://localhost:3001/tools/call \
  -H "Content-Type: application/json" \
  -d '{"tool_name": "system_info", "arguments": {"detailed": true}}'
```

#### JavaScript/Node.js
```javascript
const response = await fetch('http://localhost:3001/tools/call', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    tool_name: 'system_info',
    arguments: { detailed: true }
  })
});

const result = await response.json();
console.log(result);
```

#### Python
```python
import requests

response = requests.post('http://localhost:3001/tools/call', json={
    'tool_name': 'system_info',
    'arguments': {'detailed': True}
})

result = response.json()
print(result)
```

## 🚨 Error Handling

### HTTP Status Codes

- **200 OK** - Successful operation
- **400 Bad Request** - Invalid request format or missing fields
- **405 Method Not Allowed** - Wrong HTTP method for endpoint
- **404 Not Found** - Endpoint not found
- **500 Internal Server Error** - MCP server communication error

### Error Response Format

All errors return a consistent JSON structure:

```json
{
  "success": false,
  "content": null,
  "error": "Detailed error message here"
}
```

### Common Error Scenarios

1. **MCP Server Unavailable**
   ```json
   {
     "success": false,
     "error": "Failed to connect to MCP server at http://localhost:3002"
   }
   ```

2. **Invalid Tool Name**
   ```json
   {
     "success": false,
     "error": "Tool 'nonexistent_tool' not found"
   }
   ```

3. **Invalid Arguments**
   ```json
   {
     "success": false,
     "error": "Invalid arguments for tool 'system_info': missing required field 'type'"
   }
   ```

## 📊 Monitoring and Observability

### Logging

The bridge provides structured logging with configurable levels:

```bash
# Set detailed logging
cargo run -- --log-level debug

# JSON structured logs for production
RUST_LOG=mcp_http_bridge=info cargo run
```

### Health Monitoring

Use the `/health` endpoint for:
- Load balancer health checks
- Container orchestration readiness probes
- Monitoring system integration

### Metrics

Consider integrating with metrics systems:
- Request count and latency
- Error rates by endpoint
- MCP server response times

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines:

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Write tests** for your changes
4. **Ensure all tests pass**: `cargo test`
5. **Format your code**: `cargo fmt`
6. **Submit a pull request**

### Development Setup

```bash
# Install development dependencies
cargo install cargo-watch cargo-tarpaulin

# Run development server with auto-reload
cargo watch -x "run -- --log-level debug"

# Run tests on file changes
cargo watch -x test
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Axum](https://github.com/tokio-rs/axum) for high-performance HTTP handling
- OpenAPI documentation powered by [utoipa](https://github.com/juhaku/utoipa)
- Testing framework using [tokio-test](https://github.com/tokio-rs/tokio) and [axum-test](https://github.com/JosephLenton/axum-test)

## 🔮 Roadmap

- [ ] WebSocket support for real-time communication
- [ ] Authentication and authorization
- [ ] Rate limiting and request throttling
- [ ] Metrics and monitoring endpoints
- [ ] Plugin system for custom middleware
- [ ] gRPC interface option
- [ ] Load balancing for multiple MCP servers

---

For more information about the Model Context Protocol, visit the [official MCP documentation](https://github.com/modelcontextprotocol).