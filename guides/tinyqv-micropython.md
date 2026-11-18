# MicroPython on TinyQV

[TinyQV](https://github.com/MichaelBell/tinyQV) by Mike Bell is a small RV32EC SoC that runs code
directly from the QSPI flash. The crowdsourced ttsky25a versions add peripherals contributed by the
community, and run MicroPython.

| Shuttle  | Project                                            | Status                                               |
| -------- | -------------------------------------------------- | ---------------------------------------------------- |
| ttsky25a | `tt_um_tt_tinyQV` "Asteroids" (#495)               | tested on silicon                                    |
| ttsky25a | `tt_um_tt_tinyQVb` "Berzerk" (#687)                | tested on silicon                                    |
| ttihp25b | `tt_um_tt_tinyQV` (#490) and other TinyQV shuttles | see [program.tinyqv.com](https://program.tinyqv.com) |

## Flash and run

1. Open the [TinyQV programmer](https://program.tinyqv.com) and connect to the board.
2. Select **MicroPython**, design **495** (or **687**), QSPI latency **2**, frequency **64** MHz.
3. Click **FLASH AND RUN**. The REPL appears in the page.

The same image is in the [latest release](../../../releases/latest) as `tinyqv-micropython.bin`, a mirror
of Mike's build with its sha256 checked.

## Try it

```python
>>> import sys; sys.implementation
(name='micropython', version=(1, 25, 0, ''), _machine='ttsky25a with tinyQV')
>>> 2**100
1267650600228229401496703205376
>>> from machine import Pin
>>> Pin(9).value(), Pin(15).value()      # in1, in7 (UART RX, idles high)
(0, 1)
>>> import tinyqv; tinyqv.write_byte_reg(21, 1, 100)   # peripheral registers
```

Both chips run the same image. On ttsky25a, `sum(i*i for i in range(10000))` takes about 5.6 s.

More examples, peripherals and `mpremote` over a USB bridge:
[ttsky25a-tinyQV/micropython](https://github.com/TinyTapeout/ttsky25a-tinyQV/tree/main/micropython).

## Notes

- The MicroPython image is 1.27 MB and overwrites the KianV bootloader at 0x100000. See [flash-layout.md](flash-layout.md).
- TinyQV leaves the PSRAMs in QPI mode. Before running KianV, use `uart_console.kianv()` (it resets
  them) or power-cycle.

## Credits

TinyQV, its SDK, MicroPython port and programmer: Mike Bell ([MichaelBell](https://github.com/MichaelBell)).
Peripherals: the TinyQV peripheral challenge contributors.
