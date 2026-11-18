#!/bin/bash
# SPDX-License-Identifier: MIT
# Build Rust programs that use the full standard library for KianV uLinux
# (RV32IMA without C, no MMU, uClibc-ng, FLAT binaries).
#
# Rust has no riscv32 uClibc target, so this uses:
#   - riscv32ima-unknown-linux-uclibc.json, a custom target spec;
#   - -Zbuild-std on a patched copy of the nightly rust-src
#     (patches/std-*.patch: time32 futex, no stack-overflow handler without MMU);
#   - a patched libc crate (patches/libc-*.patch: riscv32 module for uClibc-ng);
#   - kianv_pthread.c, a small pthread layer (uClibc is built without threads);
#   - kianv-flat-cc, a linker wrapper that links through elf2flt.
#
# usage: build.sh <buildroot host dir> <install dir>
# Needs rustup with $RUST_STD_TOOLCHAIN (+ rust-src) and curl.
set -euo pipefail

HOST_DIR=$1
INSTALL_DIR=$2
HERE="$(cd "$(dirname "$0")" && pwd)"
TOOLCHAIN=${RUST_STD_TOOLCHAIN:-nightly-2026-11-17}
LIBC_VERSION=0.2.191
LIBC_SHA256=65cd87c9a6d60536580b316ad985726104730a5151c99845f2490428ef91240a
WORK=${RUST_STD_WORK:-$HERE/.work}
TARGET=riscv32ima-unknown-linux-uclibc
STACK_SIZE=131072

mkdir -p "$WORK"
if ! rustc "+$TOOLCHAIN" -V >/dev/null 2>&1; then
    echo "rust_std: toolchain $TOOLCHAIN not installed, using nightly" >&2
    TOOLCHAIN=nightly
fi

# 1. Patched copy of the standard library sources (redone when the toolchain changes).
sysroot=$(rustc "+$TOOLCHAIN" --print sysroot)
stamp="$(rustc "+$TOOLCHAIN" -V) libc-$LIBC_VERSION $(cat "$HERE"/patches/*.patch | sha256sum | cut -c1-16)"
if [ "$(cat "$WORK/stamp" 2>/dev/null)" != "$stamp" ]; then
    rm -rf "$WORK/src" "$WORK/libc" "$WORK/target"
    mkdir -p "$WORK/src/rust"
    cp -a "$sysroot/lib/rustlib/src/rust/library" "$WORK/src/rust/"
    patch -d "$WORK/src/rust/library" -p1 < "$HERE/patches/std-riscv32-uclibc-nommu.patch"

    curl -sSfL -o "$WORK/libc.crate" "https://static.crates.io/crates/libc/libc-$LIBC_VERSION.crate"
    echo "$LIBC_SHA256  $WORK/libc.crate" | sha256sum -c -
    mkdir -p "$WORK/libc"
    tar -xzf "$WORK/libc.crate" -C "$WORK/libc" --strip-components=1
    patch -d "$WORK/libc" -p1 < "$HERE/patches/libc-$LIBC_VERSION-riscv32-uclibc.patch"
    printf "libc = { path = '%s' }\n" "$WORK/libc" >> "$WORK/src/rust/library/Cargo.toml"
    echo "$stamp" > "$WORK/stamp"
fi

# 2. pthread layer, linked into every program by kianv-flat-cc.
export KIANV_CC="$HOST_DIR/bin/riscv32-buildroot-linux-uclibc-gcc"
export KIANV_SHIM="$WORK/kianv_pthread.o"
"$KIANV_CC" -march=rv32ima -mabi=ilp32 -Os -fPIE -ffunction-sections -fdata-sections \
    -Wall -c "$HERE/kianv_pthread.c" -o "$KIANV_SHIM"

# 3. Build each crate with build-std, then give the FLAT binary a 128 KiB stack.
export __CARGO_TESTS_ONLY_SRC_ROOT="$WORK/src/rust/library"
export RUSTFLAGS="-C linker=$HERE/kianv-flat-cc"
install -d "$INSTALL_DIR"
for crate in hello demo; do
    name=$(sed -n 's/^name = "\(.*\)"/\1/p' "$HERE/$crate/Cargo.toml" | head -1)
    (cd "$HERE/$crate" && cargo "+$TOOLCHAIN" build --release \
        -Zbuild-std=std,panic_abort -Zjson-target-spec \
        --target "$HERE/$TARGET.json" --target-dir "$WORK/target")
    install -m 0755 "$WORK/target/$TARGET/release/$name" "$INSTALL_DIR/$name"
    "$HOST_DIR/bin/riscv32-buildroot-linux-uclibc-flthdr" -s "$STACK_SIZE" "$INSTALL_DIR/$name"
done
