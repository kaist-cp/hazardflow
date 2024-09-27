//! Builtin value types.

mod array;
mod bounded;
mod integer;
mod option;

pub use array::*;
pub use bounded::*;
use hazardflow_macro::magic;
pub use integer::*;
pub use option::*;

/// Don't care value.
///
/// # Safety
///
/// TODO: Write safety condition
#[magic(x)]
pub unsafe fn x<T: Copy>() -> T {
    panic!("compiler magic")
}
