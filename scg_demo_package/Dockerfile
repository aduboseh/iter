FROM ubuntu:22.04

# Minimal POSIX environment for deterministic execution
RUN apt-get update && apt-get install -y \
    bash \
    coreutils \
    jq \
    bc \
    && rm -rf /var/lib/apt/lists/*

# Set deterministic environment variables
ENV SCG_TIMESTAMP_MODE=deterministic \
    SCG_DETERMINISM=1 \
    LC_ALL=C \
    LANG=C \
    TZ=UTC

WORKDIR /scg_demo

# Copy demo artifacts
COPY demos/ ./demos/
COPY demo_expected/ ./demo_expected/

# Copy MCP server binary (must be Linux binary)
# Note: Build with: cargo build --release --target x86_64-unknown-linux-gnu
COPY scg_mcp_server /usr/local/bin/scg_mcp_server

# Set permissions
RUN chmod +x /usr/local/bin/scg_mcp_server && \
    chmod +x ./demos/scg_demo.sh

# Create output directories
RUN mkdir -p demo_runs/run_1/demo_output demo_runs/run_2/demo_output

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD test -x /usr/local/bin/scg_mcp_server || exit 1

# Default command runs demo and validates checksums
CMD ["bash", "-c", "./demos/scg_demo.sh && echo 'Container validation complete'"]
