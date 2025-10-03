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

### Option 1: Custom Function (Recommended)

Create custom functions in Open WebUI that call the MCP server via HTTP:

#### 1. System Information Function

```python
import requests
import json

def get_system_info(info_type: str = "all") -> str:
    """
    Get system information from MCP server.
    
    Args:
        info_type: Type of info to retrieve (cpu, memory, disk, docker, all)
    """
    url = "http://localhost:3001/tools/call"
    payload = {
        "tool_name": "system_info",
        "arguments": {"info_type": info_type}
    }
    
    try:
        response = requests.post(url, json=payload, timeout=30)
        response.raise_for_status()
        result = response.json()
        
        if result.get("success"):
            content = result.get("content", [])
            return content[0].get("text", "No content") if content else "Empty response"
        else:
            return f"Error: {result.get('error', 'Unknown error')}"
    except Exception as e:
        return f"Request failed: {str(e)}"
```

#### 2. Home Assistant Function

```python
import requests
import json

def control_home_assistant(action: str, entity_id: str = None, domain: str = None, service: str = None, service_data: dict = None) -> str:
    """
    Interact with Home Assistant.
    
    Args:
        action: Action to perform (get_states, get_state, call_service, get_services)
        entity_id: Entity ID for get_state
        domain: Service domain for call_service
        service: Service name for call_service
        service_data: Data for service call
    """
    url = "http://localhost:3001/tools/call"
    
    arguments = {"action": action}
    if entity_id:
        arguments["entity_id"] = entity_id
    if domain:
        arguments["domain"] = domain
    if service:
        arguments["service"] = service
    if service_data:
        arguments["service_data"] = service_data
    
    payload = {
        "tool_name": "homeassistant",
        "arguments": arguments
    }
    
    try:
        response = requests.post(url, json=payload, timeout=30)
        response.raise_for_status()
        result = response.json()
        
        if result.get("success"):
            content = result.get("content", [])
            return content[0].get("text", "No content") if content else "Empty response"
        else:
            return f"Error: {result.get('error', 'Unknown error')}"
    except Exception as e:
        return f"Request failed: {str(e)}"
```

#### 3. HTTP Request Function

```python
import requests
import json

def make_http_request(url: str, method: str = "GET", headers: dict = None, body: str = None, timeout: int = 30) -> str:
    """
    Make HTTP requests via MCP server.
    
    Args:
        url: Target URL
        method: HTTP method (GET, POST, PUT, DELETE, PATCH)
        headers: HTTP headers
        body: Request body
        timeout: Timeout in seconds
    """
    mcp_url = "http://localhost:3001/tools/call"
    
    arguments = {
        "url": url,
        "method": method,
        "timeout": timeout
    }
    if headers:
        arguments["headers"] = headers
    if body:
        arguments["body"] = body
    
    payload = {
        "tool_name": "http_request",
        "arguments": arguments
    }
    
    try:
        response = requests.post(mcp_url, json=payload, timeout=timeout + 5)
        response.raise_for_status()
        result = response.json()
        
        if result.get("success"):
            content = result.get("content", [])
            return content[0].get("text", "No content") if content else "Empty response"
        else:
            return f"Error: {result.get('error', 'Unknown error')}"
    except Exception as e:
        return f"Request failed: {str(e)}"
```

### How to Add Functions to Open WebUI:

1. **Access Admin Panel**: Go to Open WebUI admin settings
2. **Navigate to Functions**: Find the "Functions" or "Tools" section
3. **Create New Function**: Add each function with proper metadata
4. **Test Functions**: Verify they work in chat interface

### Option 2: Direct HTTP Integration

If Open WebUI supports external API calls, you can directly integrate with the MCP HTTP bridge:

```bash
# Example API calls
curl -X GET http://localhost:3001/tools
curl -X POST http://localhost:3001/tools/call \
  -H "Content-Type: application/json" \
  -d '{"tool_name": "system_info", "arguments": {"info_type": "memory"}}'
```

## n8n Integration

### Option 1: HTTP Request Node

Use n8n's built-in HTTP Request node to call the MCP server:

#### Workflow Example: System Monitoring

```json
{
  "nodes": [
    {
      "name": "Get System Info",
      "type": "n8n-nodes-base.httpRequest",
      "position": [250, 300],
      "parameters": {
        "url": "http://localhost:3001/tools/call",
        "method": "POST",
        "sendHeaders": true,
        "headerParameters": {
          "parameters": [
            {
              "name": "Content-Type",
              "value": "application/json"
            }
          ]
        },
        "sendBody": true,
        "bodyContentType": "json",
        "jsonBody": "{\n  \"tool_name\": \"system_info\",\n  \"arguments\": {\n    \"info_type\": \"all\"\n  }\n}",
        "options": {}
      }
    }
  ]
}
```

#### Workflow Example: Home Assistant Control

```json
{
  "nodes": [
    {
      "name": "Turn On Lights",
      "type": "n8n-nodes-base.httpRequest",
      "position": [250, 300],
      "parameters": {
        "url": "http://localhost:3001/tools/call",
        "method": "POST",
        "sendHeaders": true,
        "headerParameters": {
          "parameters": [
            {
              "name": "Content-Type",
              "value": "application/json"
            }
          ]
        },
        "sendBody": true,
        "bodyContentType": "json",
        "jsonBody": "{\n  \"tool_name\": \"homeassistant\",\n  \"arguments\": {\n    \"action\": \"call_service\",\n    \"domain\": \"light\",\n    \"service\": \"turn_on\",\n    \"service_data\": {\n      \"entity_id\": \"light.living_room\"\n    }\n  }\n}",
        "options": {}
      }
    }
  ]
}
```

### Option 2: Custom n8n Node (Advanced)

For more advanced integration, create a custom n8n node:

#### Node Structure:
```javascript
// MCPServerNode.js
class MCPServerNode {
    description = {
        displayName: 'MCP Server',
        name: 'mcpServer',
        group: ['transform'],
        version: 1,
        description: 'Interact with MCP Server tools',
        defaults: {
            name: 'MCP Server',
        },
        inputs: ['main'],
        outputs: ['main'],
        properties: [
            {
                displayName: 'Tool',
                name: 'tool',
                type: 'options',
                options: [
                    {
                        name: 'System Info',
                        value: 'system_info',
                    },
                    {
                        name: 'Home Assistant',
                        value: 'homeassistant',
                    },
                    {
                        name: 'HTTP Request',
                        value: 'http_request',
                    },
                ],
                default: 'system_info',
            }
        ],
    };
    
    async execute(context) {
        const tool = context.getNodeParameter('tool', 0);
        const items = context.getInputData();
        
        // Implementation here
    }
}
```

## Usage Examples

### 1. Monitor System Resources

**Open WebUI Chat:**
```
Hey, check the current memory usage on the server.
```

**n8n Workflow:**
- Trigger: Schedule (every 5 minutes)
- Action: Call system_info tool
- Condition: If memory usage > 80%
- Action: Send alert

### 2. Control Smart Home

**Open WebUI Chat:**
```
Turn on the living room lights and set them to 50% brightness.
```

**n8n Workflow:**
- Trigger: Webhook from motion sensor
- Action: Call Home Assistant service
- Result: Lights turn on automatically

### 3. External API Integration

**Open WebUI Chat:**
```
Get the latest weather data for Seattle.
```

**n8n Workflow:**
- Trigger: Daily schedule
- Action: HTTP request via MCP server
- Action: Store weather data in database

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