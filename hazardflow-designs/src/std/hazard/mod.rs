//! Hazard protocol.

pub mod selector;

use core::marker::{ConstParamTy, PhantomData};

pub use mux::*;
pub use selector::*;

use super::interface::*;
use crate::prelude::*;
use crate::std::*;

/// A hazard protocol with given payload, resolver, and ready function.
///
/// A struct represents a hazard protocol when it implements this trait.
pub trait Hazard {
    /// Payload type.
    type P: Copy;

    /// Resolver type.
    type R: Copy;

    /// Indicates whether the receiver of the payload is ready to receive the payload.
    ///
    /// This ready condition is not automatically enforced by just using a hazard interface. If you want to enforce the
    /// condition, you may use `Hazard::ready` directly in the combinational logic. Note that all the `std` combinators
    /// already check the condition.
    fn ready(p: Self::P, r: Self::R) -> bool;
}

/// Dependency type of a hazard interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ConstParamTy)]
pub enum Dep {
    /// The payload (`Fwd`) does not depend on the resolver (`Bwd`).
    Helpful = 0,
    /// The payload (`Fwd`) depends on the resolver (`Bwd`), and they satisfy the condition that if the payload is
    /// `Some`, `Hazard::ready(p, r)` is true.
    ///
    /// It is a bug to make the payload depend on the resolver but break the condition.
    Demanding = 1,
}

/// Hazard interface.
#[derive(Debug)]
#[must_use]
pub struct I<H: Hazard, const D: Dep> {
    _marker: PhantomData<H>,
}

impl<H: Hazard, const D: Dep> Interface for I<H, D> {
    /// Resolver.
    type Bwd = H::R;
    /// Payload.
    ///
    /// `Some(p)` means a valid payload with data `p`, and `None` means an invalid payload.
    type Fwd = HOption<H::P>;
}

/// Wrapping resolver type for `AndH`.
#[derive(Debug, Clone, Copy)]
pub struct Ready<R> {
    /// Whether the receiver of the payload is ready to accept a new payload.
    pub ready: bool,

    /// Inner resolver type.
    pub inner: R,
}

impl<R: Copy> Ready<R> {
    /// Generates a new `Ready` with the given `ready` bit and inner resolver.
    pub fn new(ready: bool, inner: R) -> Self {
        Self { ready, inner }
    }

    /// Creates a new invalid signal.
    // TODO: We should add `inner` as parameter to set the inner hazard value when creating invalid signal.
    //       This is needed because the inner hazard value should be allowed as don't-care value only when explicit `unsafe` reasoning by user is given.
    #[allow(unreachable_code)]
    pub fn invalid() -> Self {
        Self { ready: false, inner: unsafe { x::<R>() } }
    }

    /// Creates a new valid signal.
    pub fn valid(inner: R) -> Self {
        Ready::new(true, inner)
    }
}

/// Transforms `Ready<R>` to `Option<R>`.
///
/// It is mainly used when the structural hazard (ready bit) has higher priority than data/control hazards.
impl<R: Copy> From<Ready<R>> for HOption<R> {
    fn from(value: Ready<R>) -> Self {
        if value.ready {
            // If the ready bit high, pass the inner hazard.
            Some(value.inner)
        } else {
            // Otherwise, block.
            None
        }
    }
}

impl<R1> Ready<R1> {
    /// Maps the inner resolver to another inner resolver type.
    pub fn map<R2>(self, f: impl Fn(R1) -> R2) -> Ready<R2> {
        Ready { ready: self.ready, inner: f(self.inner) }
    }
}

/// Hazard for wrapping a hazard with a `ready` bit (to represent a structural hazard).
#[derive(Debug, Clone, Copy)]
pub struct AndH<H: Hazard> {
    _marker: PhantomData<H>,
}

impl<H: Hazard> Hazard for AndH<H> {
    type P = H::P;
    type R = Ready<H::R>;

    fn ready(p: H::P, r: Ready<H::R>) -> bool {
        if r.ready {
            H::ready(p, r.inner)
        } else {
            false
        }
    }
}

impl<H: Hazard, const D: Dep> I<H, D> {
    /// A generic FSM combinator for a hazard interface.
    ///
    /// For more information, you can check the documentation for [`Interface::fsm`].
    ///
    /// # Safety
    ///
    /// To enforce the invariant of the hazard protocol, you have to consider the following depending on the dependency
    /// type of the ingress/egress interface.
    ///
    /// - Ingress interface
    ///     - [`Dep::Helpful`]: In the ingress interface's `fsm`, its payload does not depend on its resolver. So in
    ///         this `fsm`, we can use the fact that `ip` does not depend on `ir`. That means we can make `ir` depend on
    ///         `ip`.  
    ///     - [`Dep::Demanding`]: In the ingress interface's `fsm`, its payload depends on its resolver, and if the
    ///         payload is `Some`, `Hazard::ready(p, r)` is true. So in this `fsm`, we must consider that `ip` depends
    ///         on `ir`, but can assume that if `ip` is `Some`, `H::ready(ip, ir)` is true regardless of `ir`.  
    /// - Egress interface
    ///     - [`Dep::Helpful`]: In this `fsm`, we ensure that `ep` does not depend on `er`. If the dependency chain goes
    ///         through the ingress interface, we must consider that as well.  
    ///     - [`Dep::Demanding`]: In this `fsm`, we make `ep` depend on `er`, and guarantee that if `ep` is `Some`,
    ///         `EH::ready(ep, er)` is true.
    ///
    /// # Type parameters
    ///
    /// - `H`: The ingress interface's hazard type.
    /// - `D`: The ingress interface's dependency type.
    /// - `S`: The state type.
    /// - `ED`: The egress interface's dependency type.
    /// - `EH`: The egress interface's hazard type.
    ///
    /// # Parameters
    ///
    /// - `self`: The ingress interface.
    /// - `init_state`: The initial state.
    /// - `f`: Output calculation and state transition logic. If `let (ep, ir, s_next) = f(ip, er, s)`,
    ///     - `ip`: The ingress payload.
    ///     - `er`: The egress resolver.
    ///     - `s`: The current state.
    ///     - `ep`: The egress payload.
    ///     - `ir`: The ingress resolver.
    ///     - `s_next`: The next state.
    ///
    /// # Note: preventing combinational loops
    ///
    /// Combinational loops are among the most common causes of instability and unreliability in digital designs.
    /// Combinational loops generally violate synchronous design principles by establishing a direct feedback loop that
    /// contains no registers.
    ///
    /// - To prevent combinational loops, programmers have to make sure that **there is no circular dependency between
    ///     the payload and resolver of the same interface**.
    /// - Dependency types help with this.
    pub unsafe fn fsm<S: Copy, const ED: Dep, EH: Hazard>(
        self,
        init_state: S,
        f: impl Fn(HOption<H::P>, EH::R, S) -> (HOption<EH::P>, H::R, S),
    ) -> I<EH, ED> {
        unsafe { <Self as Interface>::fsm::<I<EH, ED>, S>(self, init_state, f) }
    }
}
