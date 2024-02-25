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
-   Run performance tests: `cargo bench --bench performance`
    -   Optimally with baseline: `cargo bench --bench performance -- --baseline baseline_name`
    -   Save results: `cargo bench --bench performance -- --save-baseline baseline_name`
    -   The baselines are saved in the `target/criterion` folder

## Dependency management

-   Update dependencies: `cargo update`
-   See dependency tree `cargo tree`
-   See unused dependencies `cargo +nightly udeps`
    -   Needs to be installed ([cargo-udeps](https://github.com/est31/cargo-udeps))
-   Expand macros `cargo expand`
    -   Needs to be installed ([cargo-expand](https://github.com/dtolnay/cargo-expand))
-   Cargo Repl `evcxr`
    -   Needs to be installed ([Evcxr Rust REPL](https://github.com/evcxr/evcxr/blob/main/evcxr_repl/README.md))
    -   Also possible to be used as jupyter kernel

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

# Utility function

```rust
fn print_board(board: u128) {
    let mut result = String::new();

    for row in 0..9{
        for column in 0..9 {
            let index = row * 9 + column;
            let tile = if (board >> index) & 1 > 0 { "█" } else { "░" };
            result.push_str(tile);
        }
        result.push('\n');
    }

    println!("{}", result);
}
```

## Missing DLLs

With Visual Studio installed the `dumpbin.exe` can be used to check for missing DLLs.
Usually found in `C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.38.33130\bin\Hostx64\x64`

```sh
.\dumpbin.exe /dependents "<path/to/exe>"
.\dumpbin.exe /imports "<path/to/exe>"
```

## Intel MKL Library

The MKL library has to be installed separately and be on the PATH to be used.

The `libiomp5md.dll` dynamic library is required for MKL to work. It can be
found in the `C:\Program Files (x86)\Intel\oneAPI\compiler\latest\bin` folder.
