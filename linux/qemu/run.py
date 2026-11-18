#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Boot a KianV uLinux Image in QEMU, optionally with extra programs in /root.

  run.py --image Image                          interactive shell (quit with Ctrl-A X)
  run.py --image Image --add hello hello2       same, with ./hello and ./hello2 copied to /root
  run.py --image Image --cmd "ls -l /root"      boot, run the commands, print their output, exit
  run.py --image Image --smoke                  boot, run the checks below, exit 0 on success

--add packs the files into a small initramfs (cpio) that QEMU passes next to the Image, so a
program built with the SDK can be tried without rebuilding the image. QEMU's "virt" machine has no
KianV GPIO block: programs that drive the 7-segment display (hello_7seg, rust_nostd_hello) fault
there and only work on the chip.

QEMU is taken from $QEMU, then the SDK or Buildroot host tools (--sdk, or found next to this repo),
then $PATH.
"""

import argparse
import os
import select
import shutil
import subprocess
import sys
import tempfile
import time

SMOKE = [  # (command, text that must appear before the next prompt)
    ("uname -om", "riscv32 GNU/Linux"),
    ("cat /etc/os-release", "NAME=Buildroot"),
    ("ls /root", "rust_std_demo"),
    ("/root/rust_std_hello", "Hello from Rust std on KianV uLinux!"),
    ("/root/rust_std_demo", "rust_std_demo done"),
]


def cpio_newc(entries):
    """entries: list of (path in archive, mode, data or None for a directory) -> bytes."""
    out = bytearray()

    def add(name, mode, data):
        name_b = name.encode() + b"\0"
        fields = [0, mode, 0, 0, 1, 0, len(data), 0, 0, 0, 0, len(name_b), 0]
        out.extend(b"070701" + b"".join(b"%08X" % f for f in fields) + name_b)
        out.extend(b"\0" * (-len(out) % 4))
        out.extend(data)
        out.extend(b"\0" * (-len(out) % 4))

    for name, mode, data in entries:
        add(name, mode, data if data is not None else b"")
    add("TRAILER!!!", 0, b"")
    return bytes(out)


def find_qemu(sdk):
    candidates = [os.environ.get("QEMU")]
    if sdk:
        candidates.append(os.path.join(sdk, "bin", "qemu-system-riscv32"))
    root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    candidates.append(os.path.join(root, "build", "kianRiscV", "asic", "os", "ulinux_asic_kianv_soc",
                                   "buildroot", "output", "host", "bin", "qemu-system-riscv32"))
    candidates.append(shutil.which("qemu-system-riscv32"))
    for c in candidates:
        if c and os.path.isfile(c) and os.access(c, os.X_OK):
            return c
    sys.exit("qemu-system-riscv32 not found: install QEMU (Debian/Ubuntu: qemu-system-riscv or "
             "qemu-system-misc), pass --sdk, or set $QEMU")


def run_script(cmd, steps, timeout):
    """steps: list of (command, expected text or None). Returns 0 when every expectation is met."""
    p = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    buf = b""

    def wait_for(marker, t):
        nonlocal buf
        end = time.time() + t
        while time.time() < end:
            r, _, _ = select.select([p.stdout], [], [], 0.5)
            if r:
                d = os.read(p.stdout.fileno(), 4096)
                if not d:
                    break
                sys.stdout.write(d.decode(errors="replace"))
                sys.stdout.flush()
                buf += d
                if marker in buf:
                    i = buf.index(marker) + len(marker)
                    seen, buf = buf[:i], buf[i:]
                    return seen
        return None

    ok = wait_for(b"/ # ", timeout) is not None
    failures = [] if ok else ["no shell prompt"]
    for command, expected in steps if ok else []:
        p.stdin.write(command.encode() + b"\n")
        p.stdin.flush()
        seen = wait_for(b"/ # ", timeout)
        if seen is None:
            failures.append(f"{command!r}: no prompt within {timeout:.0f} s")
        elif expected is not None and expected.encode() not in seen:
            failures.append(f"{command!r}: expected {expected!r}")
    p.kill()
    print()
    for f in failures:
        print(f"FAIL: {f}")
    print("PASS" if not failures else "FAILED")
    return 0 if not failures else 1


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--image", required=True, help="KianV uLinux Image (e.g. kianv-ulinux-Image)")
    ap.add_argument("--add", nargs="*", default=[], help="programs to copy to /root")
    ap.add_argument("--sdk", help="extracted SDK directory (to use its QEMU)")
    ap.add_argument("--cmd", action="append", default=[], help="command to run (repeatable); then exit")
    ap.add_argument("--smoke", action="store_true", help="run the smoke test and exit")
    ap.add_argument("--timeout", type=float, default=180, help="seconds per step in --smoke")
    args = ap.parse_args()

    cmd = [find_qemu(args.sdk), "-M", "virt", "-bios", "none", "-kernel", args.image, "-nographic",
           "-cpu", "rv32,mmu=off", "-m", "16M", "-append", "rootwait root=/dev/vda ro"]
    with tempfile.TemporaryDirectory() as tmp:
        if args.add:
            entries = [("root", 0o040755, None)]
            for f in args.add:
                with open(f, "rb") as fh:
                    entries.append(("root/" + os.path.basename(f), 0o100755, fh.read()))
            initrd = os.path.join(tmp, "extra.cpio")
            with open(initrd, "wb") as fh:
                fh.write(cpio_newc(entries))
            cmd += ["-initrd", initrd]
        if args.smoke:
            return run_script(cmd, SMOKE, args.timeout)
        if args.cmd:
            return run_script(cmd, [(c, None) for c in args.cmd], args.timeout)
        print("Booting in QEMU (about 30 s). Quit with Ctrl-A then X.", file=sys.stderr)
        return subprocess.call(cmd)


if __name__ == "__main__":
    sys.exit(main())
