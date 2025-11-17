#!/bin/bash
# SCG-PILOT-01 Substrate Activation Script
# Directive: SG-SCG-PILOT-ACT-01 v1.0.0 §2.1
#
# Generates continuous synthetic MCP requests to drive substrate processing
# and enable telemetry emission to OTEL collector

set -e

echo "================================================"
echo "SCG-PILOT-01 Substrate Activation"
echo "Directive: SG-SCG-PILOT-ACT-01 v1.0.0"
echo "================================================"
echo ""
echo "Starting substrate with synthetic request stream..."
echo "OTEL Endpoint: ${OTEL_EXPORTER_OTLP_ENDPOINT}"
echo ""

# Function to generate continuous MCP requests
generate_requests() {
    local request_id=1
    local cycle=0
    
    while true; do
        cycle=$((cycle + 1))
        
        # Cycle through different request types to exercise substrate
        case $((cycle % 10)) in
            0)
                # Node creation (Belief + Energy)
                echo "{\"jsonrpc\":\"2.0\",\"id\":${request_id},\"method\":\"node.create\",\"params\":{\"belief\":0.5,\"energy\":1.0}}"
                ;;
            1)
                # Node mutation
                echo "{\"jsonrpc\":\"2.0\",\"id\":${request_id},\"method\":\"node.mutate\",\"params\":{\"node_id\":1,\"belief\":0.7}}"
                ;;
            2)
                # Edge binding
                echo "{\"jsonrpc\":\"2.0\",\"id\":${request_id},\"method\":\"edge.bind\",\"params\":{\"source\":1,\"target\":2}}"
                ;;
            3)
                # Edge propagation
                echo "{\"jsonrpc\":\"2.0\",\"id\":${request_id},\"method\":\"edge.propagate\",\"params\":{\"source\":1,\"target\":2}}"
                ;;
            4)
                # Query state
                echo "{\"jsonrpc\":\"2.0\",\"id\":${request_id},\"method\":\"state.query\",\"params\":{}}"
                ;;
            5)
                # ESV validation
                echo "{\"jsonrpc\":\"2.0\",\"id\":${request_id},\"method\":\"esv.validate\",\"params\":{\"node_id\":1}}"
                ;;
            6)
                # Governor correction check
                echo "{\"jsonrpc\":\"2.0\",\"id\":${request_id},\"method\":\"governor.check\",\"params\":{}}"
                ;;
            7)
                # Lineage query
                echo "{\"jsonrpc\":\"2.0\",\"id\":${request_id},\"method\":\"lineage.query\",\"params\":{\"depth\":10}}"
                ;;
            8)
                # Shard finalization
                echo "{\"jsonrpc\":\"2.0\",\"id\":${request_id},\"method\":\"shard.finalize\",\"params\":{}}"
                ;;
            9)
                # Global hash reconstruction
                echo "{\"jsonrpc\":\"2.0\",\"id\":${request_id},\"method\":\"ledger.reconstruct\",\"params\":{}}"
                ;;
        esac
        
        request_id=$((request_id + 1))
        
        # Rate limiting: ~100 requests/sec to stay within RPS target
        sleep 0.01
        
        # Log progress every 1000 requests
        if [ $((request_id % 1000)) -eq 0 ]; then
            echo "[ACTIVATION] Sent ${request_id} requests (cycle ${cycle})" >&2
        fi
    done
}

# Start substrate server with request generator piped to stdin
echo "[ACTIVATION] Launching substrate with synthetic request stream..."
echo "[ACTIVATION] Target RPS: ~100 (well below 7500 limit)"
echo ""

# Generate requests and pipe to substrate
generate_requests | /app/scg_mcp_server
