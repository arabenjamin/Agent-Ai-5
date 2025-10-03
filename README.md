# Agent Ai5

This is Ara's Ai and Ai agent playground. There be drangons. 

# Ollama, n8n, and Open Web UI Stack, and custom mcp-server and client

This project provides a Docker Compose setup for running Ollama, n8n, and Open Web UI together, along with custom mcp-server and client services.


## Prerequisites

- Docker
- Docker Compose
- Git
- Ollama running locally
- Rust (for building the mcp-client)


## Setup Instructions

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd ollama-n8n-stack
   ```

2. Configure environment variables:
   - Update the `.env` file with your specific configurations.

3. Build and run the containers:
   ```bash
   docker-compose up --build
   ```

4. Access the services:
   - Ollama: [http://localhost:port](http://localhost:port)
   - n8n: [http://localhost:5678](http://localhost:5678)
   - Open Web UI: [http://localhost:port](http://localhost:port)

## Usage Guidelines

- Ensure that the necessary ports are open and not in use by other applications.
- Refer to the individual service documentation for more detailed usage instructions.

## License

This project is licensed under the MIT License.