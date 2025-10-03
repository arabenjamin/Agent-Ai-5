# Build stage
FROM rust:1.82-slim as builder

# Install system dependencies needed for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    docker.io \
    util-linux \
    procps \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user and add to docker group
RUN groupadd -r mcpuser && useradd -r -g mcpuser -G docker mcpuser

# Create app directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /usr/src/app/target/release/mcp-server /app/mcp-server

# Make binary executable
RUN chmod +x /app/mcp-server

# Change ownership to mcpuser
RUN chown -R mcpuser:mcpuser /app

# Switch to non-root user
USER mcpuser

# Expose port (though MCP typically uses stdio)
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pgrep mcp-server || exit 1

# Default command
CMD ["./mcp-server", "--stdio"]