# Programs for the KianV Linux image

Plain source for the programs in `/root` of the [KianV Linux image](../guides/kianv-linux.md), and the glue
that builds them into it.

| Path                                 | What                                                                                                                     |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------ |
| [`apps/hello-7seg`](apps/hello-7seg) | C: HELLO and 0-9 on the 7-segment display, through the SoC's GPIO registers                                              |
| [`apps/rust-nostd`](apps/rust-nostd) | Rust (`no_std`): primes, `alloc`, GPIO ([guide](../guides/rust-kianv-linux.md))                                          |
| [`apps/rust-std`](apps/rust-std)     | Rust with the full `std`, experimental ([guide](../guides/experimental/rust-std.md))                                     |
| [`apps/common.mk`](apps/common.mk)   | Compiler and linker flags every program needs (RV32IMA without C, no MMU, FLAT binaries)                                 |
| [`br2-external`](br2-external)       | [Buildroot external tree](https://buildroot.org/downloads/manual/manual.html#outside-br-custom): one package per program |
| [`qemu/run.py`](qemu/run.py)         | Boot an Image in QEMU, optionally with extra programs in `/root`                                                         |

Each program has its own `Makefile`, which works in two places:

- **In the image build.** `scripts/build-ulinux.sh` passes `BR2_EXTERNAL=linux/br2-external` to Buildroot. Each
  package there runs the program's Makefile with Buildroot's toolchain and installs the result in `/root`.
- **On its own, with the SDK.** Download the SDK from a release, then run `make` in the program's directory.
  See [Write your own Linux program](../guides/linux-apps.md).

## Add a program to the image

1. Copy `apps/hello-7seg` to `apps/<name>`, and change the source and the Makefile (keep the `include` line
   and the `install` target).
2. Copy `br2-external/package/tt-hello-7seg` to `br2-external/package/tt-<name>`, rename the `.mk` file, and
   replace `TT_HELLO_7SEG` / `hello-7seg` inside both files.
3. Add a `source` line for it to `br2-external/Config.in`.
4. Run `scripts/build-ulinux.sh`. The program is in `/root/<name>` of the new Image.
