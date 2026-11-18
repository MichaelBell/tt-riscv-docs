# Board setup (read once)

Every guide in this repo uses the same setup.

## You need

- A Tiny Tapeout demo board with the chip you want to use. The guides are written for the current
  demo board (RP2350B, TT MicroPython SDK 3.x). The older RP2040 board (TT04-TT07, SDK 2.x) works with the same
  steps; `uart_console.py` is tested on both.
- A [QSPI Pmod](https://github.com/mole99/qspi-pmod) (16 MiB flash + 2 x 8 MiB PSRAM), from the
  [Tiny Tapeout store](https://store.tinytapeout.com/products/QSPI-Pmod-p716541602). Plug it into the
  **BIDIR (uio)** Pmod port.
- A USB cable and a Chromium-based browser (Chrome, Edge) for the web tools, which use Web Serial.

## Tools

| Tool                                                                       | Use                                                                                                                                   |
| -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| [Tiny Tapeout Flasher](https://tinytapeout.github.io/tinytapeout-flasher/) | Write files to the QSPI Pmod flash. Choose **Custom**, enter the offset **in decimal**, pick the file, **Flash**. One file at a time. |
| [TinyQV programmer](https://program.tinyqv.com)                            | Flash and run TinyQV programs, with a built-in console.                                                                               |
| [Tiny Tapeout Commander](https://commander.tinytapeout.com)                | The board's MicroPython REPL in the browser (REPL tab). Any serial terminal or `mpremote` works too.                                  |
| [`tools/uart_console.py`](../tools/uart_console.py)                        | Starts a KianV-style SoC and bridges its UART to the REPL.                                                                            |

Close the browser tab of one tool before connecting with another: only one program can hold the serial port.

## Install the UART console once

```sh
pip install mpremote
mpremote cp tools/uart_console.py :uart_console.py
```

No `mpremote`? Open the file, copy everything, and paste it into the REPL in paste mode
(Ctrl-E, paste, Ctrl-D). The functions then exist until the next reset.

At the REPL:

```python
import uart_console
uart_console.kianv()          # start KianV at 30 MHz and open its console
```

Press **Ctrl-]** to leave the console. Ctrl-C is passed to the SoC. MicroPython soft-resets when you
leave, which returns all pins to the SDK defaults.

## Flash offsets are decimal in the Flasher

| Hex      | Decimal |
| -------- | ------- |
| 0x000000 | 0       |
| 0x100000 | 1048576 |
| 0x180000 | 1572864 |
| 0x200000 | 2097152 |

One Pmod cannot hold the images of all SoCs at the same time: see [flash-layout.md](flash-layout.md).
