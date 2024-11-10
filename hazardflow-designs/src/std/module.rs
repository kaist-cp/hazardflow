//! Module functions to operate on modules.

#![allow(clippy::type_complexity)]
#![allow(unused)]

use hazardflow_macro::magic;

use super::*;
use crate::prelude::*;

/// Splits a module into two modules.
// TODO: Can we make return type `FnOnce(I1) -> O1`?
#[magic(module::split)]
pub fn module_split<I1: Interface, I2: Interface, O1: Interface, O2: Interface>(
    _m: impl FnOnce(I1, I2) -> (O1, O2),
) -> (fn(I1) -> O1, fn(I2) -> O2) {
    compiler_magic!()
}

/// Splits a module into three modules.
pub fn module_split3<I1: Interface, I2: Interface, I3: Interface, O1: Interface, O2: Interface, O3: Interface>(
    m: impl FnOnce(I1, I2, I3) -> (O1, O2, O3),
) -> (impl FnOnce(I1) -> O1, impl FnOnce(I2) -> O2, impl FnOnce(I3) -> O3) {
    let (m1, m23) = module_split(move |i1, (i2, i3)| {
        let (e1, e2, e3) = m(i1, i2, i3);
        (e1, (e2, e3))
    });

    let (m2, m3) = module_split(move |i2, i3| m23((i2, i3)));

    (m1, m2, m3)
}

/// Generates an array of modules.
// TODO: Modify `f` to be `f: impl FnOnce(n: usize) -> T`.
#[magic(module::from_fn)]
pub fn from_fn<I: Interface, O: Interface, J: Interface, T, const N: usize>(f: T) -> [fn(I, J) -> (O, J); N]
where T: FnOnce(I, J) -> (O, J) {
    compiler_magic!()
}

/// Generates a 1D systolic array from an array of modules.
///
/// ```text
///       I           I         ...           I
///       ↓           ↓                       ↓
/// J → ms[0] → J → ms[1] → J → ... → J → ms[N - 1] → J
///       ↓           ↓                       ↓
///       O           O         ...           O
/// ```
#[magic(module::seq)]
pub fn seq<I: Interface, O: Interface, J: Interface, const N: usize>(
    ms: [fn(I, J) -> (O, J); N],
) -> impl FnOnce([I; N], J) -> ([O; N], J) {
    // This should be primitive?
    |is, j| compiler_magic!()
}

/// Flips a module's input and output.
pub fn flip<I1: Interface, I2: Interface, O1: Interface, O2: Interface>(
    f: impl FnOnce(I1, I2) -> (O1, O2),
) -> impl FnOnce(I2, I1) -> (O2, O1) {
    move |i2, i1| {
        let (o1, o2) = f(i1, i2);
        (o2, o1)
    }
}

/// Wraps `m` to guarantee that there is at most one data processing in the module.
///
/// NOTE: `m` should return one outgoing data for one incoming data.
// TODO: Write down the condition of `m` to avoid the combinational loop.
// TODO: Implement it with fork-join with flow register.
pub fn exclusive<H: Hazard, EH: Hazard, const D: Dep, const ED: Dep>(
    m: impl FnOnce(I<AndH<H>, D>) -> I<EH, ED>,
) -> impl FnOnce(I<AndH<H>, D>) -> I<EH, ED> {
    move |i| {
        let (m_resp_tx, m_resp_rx) = channel::<I<EH, ED>>();
        let m_resp = ().comb(m_resp_rx);

        let (e, m_req) = unsafe {
            (i, m_resp).fsm::<(I<EH, ED>, I<AndH<H>, D>), bool>(false, |(ip1, ip2), (er1, er2), s| {
                let ep1 = ip2;
                let et1 = ep1.is_some_and(|p| EH::ready(p, er1));

                let ep2 = if s && !et1 { None } else { ip1 };
                let et2 = ep2.is_some_and(|p| AndH::<H>::ready(p, er2));

                let ir1 = if !s || et1 { er2 } else { Ready::new(false, er2.inner) };
                let it1 = ip1.is_some_and(|p| AndH::<H>::ready(p, ir1));

                let ir2 = er1;

                let s_next = if !s && it1 && et1 {
                    false
                } else if it1 {
                    true
                } else if et1 {
                    false
                } else {
                    s
                };

                ((ep1, ep2), (ir1, ir2), s_next)
            })
        };

        m_req.comb(m).comb(m_resp_tx);

        e
    }
}

/// Returns a sender and a receiver.
// TODO: Maybe we need to change the type of receiver as `impl FnOnce() -> I`. Currently did not do it due to compile error.
pub fn channel<I: Interface>() -> (impl FnOnce(I), impl FnOnce(()) -> I) {
    let m = move |i, ()| ((), i);
    module_split(m)
}
