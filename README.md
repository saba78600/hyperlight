# Hyperlight
Hyperlight is a high-performance Rust-based programming
language designed for HPC (High-Performance Computing) applications,
scientific computing, data-intensive tasks, and working with
high precision data types.

Quick start
-----------

Development builds use the system LLVM. To build and run a test file named
`example.hl` (project includes a small example), you can use the provided
Makefile targets or the helper script:

```bash
# Build the compiler
make buildc

# Build a program
make buildp ARGS="example.hl"

# Build and run the example
make example
```

If your LLVM libraries are installed in a non-standard location, set
`LLVM_LIB` when invoking make, for example:

```bash
LLVM_LIB=/opt/llvm-18/lib make build
```

Contributing
------------

Please open issues or PRs. Keep changes small and focused. For changes that
affect the code generator, ensure you run `make build` and `make example`.
