# MCP Server Integration Guide

This guide explains how to connect to the MCP (Model Context Protocol) server from Open WebUI and n8n.

## Overview

The MCP server provides three main tools:
- **System Information**: Get CPU, memory, disk usage, and Docker container status
- **HTTP Requests**: Make external API calls with full HTTP method support  
- **Home Assistant Integration**: Control and monitor smart home devices

## Connection Methods

### Method 1: HTTP API Bridge (Recommended)

The HTTP bridge exposes the MCP server functionality via REST API at `http://localhost:3001`.

**Available Endpoints:**
- `GET /health` - Health check
- `GET /tools` - List all available tools
- `POST /tools/call` - Execute a tool

## Open WebUI Integration


## n8n Integration



## Usage Examples

### 1. Monitor System Resources


### 2. Control Smart Home

*

### 3. External API Integration



## Troubleshooting

### Common Issues:

1. **Connection Refused**
   - Check if MCP services are running: `docker compose ps`
   - Verify port 3001 is accessible: `curl http://localhost:3001/health`

2. **Tool Execution Fails**
   - Check MCP server logs: `docker compose logs mcp-server`
   - Verify arguments format in API calls

3. **Home Assistant Integration Issues**
   - Ensure `HOMEASSISTANT_TOKEN` is set in `.env.dev`
   - Check Home Assistant is accessible at `http://localhost:8123`

### Debug Commands:

```bash
# Check service status
docker compose ps

# View logs
docker compose logs mcp-server
docker compose logs mcp-http-bridge

# Test API directly
curl -s http://localhost:3001/health
curl -s http://localhost:3001/tools

# Test tool execution
curl -X POST http://localhost:3001/tools/call \
  -H "Content-Type: application/json" \
  -d '{"tool_name": "system_info", "arguments": {"info_type": "memory"}}'
```

## Advanced Configuration

### Custom Tool Development

To add new tools to the MCP server:

1. Edit `mcp-server/src/tools/mod.rs`
2. Create new tool implementation
3. Register tool in `ToolRegistry::new()`
4. Rebuild and restart services

### Security Considerations

- The MCP server runs with Docker socket access (read-only)
- Home Assistant token is passed via environment variables
- HTTP bridge has CORS enabled for local development
- Consider adding authentication for production use

## Performance Notes

- Each tool call spawns a new MCP server process (by design)
- Tools are stateless and can be called concurrently
- HTTP bridge adds ~10ms overhead per request
- Home Assistant calls depend on HA response time