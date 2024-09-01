//! Optimizations.
//!
//! TODO: Move optimizations to LIR.

mod dead_code;
mod inline_always;
mod wire_cache;

pub use dead_code::*;
pub use inline_always::*;
pub use wire_cache::*;
