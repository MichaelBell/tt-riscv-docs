################################################################################
#
# tt-rust-std: builds linux/apps/rust-std with its own Makefile
#
################################################################################

TT_RUST_STD_SITE = $(BR2_EXTERNAL_TT_RISCV_PATH)/../apps/rust-std
TT_RUST_STD_SITE_METHOD = local
TT_RUST_STD_LICENSE = Apache-2.0

define TT_RUST_STD_BUILD_CMDS
	if command -v cargo >/dev/null 2>&1; then $(TARGET_MAKE_ENV) RUST_STD_WORK=$(BUILD_DIR)/tt-rust-std-work $(MAKE) -C $(@D) CROSS_COMPILE="$(TARGET_CROSS)" KIANV_COMMON_MK=$(BR2_EXTERNAL_TT_RISCV_PATH)/../apps/common.mk; else echo "warning: cargo not found, skipping rust-std" >&2; fi
endef

define TT_RUST_STD_INSTALL_TARGET_CMDS
	if [ -d $(@D)/target -o -d $(@D)/out ]; then $(TARGET_MAKE_ENV) RUST_STD_WORK=$(BUILD_DIR)/tt-rust-std-work $(MAKE) -C $(@D) CROSS_COMPILE="$(TARGET_CROSS)" KIANV_COMMON_MK=$(BR2_EXTERNAL_TT_RISCV_PATH)/../apps/common.mk DESTDIR="$(TARGET_DIR)" install; fi
endef

$(eval $(generic-package))
