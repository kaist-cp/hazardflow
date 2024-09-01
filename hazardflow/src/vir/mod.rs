//! Verilog IR.

pub mod analysis;
mod integrate;
/// TODO: make this pub(crate)
mod ir;
/// TODO: make this pub(crate)
pub mod opt;
mod utils;

pub use integrate::*;
pub use ir::*;
