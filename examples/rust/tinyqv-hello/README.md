# Rust on TinyQV (ttsky25a)

Bare-metal `no_std` Rust for the TinyQV RISC-V SoC on Tiny Tapeout ttsky25a:
`tt_um_tt_tinyQV` (#495, "Asteroids", instance ID `A`) and `tt_um_tt_tinyQVb` (#687, "Berzerk", ID `B`).
The same image runs unchanged on both (tested on silicon).

## Hardware model

- CPU: RV32EC + Zcb + Zicond + Zicsr, plus TinyQV custom instructions (`mul16`, `lw4`/`sw4`, ...). No M extension.
- Code runs XIP from the QSPI flash at address 0 (only from flash). Data and stack live in PSRAM A
  (`0x1000000`-`0x17FFFFF`). `gp` (0x1000400) and `tp` (0x8000000) are hardwired.
- Fixed vectors: reset `0x0`, exception `0x4`, interrupt `0x8`.
- Register map: `ttsky25a-tinyQV/docs/info.md`. The C SDK (`MichaelBell/tinyQV-sdk`, branch `ttsky25a`) is the reference.

## Prerequisites

- rustup with a nightly toolchain and `rust-src` + `llvm-tools` (`rust-toolchain.toml` selects them).
- Optional: Mike Bell's GCC toolchain (`MichaelBell/riscv-gnu-toolchain`, release `15.1.0-tqv-2.0`) for an
  objdump that names TinyQV custom instructions. Release `16.1.0-tqv-2.1` needs glibc >= 2.38.

## Build

```sh
./build.sh
```

This runs `cargo build --release` and writes `tinyqv-hello.bin` (flash image for offset 0), checks the
image is below 1 MiB, and checks every instruction against the TinyQV ISA (`isa-histogram.txt`).

How it is set up:

| File                             | Role                                                                                                                                                |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `riscv32ec-tinyqv-none-elf.json` | Custom target: `riscv32emc-unknown-none-elf` minus `+m`, plus `+zcb,+zicond,+zicsr`, ABI `ilp32e`                                                   |
| `.cargo/config.toml`             | Selects the target, `-Zbuild-std=core` with `compiler-builtins-mem`, `json-target-spec`, links with `-Ttinyqv.ld`                                   |
| `tinyqv.ld`                      | FLASH `0x0` (1 MiB cap), RAM `0x1000400`-`0x17FFFFF` (first 1 KiB reserved for interrupt context), `.data` loaded from flash, stack top `0x1800000` |
| `src/rt.rs`                      | `global_asm!` startup: vectors, set gp/tp/sp, mask interrupts, zero `.bss`, copy `.data`, call `main`; trap and panic handlers print over UART      |
| `src/uart.rs`                    | Polled UART (TX, RX, `core::fmt::Write`, `print!`/`println!`, no-fmt `write_u32`/`write_hex`)                                                       |
| `src/gpio.rs`                    | OUT/IN, function select, debug select                                                                                                               |
| `src/timer.rs`                   | `mtime` (1 MHz), `rdcycle`/`rdtime`/`rdinstret`, `delay_us`                                                                                         |
| `src/mul.rs`                     | `mul16` custom instruction via `.insn r 0x33, 0, 2` and a 2-instruction 32x32 multiply                                                              |
| `src/main.rs`                    | Demo: banner, sieve, multiply benchmark, GPIO, then a `tqv>` command loop on the UART                                                               |

Plain `cargo build` with the stock `riscv32emc-unknown-none-elf` target would emit `mul`/`div`, which TinyQV
does not have: use the custom target.

## Flash and run (TT demo board + QSPI Pmod)

The flash image goes at offset 0 and stays below 1 MiB, so a KianV image higher up in the Pmod is not touched.

In the [TinyQV programmer](https://program.tinyqv.com): choose **Custom**, pick `tinyqv-hello.bin`, design
**495** (or **687**), QSPI latency **2**, **64** MHz, then **FLASH AND RUN**. The UART is 115200 baud on
`ui_in[7]` (RX) / `uo_out[0]` (TX), and the programmer shows it in the page.

## Output on silicon (#495)

```
Hello from Rust on TinyQV! (instance 'A', uart div 555)
.data check: COUNTER=42 (expect 42)
delay_us(10000): 80010 cycles, mtime +10001 us -> 8 MHz
sieve: 1229 primes below 10000, largest 9973, 73281 us
sum i*i, i<1000: soft __mulsi3 332833500 (195633 cycles), mul16 332833500 (52513 cycles)
func_sel out0..7 = 2 2 1 1 1 1 1 1, OUT=0x00, IN=0x80
tqv> mul 123456 789
123456 * 789 = 97406784 (mul16: 97406784)
```

## Notes

- `rdcycle` counts at clock/8 (8 MHz at 64 MHz): the 4-bit serial core takes at least 8 clocks per instruction.
  `mtime` and `rdtime` count microseconds.
- At reset `out1` is function 2 (UART, the SDK moves CTS to `out5`). The demo sets `out1`-`out4` to GPIO and
  never touches `out0` (UART TX).
- With the programmer's run script, `in0`-`in6` are pulled low and `in7` (UART RX) idles high, so `IN` reads `0x80`.
- Interrupts stay masked. Using them needs the SDK's context-save scheme (`gp-0x200`, custom
  instructions in `macros.s`) and vectors at `0xc0`.
- UART RX is polled with a 1-byte hardware buffer: fine for typed input, not for bulk transfers.

## Next steps

- Split into crates: `tinyqv-rt` (target JSON, linker script, startup, interrupts), `tinyqv-pac`
  (registers for the core block and each contributed peripheral), `tinyqv-hal` with `embedded-hal` 1.0
  (`OutputPin`/`InputPin`, `DelayNs` on `mtime`) and `embedded-io` (`Read`/`Write` on the UART).
- Interrupt support (timer via `MTIMECMP`, UART RX) in `tinyqv-rt`; buffered UART RX.
- Provide `__mulsi3` with `mul16` (about 3.7x faster here) and memcpy using `lw4`/`sw4`.
- Drivers for contributed peripherals (PWM #20/#21, SPI #30, VGA, ...).
