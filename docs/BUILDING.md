# Building

Install Rust stable from the official Rust toolchain instructions, then run:

```sh
cargo check
cargo test
cargo run
cargo build --release
```

Windows: install the Microsoft C++ Build Tools if Cargo reports a missing linker or MSVC toolchain.

macOS: install Xcode Command Line Tools (`xcode-select --install`).

Linux: install your distribution's C/C++ build essentials and the native libraries required by egui and rfd. If a native file dialog dependency fails, consult the error message and install the matching system development package.

Cross-compilation is not guaranteed by this repository; build on each target OS for the most reliable result.
