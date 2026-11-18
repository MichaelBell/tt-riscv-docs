# Rust programs on the KianV Linux

The [KianV Linux image](kianv-linux.md) includes `/root/rust_nostd_hello`, a `no_std` Rust program
running as a normal Linux process:

```
/ # /root/rust_nostd_hello a b
Hello from Rust on Tiny Tapeout KianV
first 16 primes: 2 3 5 7 11 13 17 19 23 29 31 37 41 43 47 53
sum = 381, 2^31/7 = 306783378, argc = 3
ui_in = 0x08
7-seg: rust 0-9 (UART TX muted ~7 s)
Rust done, uo released
```

It writes "rust" and then 0-9 on the 7-segment display through the GPIO registers. A program can do
that directly from userspace, because the SoC has no MMU. Tested on ttsky25a.

## How it is built

KianV Linux is unusual for Rust:

- No MMU.
- uClibc instead of glibc/musl.
- RV32IMA without compressed instructions.
- Programs are in the FLAT format, made with `elf2flt`.

There is no Rust `std` target for it, so the program is built as a `no_std` static library and
linked like a C program:

1. `cargo +nightly-2026-11-17 build --release` with `-Zbuild-std=core,alloc` and target
   `riscv32ima-unknown-none-elf`. rustflags: `-C relocation-model=pie -C code-model=medium -C target-feature=-c`.
   `crate-type = ["staticlib"]`, `panic = "abort"`.
2. `#[no_mangle] extern "C" fn main(argc, argv)` is the entry point. uClibc functions (`write`, `usleep`,
   `malloc`) are declared as `extern "C"`. A `GlobalAlloc` over `malloc`/`free` makes `alloc`
   (`Vec`, `String`, `format!`) work.
3. The Buildroot toolchain links it into a FLAT binary:

   ```sh
   riscv32-buildroot-linux-uclibc-gcc -mabi=ilp32 -march=rv32ima -fPIE -pie -static -Os -s \
       librust_nostd_hello.a "-Wl,-elf2flt=-r -s 16384" -Wl,-gc-sections -o rust_nostd_hello
   ```

The source is in [`linux/apps/rust-nostd`](../../linux/apps/rust-nostd), with a Makefile that runs the two steps
above. The image build includes it through the Buildroot external tree in `linux/br2-external`. To change
it, edit the crate and either rebuild the image ([building.md](building.md)), or build just the program with
the SDK and try it in QEMU ([linux-apps.md](linux-apps.md)).

## The FLAT trailing-word workaround

The kernel's FLAT loader writes a pointer into the 4 bytes just before `.data`. With `elf2flt`'s
linker script, that is the last word of `.text`. In the first Rust build, that word was an instruction
in `memcpy`, and the program died with `Illegal instruction`.

The crate puts a 4-byte placeholder in `.call_table_text`, which the linker script places last in
`.text`. Any FLAT program, C or Rust, can hit this. A proper fix in `elf2flt.ld` or the kernel is still
to be done.

## Full `std`

Full Rust `std` (threads, files, time) also works, with patches and caveats: see
[experimental/rust-std.md](experimental/rust-std.md).
