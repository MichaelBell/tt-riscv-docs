# Bare-metal Rust on TinyQV

A `no_std` Rust program for TinyQV, with its own startup code and drivers for the UART, GPIO, timer and
the `mul16` custom instruction. Source: [`examples/rust/tinyqv-hello`](../examples/rust/tinyqv-hello).

Tested on ttsky25a: `tt_um_tt_tinyQV` (#495) and `tt_um_tt_tinyQVb` (#687). The same image runs on both.

## Just run it

1. Download `tinyqv-rust-hello.bin` from the [latest release](../../../releases/latest).
2. In the [TinyQV programmer](https://program.tinyqv.com) choose **Custom**, pick the file, design **495**
   (or **687**), latency **2**, **64** MHz, then **FLASH AND RUN**.

```
Hello from Rust on TinyQV! (instance 'A', uart div 555)
.data check: COUNTER=42 (expect 42)
delay_us(10000): 80010 cycles, mtime +10001 us -> 8 MHz
sieve: 1229 primes below 10000, largest 9973, 73281 us
sum i*i, i<1000: soft __mulsi3 332833500 (195633 cycles), mul16 332833500 (52513 cycles)
no-fmt u32: 4294967295 hex: 0x00c0ffee
func_sel out0..7 = 2 2 1 1 1 1 1 1, OUT=0x00, IN=0x80
toggle(4) -> OUT=0x1e
tqv>
```

At the `tqv>` prompt, try `in`, `out 0x1e`, `primes 1000`, `mul 123456 789`, `time`.

The image is about 10 KB, starts at flash offset 0 and stays below 1 MiB, so a KianV Linux image in the
same Pmod is not touched.

## Build it

```sh
rustup toolchain install nightly-2026-11-17 --profile minimal -c rust-src -c llvm-tools
cd examples/rust/tinyqv-hello
./build.sh        # -> tinyqv-hello.bin, plus an instruction-set check
```

Key points (details in the example's README):

- **Target.** TinyQV has no hardware multiply, so the stock `riscv32emc-unknown-none-elf` target
  (which emits `mul`/`div`) does not fit. The example uses a custom target,
  `riscv32ec-tinyqv-none-elf.json`, with `-Zbuild-std=core`.
- **Memory.** Code runs from flash at 0. `.data`, `.bss` and the stack live in PSRAM. The linker
  script mirrors Mike Bell's [tinyQV-sdk](https://github.com/MichaelBell/tinyQV-sdk) (branch `ttsky25a`).
- **Instruction check.** `build.sh` checks every instruction against what TinyQV implements.

## Notes

- `rdcycle` counts at clock/8 (8 MHz at 64 MHz): the 4-bit serial core takes 8 clocks per instruction.
  `mtime` counts microseconds.
- Interrupts are not used yet.
- Next steps (help welcome): `tinyqv-rt` / `tinyqv-pac` / `tinyqv-hal` crates with `embedded-hal` 1.0,
  and a faster `__mulsi3` using `mul16`.
