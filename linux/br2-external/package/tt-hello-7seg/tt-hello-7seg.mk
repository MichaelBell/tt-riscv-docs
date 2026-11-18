################################################################################
#
# tt-hello-7seg: builds linux/apps/hello-7seg with its own Makefile
#
################################################################################

TT_HELLO_7SEG_SITE = $(BR2_EXTERNAL_TT_RISCV_PATH)/../apps/hello-7seg
TT_HELLO_7SEG_SITE_METHOD = local
TT_HELLO_7SEG_LICENSE = Apache-2.0

define TT_HELLO_7SEG_BUILD_CMDS
	$(TARGET_MAKE_ENV) $(MAKE) -C $(@D) CROSS_COMPILE="$(TARGET_CROSS)" KIANV_COMMON_MK=$(BR2_EXTERNAL_TT_RISCV_PATH)/../apps/common.mk
endef

define TT_HELLO_7SEG_INSTALL_TARGET_CMDS
	$(TARGET_MAKE_ENV) $(MAKE) -C $(@D) CROSS_COMPILE="$(TARGET_CROSS)" KIANV_COMMON_MK=$(BR2_EXTERNAL_TT_RISCV_PATH)/../apps/common.mk DESTDIR="$(TARGET_DIR)" install
endef

$(eval $(generic-package))
