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
# or manually run if you ran the buildp command
./example
```

If your LLVM libraries are installed in a non-standard location, change the
`LLVM_LIB` variable in the Makefile to point to the correct path.
