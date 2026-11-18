# Zephyr and MicroPython on the KianV SoC

Hirosh Dabui's [Zephyr port](https://github.com/splinedrive/kianRiscV/tree/master/asic/os/zephyr_kianv_asic)
puts a boot menu, Zephyr and a bare-metal MicroPython into one flash image. This repo builds it with one fix
(patch 0006): the system timer runs at the real 10 kHz `mtime` rate, so `uptime` and sleeps are correct.

Tested on ttsky25a and TT06 (30 MHz; TT06 with its RP2040 demo board). It uses 8 MiB of PSRAM, so it runs on both
the original 8 MiB KianV and the 16 MiB variants.

## Flash

From the [latest release](../../../releases/latest), flash `kianv-zephyr-dualboot.bin` at offset
**1048576** (0x100000) with the [Flasher](https://tinytapeout.github.io/tinytapeout-flasher/) (**Custom**).
It covers 0x100000 - 0x3B1000 and replaces Linux. See [flash-layout.md](flash-layout.md).

## Boot

```python
import uart_console
uart_console.kianv()
```

```
KianV RISC-V uLinux ASIC Tiny Tapeout SOC
bootloader
z: Zephyr  m: MicroPython
```

Press **z** or **m**.

### Zephyr (z)

Zephyr boots in about 3 s to a shell:

```
*** Booting Zephyr OS build kianv-local ***
Zephyr RTOS on KianV RV32IMA ASIC SoC
Shell ready
kianv:~$ kernel version
Zephyr version 4.5.99
kianv:~$ device list
devices:
- uart@10000000 (READY)
- ramdisk0 (READY)
kianv:~$ fs write /RAM/test.txt 68 65 6c 6c 6f 0a
kianv:~$ fs read /RAM/test.txt
00000000  68 65 6C 6C 6F 0A                                hello.
kianv:~$ devmem 0x10600008
Read value 0x8
```

`devmem 0x10600008` reads the GPIO `ui_in` register. Other useful commands: `kernel uptime`,
`kernel sleep 1000`, `kernel thread list`, `help`.

### MicroPython (m)

```
MicroPython 9bf76fcf9-dirty on 2026-11-17; RV32IMA uLinux SoC TT05 with KianV
>>> print(355/113, [x*x for x in range(10)])
3.141593 [0, 1, 4, 9, 16, 25, 36, 49, 64, 81]
```

This is a minimal port (MicroPython 1.19.1, 6 MB heap):

- No big integers, and no `machine`, `time` or `help`.
- `gc.mem_free()` takes about a minute, because it scans the whole heap.
- Wait for the `>>>` prompt before typing: characters sent while it is busy are lost.

## Credits

Zephyr and MicroPython ports: Hirosh Dabui ([splinedrive/kianRiscV](https://github.com/splinedrive/kianRiscV),
[splinedrive/micropython](https://github.com/splinedrive/micropython)).
