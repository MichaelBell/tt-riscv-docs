#!/usr/bin/env bash
# Download Mike Bell's TinyQV MicroPython image for ttsky25a and check its sha256.
# Usage: scripts/fetch-tinyqv-micropython.sh [out-dir]   (default: out/tinyqv-micropython)
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
. "$ROOT/pins.env"
OUT=${1:-$ROOT/out/tinyqv-micropython}
mkdir -p "$OUT" && OUT=$(cd "$OUT" && pwd)  # absolute: the script changes directory
mkdir -p "$OUT"
F=$OUT/$(basename "$TINYQV_MICROPYTHON_URL")
curl -fsSL -o "$F" "$TINYQV_MICROPYTHON_URL"
echo "$TINYQV_MICROPYTHON_SHA256  $F" | sha256sum -c -
