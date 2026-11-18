# Check a KianV chip and Pmod with the memtest

[`kianv-memtest-16m-gpio`](https://github.com/urish/kianv-memtest-16m-gpio) is a power-on self-test
that takes the place of the Linux bootloader. It exercises all 16 MiB of PSRAM, reports over the
UART, and latches **P** (pass) or **F** (fail) on the 7-segment display. Use it to check a new board,
a new Pmod, or a chip that won't boot Linux.

Tested on ttsky25a (30 MHz). On ttgf0p2, the project owner ran it over all 16 MiB (project feedback). It applies to
all 16 MiB KianV variants: ttsky25a, ttsky25b, ttihp25b, ttcad25a, ttgf0p2/0p3.

## Flash

From the [latest release](../../../releases/latest), flash `kianv-memtest.bin` at offset **1048576**
(0x100000) with the [Flasher](https://tinytapeout.github.io/tinytapeout-flasher/) (**Custom**).
This replaces the Linux bootloader; the Linux Image stays in flash.

## Run

```python
import uart_console
uart_console.kianv_memtest()                 # fast: about 7 s
uart_console.kianv_memtest(thorough=True)    # 3 full sweeps of every word: about 8.5 minutes at 30 MHz
```

Fast mode on ttsky25a:

```
KianV uLinux SoC v2: 16 MiB QSPI PSRAM memtest
  region = 0x80000000 .. 0x80fff000 (16 MiB - 4 KiB top reserved for stack)
  mode   = fast (ui_in[1]=0)
--- probe phase (write+read 2 patterns at 8 addresses) ---
  0x80000000 wr 0xdeadbeef rd 0xdeadbeef OK
  ...
  0x80ffe000 wr 0x12345678 rd 0x12345678 OK
probe phase clean. starting bulk sweep
[fast] stride-64 address-in-data  write. verify. OK

PASS - all PSRAM tests succeeded
```

Thorough mode prints `[1/3]`, `[2/3]`, `[3/3]` with a dot per MiB, then `PASS`.

Notes:

- The mode is read from `ui_in[1]` once, at reset.
- `ui_in[0]` = 1 switches progress to a spinner on the 7-segment display (the UART then goes quiet).
- One `[ui_in[0]=0] -> back to UART dots` line at the start is normal.
- If it hangs right after the banner, the PSRAMs are in the wrong mode. `uart_console` resets them.
  Otherwise, power-cycle.
- A failure prints `FAIL test=N addr=... exp=... got=...`; the last line before a hang names the access
  that hung.

## Back to Linux

Flash `kianv-ulinux-bootloader.bin` at 1048576 again ([kianv-linux.md](kianv-linux.md)).
