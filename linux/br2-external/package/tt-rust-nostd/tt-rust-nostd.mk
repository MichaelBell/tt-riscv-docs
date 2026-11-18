################################################################################
#
# tt-rust-nostd: builds linux/apps/rust-nostd with its own Makefile
#
################################################################################

TT_RUST_NOSTD_SITE = $(BR2_EXTERNAL_TT_RISCV_PATH)/../apps/rust-nostd
TT_RUST_NOSTD_SITE_METHOD = local
TT_RUST_NOSTD_LICENSE = Apache-2.0

define TT_RUST_NOSTD_BUILD_CMDS
	if command -v cargo >/dev/null 2>&1; then $(TARGET_MAKE_ENV) $(MAKE) -C $(@D) CROSS_COMPILE="$(TARGET_CROSS)" KIANV_COMMON_MK=$(BR2_EXTERNAL_TT_RISCV_PATH)/../apps/common.mk; else echo "warning: cargo not found, skipping rust-nostd" >&2; fi
endef

define TT_RUST_NOSTD_INSTALL_TARGET_CMDS
	if [ -d $(@D)/target -o -d $(@D)/out ]; then $(TARGET_MAKE_ENV) $(MAKE) -C $(@D) CROSS_COMPILE="$(TARGET_CROSS)" KIANV_COMMON_MK=$(BR2_EXTERNAL_TT_RISCV_PATH)/../apps/common.mk DESTDIR="$(TARGET_DIR)" install; fi
endef

$(eval $(generic-package))
