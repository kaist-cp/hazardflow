//! Builtin value types.

use hazardflow_macro::magic;

use crate::prelude::*;

mod array;
mod bounded;
mod option;
mod sint;
mod uint;

pub use array::*;
pub use bounded::*;
pub use option::*;
pub use sint::*;
pub use uint::*;

/// Don't care value.
///
/// # Safety
///
/// TODO: Write safety condition
#[magic(x)]
pub unsafe fn x<T: Copy>() -> T {
    compiler_magic!()
}
