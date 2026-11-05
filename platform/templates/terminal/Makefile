# Builds the terminal host against the C bridge. The output is in target/terminal.
ROOT := ../..
OUT := $(ROOT)/target/terminal
CARGO ?= cargo
CFLAGS ?= -std=c99 -O2 -Wall -Wextra -Wpedantic -Werror
# What a static Rust library needs from the system, on systems that do not add it themselves.
ifeq ($(shell uname -s),Linux)
LDLIBS ?= -lgcc_s -lpthread -ldl -lm
endif

$(OUT)/terminal: terminal.c $(ROOT)/bridges/c/core.h library
	mkdir -p $(OUT)
	$(CC) $(CFLAGS) -I$(ROOT)/bridges/c terminal.c $(ROOT)/target/native/libbridge_c.a $(LDLIBS) -o $@

.PHONY: library
library:
	$(CARGO) build --locked --profile native -p bridge-c
