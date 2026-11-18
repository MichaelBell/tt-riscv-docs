# SPDX-License-Identifier: Apache-2.0
# rust_std_hello, rust_std_demo: Rust with the full std (experimental; see build.sh).
# Needs rustup with the nightly in $RUST_STD_TOOLCHAIN (default: build.sh) and rust-src, plus curl.
include $(or $(KIANV_COMMON_MK),../common.mk)
# build.sh wants the toolchain's bin/ directory: derive it from CROSS_COMPILE.
TOOLCHAIN_BIN := $(patsubst %/,%,$(dir $(shell command -v $(CC) 2>/dev/null || echo $(CC))))

all:
	env -u CC -u CXX -u CFLAGS -u CXXFLAGS -u LDFLAGS -u AR ./build.sh $(TOOLCHAIN_BIN)/.. out

install: all
	install -D -m 0755 out/rust_std_hello $(DESTDIR)/root/rust_std_hello
	install -D -m 0755 out/rust_std_demo $(DESTDIR)/root/rust_std_demo

clean:
	rm -rf out .work

.PHONY: all install clean
