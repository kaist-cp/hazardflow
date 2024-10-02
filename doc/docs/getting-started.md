# Getting Started

HazardFlow HDL allows users to modularly describe pipelined circuits with hazards, achieving cycle-level accuracy.
It provides hazard interfaces and combinators to handle backward dependencies and simplify decomposition.
By the end, users to design and compile hardware modules into Verilog.

<!--
HazardFlow HDL allows users to describe pipelined circuits with hazards in a modular way, achieving cycle-level accuracy.
It introduces hazard interfaces that encapsulate pipeline-backward dependencies into a resolver signal within each interface.
Additionally, HazardFlow HDL provides combinators, inspired by a map-reduce style, to simplify the decomposition of pipelined circuits.
By the end, you will be able to design hardware modules using HazardFlow HDL and compile them into Verilog.
-->

## Installation

Install [rustup](https://rustup.rs/). After the installation is complete, restart the terminal.

```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Clone the [HazardFlow repository](https://github.com/kaist-cp/hazardflow):

```
$ git clone https://github.com/kaist-cp/hazardflow.git
$ cd hazardflow
```

## Compiling HazardFlow Modules to Verilog

First, build the Rust macros:

```
$ cargo build -p hazardflow-macro
```

To generate the Verilog code for 5-stage pipelined RISC-V CPU core:

```bash
# Generate a separate Verilog file for each submodule.
$ cargo run --release -- --target cpu --deadcode --wire-cache --system-task

# Generate an integrated Verilog file combining all submodules.
$ cargo run --release -- --target cpu --deadcode --wire-cache --merge --system-task
```

To generate the Verilog code for systolic-array based NPU core:

```bash
# Generate a separate Verilog file for each submodule.
$ cargo run --release -- --target gemmini --deadcode --wire-cache --system-task

# Generate an integrated Verilog file combining all submodules.
$ cargo run --release -- --target gemmini --deadcode --wire-cache --merge --system-task
```

The generated code is located in `build`, with each top-level module with a `#[synthesize]` attribute in separate directories.
