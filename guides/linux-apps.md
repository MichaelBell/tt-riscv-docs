# Write your own Linux program for KianV

Build a program for the [KianV Linux image](kianv-linux.md) in seconds, try it in QEMU, then put it on the chip.

You need a Linux x86-64 machine (or WSL) with `make` and Python 3. Rust programs also need
[rustup](https://rustup.rs).

## 1. Get the SDK

From the [latest release](../../../releases/latest), download `kianv-ulinux-sdk.tar.gz` (the toolchain
the image itself was built with: uClibc gcc, `elf2flt`, QEMU) and `kianv-ulinux-Image`, then:

```sh
tar xzf kianv-ulinux-sdk.tar.gz
cd riscv32-buildroot-linux-uclibc_sdk-buildroot && ./relocate-sdk.sh && cd ..
export SDK=$PWD/riscv32-buildroot-linux-uclibc_sdk-buildroot
```

## 2. Build

Start from one of the programs in [`linux/apps`](../linux/apps):

```sh
cp -r linux/apps/hello-7seg my-app && cp linux/apps/common.mk .
cd my-app
make CROSS_COMPILE=$SDK/bin/riscv32-buildroot-linux-uclibc-
```

The result is a FLAT binary, the executable format of Linux without an MMU. Every program needs the
flags in [`common.mk`](../linux/apps/common.mk): RV32IMA without compressed instructions, position-independent,
static, linked through `elf2flt`.

Rust: copy `linux/apps/rust-nostd` instead. Its Makefile builds the crate with `cargo` and links it with the
SDK's gcc in the same way ([details](rust-kianv-linux.md)).

## 3. Try it in QEMU

```sh
python3 linux/qemu/run.py --sdk $SDK --image kianv-ulinux-Image --add my-app/hello_7seg
```

Linux boots in about 30 seconds. Your program is in `/root`. Quit with Ctrl-A, then X.

QEMU has no KianV GPIO block: programs that drive the 7-segment display fault there, so test that part on
the chip.

## 4. Run it on the chip

The programs in `/root` are part of the Image, so the program has to be built into the image:

1. Add it as a package, as described in [`linux/README.md`](../linux/README.md#add-a-program-to-the-image).
2. Build the image with `scripts/build-ulinux.sh` ([building.md](building.md); the first build takes 30-60 minutes,
   later ones a few minutes).
3. Flash `bootloader.bin` and `Image` ([kianv-linux.md](kianv-linux.md)). The two always go together.

## Tips

- `/root` is in RAM: files you create there are gone after a reset.
- The default stack of a FLAT program is 4 KiB. Set `STACK := 16384` (or more) in the Makefile if the
  program crashes in deep calls or with large local variables.
- `printf` works. To drive the pins, write to the GPIO registers directly: `GPIO_UO_EN` at 0x10600000,
  `GPIO_UO_OUT` at 0x10600004 and `GPIO_UI_IN` at 0x10600008 (see `hello_7seg.c`).
