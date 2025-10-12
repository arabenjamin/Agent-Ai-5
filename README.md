# Agent-Ai-5
This is a playground for experimenting with the Model Context Protocol (MCP) stack. There be dragons here, you have been warned.

In this repository, you will find an experimental tooling system that integrates Ollama, n8n, Open Web UI, Home Assistant, and other services through a Model Context Protocol (MCP) server architecture. As well as a local development CLI tool for interacting with Ollama models and MCP server. This is mostly for research and development purposes... mostly. 

# Model Context Protocol (MCP) Stack

## System Architecture

The MCP system consists of three main components:

### 1. MCP Server
- Core server that provides tool execution and context management
- Runs in Docker container via docker-compose
- Exposes JSON-RPC 2.0 interface on port 3002
- Manages various tools including system info, HTTP requests, and Home Assistant integration
- Handles context and rule processing for AI interactions

### 2. MCP HTTP Bridge
- REST API gateway to the MCP server
- Runs in Docker container via docker-compose
- Exposes HTTP endpoints on port 3001:
  - `GET /health` - Health check
  - `GET /tools` - List available tools
  - `POST /tools/call` - Execute tools
- Translates HTTP requests to JSON-RPC 2.0 calls for the MCP server

### 3. MCP Client (Local Development Tool)
- CLI tool for local development and testing
- Runs locally (not in Docker)
- Connects to both Ollama and MCP server
- Provides commands for:
  - Listing available tools
  - Calling specific tools
  - Interacting with Ollama models
  - Testing integrations

## Additional Services

### Ollama
- Local AI model server
- Runs on port 11434
- Provides API for model interaction
- Used by Open Web UI for model execution

### Open Web UI
- Web interface for Ollama interaction
- Runs on port 8080
- Supports custom functions for MCP integration
- See [Integration Guide](MCP-INTEGRATION-GUIDE.md) for examples

### n8n
- Automation workflow platform
- Runs on port 5678
- Can integrate with MCP through HTTP requests
- Enables complex AI-powered workflows

### Home Assistant
- Smart home automation platform
- Runs on port 8123
- Integrated through MCP tools
- Enables AI control of smart devices

## Setup Instructions

1. Prerequisites:
   ```bash
   - Docker and Docker Compose
   - Rust (for MCP client)
   - Git
   ```

2. Clone the repository:
   ```bash
   git clone <repository-url>
   cd ollama-n8n-stack
   ```

3. Start the Docker services:
   ```bash
   docker compose up -d
   ```

4. Build and run the MCP client locally:
   ```bash
   cd mcp-client
   cargo build
   cargo run -- --help
   ```

## Integration Guide

For detailed information on integrating with Open WebUI, n8n, or other services, see our [MCP Integration Guide](MCP-INTEGRATION-GUIDE.md).

Example integrations include:
- Custom functions in Open WebUI
- HTTP workflow nodes in n8n
- Direct API calls from any service

## Troubleshooting

Common issues and solutions:

1. Connection Issues
   - Check if all services are running: `docker compose ps`
   - Verify ports are not in use
   - Check network connectivity between containers

2. Tool Execution Problems
   - Verify MCP server is running: `curl http://localhost:3001/health`
   - Check tool availability: `curl http://localhost:3001/tools`
   - Review server logs: `docker compose logs mcp-server`

3. Integration Issues
   - Verify HTTP bridge is accessible
   - Check correct endpoints and payload format
   - Review the Integration Guide for proper setup

4. Client Issues
   - Ensure correct URLs are configured
   - Check connection to both Ollama and MCP server
   - Review client logs with increased verbosity

## CI/CD Pipeline

This repository includes a comprehensive GitHub Actions CI/CD pipeline that ensures code quality, security, and reliable deployments.

### Workflows Overview

#### 1. Continuous Integration (`ci.yml`)
- **Triggers**: Push to main branches, pull requests
- **Jobs**:
  - **Test Suite**: Multi-matrix testing across Rust versions (stable, beta) and all projects
  - **Security Audit**: Automated security vulnerability scanning with `cargo-audit`
  - **Code Coverage**: Comprehensive coverage reporting with `cargo-llvm-cov`
  - **Multi-Platform Builds**: Binary compilation for Linux, Windows, macOS
  - **Docker Builds**: Container image building and caching
  - **Integration Tests**: End-to-end testing with Docker services

