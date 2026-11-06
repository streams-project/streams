# Builds the terminal host against the C bridge. The output is in $(TARGET_DIR)/terminal.
# TARGET_DIR is the folder cargo writes into: CARGO_TARGET_DIR when the build sets one, else the
# repository's own target folder. A caller with its own folder passes TARGET_DIR=<folder>. The
# library is linked from that same folder, and cargo is told to write there, so the two cannot
# disagree and no fixed target path is assumed.
ROOT := ../../..
PLATFORM := ../..
TARGET_DIR ?= $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),$(ROOT)/target)
OUT := $(TARGET_DIR)/terminal
LIBRARY := $(TARGET_DIR)/native/libbridge_c.a
CARGO ?= cargo
CFLAGS ?= -std=c99 -O2 -Wall -Wextra -Wpedantic -Werror
# What a static Rust library needs from the system, on systems that do not add it themselves.
ifeq ($(shell uname -s),Linux)
LDLIBS ?= -lgcc_s -lpthread -ldl -lm
endif

$(OUT)/terminal: terminal.c $(PLATFORM)/bridges/c/core.h library
	mkdir -p '$(OUT)'
	$(CC) $(CFLAGS) -I$(PLATFORM)/bridges/c terminal.c '$(LIBRARY)' $(LDLIBS) -o '$@'

.PHONY: library
library:
	CARGO_TARGET_DIR='$(TARGET_DIR)' $(CARGO) build --locked --profile native -p bridge-c
