include spdk_config.mk

APP = hello_blob

all : $(APP)
	@:

define VARS
CPPFLAGS="""$(CPPFLAGS)"""
LDFLAGS="""$(LDFLAGS)"""
OBJS="""$(OBJS)"""
LIBS="""$(LIBS)"""
SYS_LIBS="""$(SYS_LIBS)"""
endef
export VARS

$(APP) : $(OBJS) $(SPDK_LIB_FILES) $(BLOCKDEV_MODULES_FILES) $(LINKER_MODULES) $(ENV_LIBS)
	@echo "$$VARS" > "$(ENV_PATH)"
