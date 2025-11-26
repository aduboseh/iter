# SCG Demo Package - Docker Reproducibility

## Prerequisites

- Docker 20.10+ installed
- Linux binary of `scg_mcp_server` (cross-compile if needed)

## Cross-Compile Server (if on Windows/macOS)

```bash
# Install Linux target
rustup target add x86_64-unknown-linux-gnu

# Build for Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Copy binary to package
cp target/x86_64-unknown-linux-gnu/release/scg_mcp_server scg_demo_package/
```

## Build

```bash
cd scg_demo_package
docker build -t scg-demo-package:v1.0 .
```

**Expected:** Build completes with zero warnings.

## Run

```bash
docker run --rm scg-demo-package:v1.0
```

**Expected Output:**
```
[SCG-DEMO] SCG Substrate Demo v1.0 (Production Edition)
[SCG-DEMO] Determinism mode: enabled
[SCG-DEMO] Locale: LC_ALL=C
...
[SCG-DEMO] DETERMINISM VERIFIED
[SCG-DEMO] All invariant artifacts match across runs
Container validation complete
```

## Extract Artifacts

```bash
# Create output directory
mkdir -p ./output

# Run and extract
docker run --rm -v $(pwd)/output:/output scg-demo-package:v1.0 \
  bash -c "./demos/scg_demo.sh && cp -r demo_runs/run_1/demo_output /output"
```

Artifacts appear in `./output/demo_output/`.

## Verify Checksums from Container

```bash
docker run --rm scg-demo-package:v1.0 \
  bash -c "./demos/scg_demo.sh && cat demo_runs/run_1/demo_output/07_checksums.txt"
```

## Interactive Debugging

```bash
docker run --rm -it scg-demo-package:v1.0 bash
# Inside container:
./demos/scg_demo.sh
cat demo_runs/run_1/demo_output/01_start.log | jq .
```

## Image Size Validation

```bash
docker images scg-demo-package:v1.0 --format "{{.Size}}"
```

**Expected:** <100MB (minimal dependencies).

## Clean Up

```bash
docker rmi scg-demo-package:v1.0
docker system prune -f
```