#### 2. Release Automation (`release.yml`)
- **Triggers**: Git tags matching `v*` pattern
- **Jobs**:
  - **Multi-Platform Releases**: Automated binary releases for all supported platforms
  - **Docker Publishing**: Multi-architecture container images to Docker Hub and GHCR
  - **Crate Publishing**: Automated publishing to crates.io
  - **Release Notes**: Automatic GitHub release creation with changelogs

#### 3. Code Quality (`quality.yml`)
- **Triggers**: Push, PR, weekly schedule
- **Jobs**:
  - **Lint & Format**: Strict code formatting and advanced Clippy linting
  - **Documentation**: Doc generation, testing, and GitHub Pages deployment
  - **Dependency Analysis**: License compliance and security checks with `cargo-deny`
  - **MSRV Testing**: Minimum Supported Rust Version compatibility
  - **Performance Benchmarking**: Automated performance regression detection

#### 4. Dependency Management (`dependencies.yml`)
- **Triggers**: Weekly schedule, manual dispatch
- **Jobs**:
  - **Automated Updates**: Dependency version updates with testing
  - **Security Monitoring**: Continuous vulnerability scanning
  - **Update PRs**: Automatic pull request creation for dependency updates

### Setup Requirements

#### GitHub Repository Secrets

Configure these secrets in your GitHub repository settings:

```bash
# Docker publishing
DOCKER_USERNAME       # Docker Hub username
DOCKER_PASSWORD       # Docker Hub access token

# Crate publishing  
CRATES_IO_TOKEN      # crates.io API token

# The following are automatically provided by GitHub:
# GITHUB_TOKEN         # GitHub API access (automatic)
```

#### Repository Permissions

Enable the following in your repository settings:
- **Actions > General > Workflow permissions**: Read and write permissions
- **Pages > Source**: GitHub Actions
- **Security > Code scanning**: Enable Dependabot alerts

### Local Development

#### Pre-commit Hooks

Set up local pre-commit hooks to match CI requirements:

```bash
# Install pre-commit
pip install pre-commit

# Set up hooks (create .pre-commit-config.yaml)
pre-commit install

# Run hooks manually
pre-commit run --all-files
```

#### Quality Checks

Run the same checks locally that CI performs:

```bash
# Format check
cargo fmt --check

# Advanced linting
cargo clippy --all-targets --all-features -- -D warnings

# Security audit
cargo audit

# Test with coverage
cargo llvm-cov --workspace --html

# Documentation
cargo doc --no-deps --document-private-items
```

### Release Process

#### Creating a Release

1. **Update Version Numbers**:
   ```bash
   # Update Cargo.toml versions in each project
   cd mcp-client && cargo release patch --dry-run
   cd ../mcp-http-bridge && cargo release patch --dry-run  
   cd ../mcp-server && cargo release patch --dry-run
   ```

2. **Create and Push Tag**:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

3. **Automated Release**: The release workflow automatically:
   - Creates GitHub release with binaries
   - Publishes Docker images
   - Publishes crates to crates.io
   - Generates release notes

#### Version Strategy

- **Semantic Versioning**: `MAJOR.MINOR.PATCH`
- **Tag Format**: `v1.0.0`
- **Branch Strategy**: 
  - `master`: Stable releases
  - `develop`: Development branch
  - Feature branches: `feature/feature-name`

### Monitoring and Maintenance

#### Automated Monitoring

- **Dependency Updates**: Weekly automated checks and PR creation
- **Security Scanning**: Continuous vulnerability monitoring
- **Performance Tracking**: Benchmark comparisons on releases
- **Documentation**: Automatic doc updates and deployment

#### Manual Tasks

- **Release Notes**: Review and enhance automated release notes
- **Security Issues**: Address any security advisories promptly
- **Performance Regressions**: Investigate benchmark failures
- **Dependency Conflicts**: Resolve complex dependency updates

## Future Development

### TODOs
1. Testing Infrastructure
   - Set up CI/CD pipeline

2. Tool Validation
   - Systematic testing of all tools
   - Documentation of tool behaviors
   - Performance benchmarking

3. Security Enhancements
   - Implement secret management
   - Add authentication/authorization
   - Secure sensitive integrations

4. Feature Development
   - Plugin architecture for tools
   - Expand context processing
   - Implement rule engine
   - Add more tool capabilities

5. Documentation

   - Expand integration examples
   - Add architectural diagrams

## License

This project is licensed under the MIT License.