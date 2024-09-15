//! RISC-V Sodor 5-stage.
//!
//! # References
//!
//! - 5-stage constants: <https://github.com/ucb-bar/riscv-sodor/tree/ef6d156fdafdcc79550ce12e45c7daf7a02a4e11/src/main/scala/sodor/rv32_5stage/consts.scala>
//! - Memory op constants: <https://github.com/ucb-bar/riscv-sodor/tree/ef6d156fdafdcc79550ce12e45c7daf7a02a4e11/src/main/scala/sodor/common/memory.scala#L28>

#![allow(clippy::type_complexity)]

pub mod alu;
pub mod branch_predictor;
pub mod csr;
pub mod decode;
pub mod exe;
pub mod fetch;
pub mod mem;
pub mod mem_interface;
pub mod multiplier;
pub mod riscv32_5stage;
pub mod riscv_isa;
pub mod wb;

pub use alu::*;
pub use branch_predictor::*;
pub use csr::*;
pub use decode::*;
pub use exe::*;
pub use fetch::*;
pub use mem::*;
pub use mem_interface::*;
pub use multiplier::*;
pub use riscv_isa::*;
pub use wb::*;

use crate::prelude::*;
use crate::std::*;
