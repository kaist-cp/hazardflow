# Getting Started

The HazardFlow HDL is a Rust library and compiler plugin.

## Installation

Prerequisite:

```bash
# Dependent packages
apt update
apt install -y curl build-essential git python3 pip

# Install rust. If it asks to select option, choose 1 (default one)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Python packages
pip install numpy scipy matplotlib seaborn parse

# Run this command in our artifact directory (TODO: maybe tweak rust-toolchain.toml?)
rustup component add rustc-dev llvm-tools rust-src llvm-tools-preview
```

Script

1. Clone the repo from Github
```bash
TODO: Will change this part later when we decide how to host the barebone repo. @minseong
```

2. Build `hazardflow-macro`
```bash
# change directory to the macro directory
cd hazardflow-macro

# build hazardflow-macro module
cargo build

# Back to root directory
cd ..
```

3. Running the compiler
```bash
# Compile all modules without any optimization passes
cargo run --release

# Compile modules with "cpu" or "nic" in their definition path, without any optimization passes. For example, `src/cpu/*.rs` and `src/nic/cmac_pad.rs` will be compiled, but `src/netstack/ip_handler.rs` will not.
cargo run --release -- --target cpu cmac_pad

# Compile all modules with system tasks. By default, the compiler will not generate system tasks statements like `display!` or `hassert!`.
cargo run --release -- --system-task

# Compile all modules with deadcode and wire-cache optimizations. By default, the compiler will not perform any optimization passes.
cargo run --release -- --deadcode --wire-cache
```

The generated code will reside in the `build` directory, with each top-level module with a `#[synthesize]` attribute in separate directories.

## Test Your Module

TODO: How to test if the program works correctly. @minseong
