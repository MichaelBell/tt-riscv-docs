# SPDX-License-Identifier: Apache-2.0
# rust_nostd_hello: no_std Rust built as a static library, linked into a FLAT binary by the uClibc gcc.
# Needs rustup; the nightly in rust-toolchain.toml (or $RUSTUP_TOOLCHAIN) with rust-src.
include $(or $(KIANV_COMMON_MK),../common.mk)
STACK := 16384
LIB := target/riscv32ima-unknown-none-elf/release/librust_nostd_hello.a

all: rust_nostd_hello

# Buildroot exports CC/CFLAGS/... for the target: keep them away from cargo.
$(LIB): FORCE
	env -u CC -u CXX -u CFLAGS -u CXXFLAGS -u LDFLAGS -u AR cargo build --release

rust_nostd_hello: $(LIB)
	$(CC) $(KIANV_CFLAGS) $< $(KIANV_LDFLAGS) -o $@
	rm -f $@.gdb

install: rust_nostd_hello
	install -D -m 0755 rust_nostd_hello $(DESTDIR)/root/rust_nostd_hello

clean:
	rm -rf target rust_nostd_hello rust_nostd_hello.gdb

.PHONY: all install clean FORCE
