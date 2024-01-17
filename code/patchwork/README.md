# Useful cargo commands

## General

-   Build: `cargo build`
-   Build Release Mode: `cargo build --release`
-   Check: `cargo check --all-features --all-targets`
-   Format: `cargo fmt --all -- --check`
-   Clippy: `cargo clippy --all -- -D warnings`

## Run and test

-   Run: `cargo run --bin console -- -1 greedy -2 random`
-   Run with release optimizations: `cargo run --release`
-   Run tests: `cargo test --workspace --all-targets`
-   Run doctests: `cargo test --workspace --doc`
-   Run performance tests: `cargo test --no-default-features --features performance_tests --release --workspace --all-targets performance_tests -- --nocapture --test-threads=1`
    -   Once stable: change performance tests to cargo bench

## Dependency management

-   Update dependencies: `cargo update`
-   See dependency tree `cargo tree`
-   See unused dependencies `cargo +nightly udeps`
    -   Needs to be installed ([cargo-udeps](https://github.com/est31/cargo-udeps))
-   Expand macros `cargo expand`
    -   Needs to be installed ([cargo-expand](https://github.com/dtolnay/cargo-expand))

# Debugging

## Debugging in vscode

-   Use the Run and Debug

## Debugging with lldb

1. Install lldb from [Winlibs](https://winlibs.com/#download-release) (UCRT runtime, GCC 13.2.0 with POSIX threads with LLVM/CLANG/LLD/LLDB)
2. Start lldb: `path/to/lldb.exe target/debug/console.exe`
3. (Optional) Set breakpoints: `breakpoint set --file src/main.rs --line 10`
4. Launch the program: `process launch`
5. (Optional) Continue execution: `process continue`
6. (Optional) See backtrace after stack overflow: `thread backtrace`

# BUG:

-   No draw possible. Needs to be implemented.
-   The 7x7 board is wrongly implemented.
