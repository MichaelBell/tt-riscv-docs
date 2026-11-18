# SPDX-License-Identifier: Apache-2.0
# hello_7seg: HELLO then 0-9 on the demo board's 7-segment display, through the SoC's GPIO registers.
include $(or $(KIANV_COMMON_MK),../common.mk)

all: hello_7seg

hello_7seg: hello_7seg.c
	$(CC) $(KIANV_CFLAGS) $< $(KIANV_LDFLAGS) -o $@

install: hello_7seg
	install -D -m 0755 hello_7seg $(DESTDIR)/root/hello_7seg

clean:
	rm -f hello_7seg hello_7seg.gdb

.PHONY: all install clean
