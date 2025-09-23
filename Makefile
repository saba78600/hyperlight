# Minimal Makefile wrapper for common developer tasks
# Usage:
#   make build         -> builds with required LLVM env
#   make run ARGS="..." -> runs 'cargo run -- $(ARGS)'
#   make clean         -> cargo clean

LLVM_LIB ?= /usr/lib/llvm-18/lib
export LLVM_SYS_USE_SHARED=1
export LLVM_SYS_LINK_SHARED=1

.PHONY: build build-release run clean example

build:
	@echo "[make] Building with LLVM_LIB=$(LLVM_LIB)"
	LIBRARY_PATH=$(LLVM_LIB) LD_LIBRARY_PATH=$(LLVM_LIB):$$LD_LIBRARY_PATH LLVM_SYS_USE_SHARED=1 LLVM_SYS_LINK_SHARED=1 cargo build

build-release:
	@echo "[make] Building (release) with LLVM_LIB=$(LLVM_LIB)"
	LIBRARY_PATH=$(LLVM_LIB) LD_LIBRARY_PATH=$(LLVM_LIB):$$LD_LIBRARY_PATH LLVM_SYS_USE_SHARED=1 LLVM_SYS_LINK_SHARED=1 cargo build --release

run:
	@echo "[make] Running with args: $(ARGS)"
	LIBRARY_PATH=$(LLVM_LIB) LD_LIBRARY_PATH=$(LLVM_LIB):$$LD_LIBRARY_PATH LLVM_SYS_USE_SHARED=1 LLVM_SYS_LINK_SHARED=1 cargo run -- $(ARGS)

clean:
	cargo clean
	rm -rf *.o *.ll example

example:
	@echo "[make] Running produced example binary"
	make run ARGS="example.hl"
	./example
