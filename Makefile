LLVM_LIB ?= /usr/lib/llvm-18/lib
export LLVM_SYS_USE_SHARED=1
export LLVM_SYS_LINK_SHARED=1

.PHONY: buildc buildc-release buildp clean example

buildc:
	LIBRARY_PATH=$(LLVM_LIB) LD_LIBRARY_PATH=$(LLVM_LIB):$$LD_LIBRARY_PATH LLVM_SYS_USE_SHARED=1 LLVM_SYS_LINK_SHARED=1 cargo build --workspace

buildc-release:
	LIBRARY_PATH=$(LLVM_LIB) LD_LIBRARY_PATH=$(LLVM_LIB):$$LD_LIBRARY_PATH LLVM_SYS_USE_SHARED=1 LLVM_SYS_LINK_SHARED=1 cargo build --workspace --release

buildp:
	LIBRARY_PATH=$(LLVM_LIB) LD_LIBRARY_PATH=$(LLVM_LIB):$$LD_LIBRARY_PATH LLVM_SYS_USE_SHARED=1 LLVM_SYS_LINK_SHARED=1 cargo run -- $(ARGS)

clean:
	cargo clean
	rm -rf *.o *.ll example

example:
	make buildp ARGS="example.hl"
	./example
