#!/usr/bin/env bash
# Build the KianV 16 MiB PSRAM memtest (flash at 0x100000 in place of the Linux bootloader).
# Needs gcc-riscv64-unknown-elf (Ubuntu/Debian package).
# Usage: scripts/build-memtest.sh [out-dir]   (default: out/kianv-memtest)
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
. "$ROOT/pins.env"
OUT=${1:-$ROOT/out/kianv-memtest}
mkdir -p "$OUT" && OUT=$(cd "$OUT" && pwd)  # absolute: the script changes directory
SRC=$ROOT/build/kianv-memtest
[ -d "$SRC/.git" ] || git clone "$MEMTEST_REPO" "$SRC"
git -C "$SRC" -c advice.detachedHead=false checkout -q "$MEMTEST_COMMIT"
make -C "$SRC/src" CPU_FREQ="$KIANV_CPU_FREQ" BAUDRATE="$KIANV_BAUDRATE"
mkdir -p "$OUT"
cp "$SRC/src/memtest.bin" "$SRC/src/memtest.elf" "$OUT/"
(cd "$OUT" && sha256sum memtest.bin memtest.elf > SHA256SUMS && ls -l)
