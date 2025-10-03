# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

This is a Docker Compose-based stack that orchestrates multiple AI and home automation services:
- **Home Assistant**: Home automation platform (port 8123)
- **Open WebUI**: Web interface for AI models with CUDA GPU support (port 3000)
- **n8n**: Workflow automation platform (port 5678)
- **Ollama**: Local LLM server (currently commented out, port 11434)
- **MCP Server**: Model Context Protocol server in Rust for tool integration

The stack is designed for local development and includes Python tools for integrating with Google APIs and weather services.

## Common Commands

### Docker Compose Operations
```bash
# Start all services
docker compose up -d

# Start services and rebuild
docker compose up --build -d

# Stop all services
docker compose down

# View logs for all services
docker compose logs -f

# View logs for specific service
docker compose logs -f open-webui
docker compose logs -f n8n
docker compose logs -f homeassistant

# Restart a specific service
docker compose restart open-webui

# Build and run MCP server
docker compose build mcp-server
docker compose up mcp-server -d

# View MCP server logs
docker compose logs -f mcp-server

# Test MCP server interactively
docker compose exec mcp-server /app/mcp-server --stdio
```

### Service Access
- Home Assistant: http://localhost:8123
- Open WebUI: http://localhost:3000
- n8n: http://localhost:5678
- MCP Server: STDIO-based (use docker exec for interaction)

### Python Development
```bash
# Activate virtual environment (if using .env directory as venv)
source .env/bin/activate

# Run the main Python script
python main.py

# Install dependencies (uses uv-based environment)
uv pip install <package_name>
```

### Documentation (Docusaurus)
```bash
# Navigate to docs directory
cd docs/docs-main

# Install dependencies
npm install

# Start development server
npm start

# Build documentation
npm run build

# Lint and format
npm run lint
npm run prettier
```

## Architecture & Structure

### Service Configuration
- **GPU Support**: Open WebUI is configured with NVIDIA CUDA runtime for GPU acceleration
- **Network Mode**: All services use `host` networking for simplified local communication
- **Data Persistence**: Service data stored in `./data/` directory with bind mounts
- **Environment Variables**: Configuration via `.env.dev` file for sensitive data

### Key Components

#### Docker Services (`docker-compose.yml`)
- Home Assistant: Full-featured home automation with privileged access
- Open WebUI: CUDA-enabled AI chat interface with Ollama integration
- n8n: Workflow automation with root privileges for system access
- Ollama: Currently disabled but configured for local LLM hosting
- MCP Server: Rust-based Model Context Protocol server with tools for system info, HTTP requests, and Home Assistant integration

#### Python Tools (`tools/`)
- `basic_tools.py`: Google OAuth authentication and calendar integration
- `weather_tool.py`: OpenWeatherMap API integration with geocoding
- Event emission system for real-time progress updates

#### Configuration Management
- Environment variables in `.env.dev` (Google OAuth, weather API keys)
- Service-specific configs in `config/` directory
- Token storage for OAuth flows in root directory

### Data Flow
1. Services communicate via host networking (localhost)
2. Open WebUI connects to Ollama API at `localhost:11434`
3. Python tools provide external API integrations (Google, Weather)
4. Home Assistant manages local IoT devices and automation

## Environment Setup

### Required Environment Variables (.env.dev)
```bash
GOOGLE_CLINETID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret
OPENWEATHERMAP_API_KEY=your-weather-api-key
HOMEASSISTANT_TOKEN=your-home-assistant-token
```

### GPU Requirements
- NVIDIA Docker runtime for Open WebUI CUDA support
- GPU drivers with compute capabilities

### Port Availability
Ensure ports 3000, 5678, and 8123 are not in use by other applications.

## Development Notes

### Service Integration
- Open WebUI is pre-configured to connect to Ollama at `localhost:11434`
- n8n workflows can interact with all services via localhost
- Home Assistant can be integrated with external APIs through Python tools

### OAuth Flow
The Google authentication in `basic_tools.py` uses a local server callback flow with redirect URI `http://localhost:8080/oauth/google/callback/`.

### Logging
- Python tools use structured logging with DEBUG level
- Docker services log to stdout/stderr (viewable via `docker compose logs`)

### Data Persistence
- All service data persists in `./data/` directory
- Docker volumes for internal application data
- Configuration files in `config/` directory