//! Combinators.
//!
//! # How to read this documentation
//!
//! If you're reading the source code, you can go to each module listed in the [Categories](#categories) section to see
//! the combinators' documentation and implementation.
//!
//! If you're viewing the generated documentation in the browser, you should go to the following pages to see the
//! documentation.
//!
//! - Combinators implemented directly on an interface (All combinators other than N-to-1 combinators)
//!     - [`I<H, D>`#implementations]
//! - Combinators implemented on tuples or arrays (N-to-1 combinators)
//!     - [`JoinExt`#foreign-impls]
//!     - [`JoinValidExt`#foreign-impls]
//!     - [`JoinVrExt`#foreign-impls]
//!     - [`ZipAnyExt`#foreign-impls]
//!     - [`ZipAnyValidExt`#foreign-impls]
//!     - [`MergeExt`#foreign-impls]
//!     - [`MuxExt`#foreign-impls]
//!
//! # Categories
//!
//! The combinators can be organized into the following categories.
//!
//! - Mapping
//!     - [`filter_map`]
//!     - [`filter`]
//!     - [`map`]
//!     - [`map_resolver`]
//!     - [`flatten`]
//! - 1-to-N
//!     - Distribute to all
//!         - [`fork`]
//!         - [`unzip`]
//!     - Distribute to some
//!         - [`fork_some`]
//!         - [`unzip_some`]
//!     - Distribute to one
//!         - [`branch`]
//! - N-to-1
//!     - Keep all
//!         - [`join`]
//!     - Keep some
//!         - [`zip_any`]
//!     - Choose one
//!         - [`merge`]
//!         - [`mux`]
//! - Register
//!     - [`reg`]
//!     - [`fifo`]
//! - Source/sink
//!     - [`sink`]
//!     - [`source`]
//! - FSM
//!     - [`fsm_map`]
//!     - [`fsm_ingress`]
//!     - [`fsm_egress`]
//! - Conversion
//!     - [`convert`]
//!
//! # Naming conventions
//!
//! The combinators have a main name, and can have various prefixes and suffixes clarifying their behavior.
//!
//! - Combinators that may change the payload/resolver (Mapping, Source/sink, FSM combinators)
//!     - No additional words: Other than what the combinator itself does, does not change the validity of the payload
//!         nor the readiness of the resolver.
//!     - Suffix `drop`: If the egress hazard ready condition (`EH::ready`) is false, the egress payload becomes `None`.
//!     - Suffix `block`: If the egress hazard ready condition (`EH::ready`) is false, send an additional "not ready"
//!         signal to the ingress resolver.
//! - Combinators with an internal state (Register, FSM combinators)
//!     - Prefix `transparent`: Outputs the internal state to the ingress resolver.
//! - Conversion combinators
//!     - Start with `into`: The combinator doesn't change the behavior in a meaningful way. You can just use it to get
//!         the type you want.
//!     - Start with other words (`discard`/`always`/`drop`/`block`): The combinator does change the behavior. Refer to
//!         each combinator's documentation for more information.
//! - Combinators with a closure argument
//!     - Suffix `with_p`/`with_r`: The closure takes an additional payload/resolver parameter.
//!     - (For `I<VrH<P, R>, _>`) Suffix `inner`: The closure takes the inner value `R` of the resolver instead of the
//!         whole `Ready<R>`.
//!
//! # Notes on dropping combinators
//!
//! If a combinator returns a [`Dep::Demanding`] interface because of the semantics of the combinator, it has to have a
//! dropping behavior to force the required condition for [`Dep::Demanding`]. (If the payload is `Some`,
//! `Hazard::ready(p, r)` is true.) Note that for `I<ValidH<P, R>, _>`, this is unnecessary as `ValidH::ready` is always
//! true.
//!
//! For the combinators implemented on a generic hazard interface `I<H, _>` that allows the caller to choose the egress
//! hazard `EH`, the returned interface is forced to be [`Dep::Demanding`], making them have a dropping behavior.
//! This is because the combinator first has to check the ingress transfer condition
//! (`ip.is_some_and(|p| H::ready(p, ir))`). Otherwise, checking `H::ready` will never be done as the hazard type and
//! thus the ready condition is changed to `EH::ready`. This makes the egress payload depend on the ingress resolver,
//! and in turn the egress resolver.

// Note that this has to be above `mod`s since it uses textual scope.
/// Adopted from https://veykril.github.io/tlborm/decl-macros/patterns/repetition-replacement.html.
macro_rules! replace {
    ($_t:tt, $($sub:tt)+) => {
        $($sub)+
    };
}

// Mapping
pub mod filter;
pub mod filter_map;
pub mod flatten;
pub mod map;
pub mod map_resolver;

// 1-to-N
pub mod branch;
pub mod fork;
pub mod fork_some;
pub mod unzip;
pub mod unzip_some;

// N-to-1
pub mod join;
pub mod merge;
pub mod mux;
pub mod zip_any;

// Register
pub mod fifo;
pub mod reg;

// Source/sink
pub mod sink;
pub mod source;

// FSM
pub mod fsm_egress;
pub mod fsm_ingress;
pub mod fsm_map;

// Conversion
pub mod convert;

// Other
pub mod generator;

pub use fifo::*;
pub use join::*;
pub use merge::*;
pub use mux::*;
pub use zip_any::*;

use super::hazard::*;
use super::valid::*;
use crate::prelude::*;
use crate::std::*;
