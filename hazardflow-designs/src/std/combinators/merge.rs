//! Merge.

use super::*;

/// Extension trait for `merge` and `cmerge`.
// Semantically `ED` should be an associated constant instead of a const parameter, but the associated constant version
// doesn't compile.
pub trait MergeExt<const N: usize, const ED: Dep>: Interface
where [(); clog2(N)]:
{
    /// Hazard specification of egress interface.
    type EH: Hazard;

    /// A variation of [`cmerge`](merge) that does not output a control signal that indicates which interface is
    /// selected. See [`cmerge`](merge) for more information.
    fn merge(self) -> I<Self::EH, ED> {
        self.cmerge().into_inner()
    }

    /// Control merge.
    fn cmerge(self) -> I<SelH<Self::EH, N>, ED>;
}

impl<const N: usize, P: Copy, R: Copy, const D: Dep> MergeExt<N, D> for [I<ValidH<P, R>, D>; N]
where [(); clog2(N)]:
{
    type EH = ValidH<P, R>;

    /// Merges `N` `ValidH` hazard interfaces with a control signal that outputs which interface is selected.
    ///
    /// - Payloads: Selects the first ingress interface with a valid payload and outputs that payload.
    /// - Resolver: Duplicated to multiple interfaces.
    ///
    /// | Interface | Ingress                | Egress                      |
    /// | :-------: | ---------------------- | --------------------------- |
    /// |  **Fwd**  | `Array<HOption<P>, N>` | `(HOption<P>, BoundedU<N>)` |
    /// |  **Bwd**  | `Array<R, N>`          | `R`                         |
    fn cmerge(self) -> I<SelH<ValidH<P, R>, N>, D> {
        unsafe {
            self.fsm::<I<SelH<ValidH<P, R>, N>, D>, ()>((), |ip, er, s| {
                let sel = ip.find_idx(|ele| ele.is_some());
                let ep = sel.map(|sel| (ip[sel].unwrap(), BoundedU::new(sel)));
                let ir = er.repeat();
                (ep, ir, s)
            })
        }
    }
}

impl<const N: usize, H: Hazard, const D: Dep> MergeExt<N, { Dep::Demanding }> for [I<AndH<H>, D>; N]
where [(); clog2(N)]:
{
    type EH = AndH<H>;

    /// Merges `N` `AndH<H>` hazard interfaces with a control signal that outputs which interface is selected.
    ///
    /// - Payloads: Selects the first ingress interface on which a transfer can happen
    ///     (`ip[sel].is_some_and(|p| AndH::<H>::ready(p, er))`), and outputs the selected interface's payload.
    /// - Resolver: If the index of the selected interface is `sel`, the ingress ready signals for the interfaces
    ///     `0..=sel` is true, and `(sel + 1)..N` is false. This is to ensure that a transfer happens only on the
    ///     interface `sel`. The inner value `H::R` of the resolver is duplicated to multiple interfaces.
    ///
    /// | Interface | Ingress                   | Egress                         |
    /// | :-------: | ------------------------- | ------------------------------ |
    /// |  **Fwd**  | `Array<HOption<H::P>, N>` | `(HOption<H::P>, BoundedU<N>)` |
    /// |  **Bwd**  | `Array<Ready<H::R>, N>`   | `Ready<H::R>`                  |
    fn cmerge(self) -> I<SelH<AndH<H>, N>, { Dep::Demanding }> {
        // TODO: Write safety comments
        unsafe {
            self.fsm::<I<SelH<AndH<H>, N>, { Dep::Demanding }>, ()>((), |ip, er, s| {
                // Logic for ingress hazard calculation
                //
                // `ir[i].ready` is true if and only if forall j < i, ingress[j] is not transferrable.
                //
                // NOTE: We have to give `er.repeat()`, not `Ready::invalid().repeat()`.
                // This is because the ingress interface can be demanding, which should see the `er` and set the valid bit.
                let (ir, sel) = ip.enumerate().fold((er.repeat::<N>(), None), |(acc_ir, xferred_idx), (idx, ele)| {
                    if xferred_idx.is_some() {
                        // If there exists `j < i` such that `ingress[j]` is transferrable, then `ingress[i]` is not transferrable.
                        //
                        // NOTE: We do not use `Ready::invalid()` because the ingress interface can be demanding, which might see the `er` in the ready function which should not receive don't-care value as `inner`.
                        // TODO: After changing the `Ready::invalid` api to receive `inner` as parameter, we should change this to `Ready::invalid`.
                        (acc_ir.set(idx, Ready::new(false, er.inner)), xferred_idx)
                    } else {
                        let xferred_sel = ele.and_then(|ele| if AndH::<H>::ready(ele, er) { Some(idx) } else { None });
                        (acc_ir, xferred_sel)
                    }
                });

                // We can safely unwrap `sel` since it is guaranteed to be `Some` by the above logic.
                //
                // If `sel` is `Some`, then `ip[sel]` is guaranteed to be `Some`.
                (sel.map(|sel| (ip[sel].unwrap(), BoundedU::new(sel))), ir, s)
            })
        }
    }
}
