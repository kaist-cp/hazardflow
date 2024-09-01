//! Hazardflow standard library.
//!
//! # Notable APIs
//!
//! This section lists out notable APIs of the Hazardflow standard library.
//!
//! ## Builtin value types
//!
//! - [`HOption<T>`]
//! - [`Array<V, N>`]
//! - [`U<N>`]
//! - [`BoundedU<MAX, WIDTH>`]
//!
//! ## Hazards and interfaces
//!
//! ### Traits
//!
//! - [`Hazard`]
//! - [`Interface`]
//!
//! ### Hazard interfaces
//!
//! - Interface [`I<H, D>`]
//!     - [`Dep`]
//! - Hazard [`AndH<H>`]
//!     - [`Ready<R>`]
//! - Hazard [`ValidH<P, R>`]
//! - Hazard [`VrH<P, R>`]
//!
//! ### Valid interface
//!
//! - Interface [`Valid<P>`]
//!
//! ### Valid-ready interface
//!
//! - Interface [`Vr<P, D>`]
//!
//! ## Module functions
//!
//! - See [`module`] for general module functions.
//! - See [`valid_ready`] for module funtions for modules with `VrH` hazard or valid-ready interfaces.
//!
//! ## Combinators
//!
//! - See [`combinators`] for combinator documentation and implementations.
//!
//! ## Utility functions and macros
//!
//! - See [`utils`] for utility functions.
//! - [`display`](crate::display!)
//! - [`hassert`](crate::hassert!)
//! - [`hpanic`](crate::hpanic!)

pub mod combinators;
pub mod hazard;
pub mod interface;
pub mod module;
pub mod utils;
pub mod valid;
pub mod valid_ready;
pub mod value;

use core::marker::*;
use core::ops::*;

pub use combinators::*;
pub use hazard::*;
pub use interface::*;
pub use module::*;
pub use utils::*;
pub use valid::*;
pub use valid_ready::*;
pub use value::*;
