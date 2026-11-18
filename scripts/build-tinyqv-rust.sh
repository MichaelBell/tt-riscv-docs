#!/usr/bin/env bash
# Build examples/rust/tinyqv-hello (flash image for offset 0).
# Usage: scripts/build-tinyqv-rust.sh [out-dir]   (default: out/tinyqv-rust)
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT=${1:-$ROOT/out/tinyqv-rust}
mkdir -p "$OUT" && OUT=$(cd "$OUT" && pwd)  # absolute: the script changes directory
"$ROOT/examples/rust/tinyqv-hello/build.sh"
mkdir -p "$OUT"
cp "$ROOT/examples/rust/tinyqv-hello/tinyqv-hello.bin" "$OUT/"
(cd "$OUT" && sha256sum tinyqv-hello.bin > SHA256SUMS && ls -l)
