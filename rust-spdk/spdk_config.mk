ifneq "$(wildcard $(SPDK_ROOT_DIR) )" ""
ifneq "$(wildcard $(DPDK_DIR) )" ""

    SPDK_LIB_LIST = event_bdev event_copy
    SPDK_LIB_LIST += blobfs blob bdev blob_bdev copy event util conf trace \
		log jsonrpc json rpc

    include $(SPDK_ROOT_DIR)/mk/spdk.common.mk
    include $(SPDK_ROOT_DIR)/mk/spdk.app.mk
    include $(SPDK_ROOT_DIR)/mk/spdk.modules.mk

    LIBS += $(COPY_MODULES_LINKER_ARGS) $(BLOCKDEV_MODULES_LINKER_ARGS)
    LIBS += $(SPDK_LIB_LINKER_ARGS) $(ENV_LINKER_ARGS) 
endif
endif