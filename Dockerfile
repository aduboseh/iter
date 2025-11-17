# SCG-MCP Substrate v1.0.0 - Production Dockerfile
# Directive: SG-SCG-PILOT-AUTH-01 v1.2.0
# 
# Multi-stage build for optimized production image

# Stage 1: Build
FROM rust:1.83-slim as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY tests ./tests

# Build release binary
RUN cargo build --release --bin scg_mcp_server

# Strip binary to reduce size
RUN strip /build/target/release/scg_mcp_server

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -m -u 1000 scg && \
    mkdir -p /var/log/scg /var/lib/scg && \
    chown -R scg:scg /var/log/scg /var/lib/scg

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/scg_mcp_server /app/scg_mcp_server

# Switch to non-root user
USER scg

# Expose port (if needed for HTTP transport in future)
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD pgrep scg_mcp_server || exit 1

# Labels for traceability
LABEL scg.version="v1.0.0-substrate" \
      scg.commit="ab48747" \
      scg.mission="pilot-01" \
      scg.directive="SG-SCG-PILOT-AUTH-01-v1.2.0" \
      maintainer="armonti.dubo.sehill@only-sg.systems"

# Run SCG-MCP server
ENTRYPOINT ["/app/scg_mcp_server"]
