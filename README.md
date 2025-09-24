# Hyperlight
Hyperlight is a high-performance Rust-based programming
language designed for HPC (High-Performance Computing) applications,
scientific computing, data-intensive tasks, and working with
high precision data types.

## Quick start
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

## Inspiration
A while ago, I was stumbled across a statement on Reddit that essentially said "There are not enough FORTRAN programmers anymore." I had already created some small, interpreter-based languages in Python, so for the next year, I tried on and off to create my first compiled language. I wanted to make something that was easy to use, had a familiar syntax, and could be used for high-performance computing tasks. Thus, Hyperlight was born, a language that aims to combine FORTRAN-like performance with Python-like ease of use and syntax from Python and Rust.

## Example Output
```sh
[username]@[computer-name]:~/dev/hyperlight$ make buildc
LIBRARY_PATH=/usr/lib/llvm-18/lib LD_LIBRARY_PATH=/usr/lib/llvm-18/lib:$LD_LIBRARY_PATH LLVM_SYS_USE_SHARED=1 LLVM_SYS_LINK_SHARED=1 cargo build --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
[username]@[computer-name]:~/dev/hyperlight$ make buildp ARGS="example.hl"
LIBRARY_PATH=/usr/lib/llvm-18/lib LD_LIBRARY_PATH=/usr/lib/llvm-18/lib:$LD_LIBRARY_PATH LLVM_SYS_USE_SHARED=1 LLVM_SYS_LINK_SHARED=1 cargo run -- example.hl
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/hyperlight example.hl`
wrote executable to example
[username]@[computer-name]:~/dev/hyperlight$ time ./example
4
4.000000

real    0m0.003s
user    0m0.001s
sys     0m0.000s
[username]@[computer-name]:~/dev/hyperlight$ 
```
