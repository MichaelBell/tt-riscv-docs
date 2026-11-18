# SPDX-License-Identifier: Apache-2.0
# Shared settings for the KianV uLinux programs (RV32IMA without C, no MMU, uClibc, FLAT binaries).
#
# CROSS_COMPILE selects the toolchain:
#   - from the Buildroot build: set by the tt-* packages in ../br2-external
#   - from the SDK release:     CROSS_COMPILE=<sdk>/bin/riscv32-buildroot-linux-uclibc-
#   - default:                  that prefix from $PATH (e.g. after sourcing the SDK's environment-setup)
CROSS_COMPILE ?= riscv32-buildroot-linux-uclibc-
CC := $(CROSS_COMPILE)gcc
FLTHDR := $(CROSS_COMPILE)flthdr

KIANV_CFLAGS := -mabi=ilp32 -march=rv32ima -fPIE -pie -static -Os -s \
                -ffunction-sections -fdata-sections -Wall -Wextra
# elf2flt makes the FLAT binary; -s sets its stack (the elf2flt default is 4 KiB).
KIANV_LDFLAGS = '-Wl,-elf2flt=-r$(if $(STACK), -s $(STACK))' -Wl,-gc-sections

# Programs are installed to $(DESTDIR)/root.
DESTDIR ?= $(CURDIR)/out
