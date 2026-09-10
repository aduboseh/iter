# Local/RC image. Run an explicit runtime mode; this is not GA certification.
FROM rust:1.93.0-slim-bookworm@sha256:776861219cd851131c1cec3bbd7cbeb16b99a794048097eb69ad9682a8ed0d57 AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY vendor/governance-bridge ./vendor/governance-bridge
COPY governance ./governance
RUN cargo build --locked --release --no-default-features --features public_stub \
    --bin iter-server --bin iter-cli --bin iter-verify

FROM debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 1000 --create-home iter \
    && mkdir -p /var/lib/iter && chown iter:iter /var/lib/iter
WORKDIR /app
COPY --from=builder /build/target/release/iter-server /build/target/release/iter-cli /build/target/release/iter-verify /usr/local/bin/
COPY --from=builder /build/governance/governance.hash /app/governance/governance.hash
USER 1000:1000
# MCP uses stdin/stdout, not HTTP. No fake HTTP port or process-only health check.
ENTRYPOINT ["/usr/local/bin/iter-server"]
