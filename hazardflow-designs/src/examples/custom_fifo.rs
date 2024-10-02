//! Custom FIFO implementation

use crate::prelude::*;
use crate::std::*;

const N: usize = 5;
const M: usize = 5;

/// Masked merge trait
pub trait MaskedMergeExt<P: Copy + Default, const N: usize>: Interface
where [(); clog2(N)]:
{
    /// Hazard type
    type EH: Hazard;

    /// Fair Mux
    fn masked_merge(self) -> I<Self::EH, { Dep::Demanding }>;
}

impl<P: Copy + Default, const N: usize> MaskedMergeExt<P, N> for [Vr<P>; N]
where [(); clog2(N)]:
{
    type EH = VrH<(P, U<{ clog2(N) }>), Array<bool, N>>;

    fn masked_merge(self) -> I<Self::EH, { Dep::Demanding }> {
        unsafe {
            self.fsm::<I<VrH<(P, U<{ clog2(N) }>), Array<bool, N>>, { Dep::Demanding }>, ()>((), |ip, er, s| {
                if !er.ready {
                    let ir = Ready::new(false, ()).repeat();
                    let ep = None;
                    return (ep, ir, s);
                }

                let ep_idx = ip.zip(er.inner).find_idx(|(p, selected)| p.is_some() && !selected);
                let ep = if let Some(idx) = ep_idx { Some((ip[idx].unwrap(), idx)) } else { None };

                let ir = Ready::invalid().repeat::<N>().set_cond(ep.is_some(), ep_idx.unwrap(), Ready::valid(()));
                (ep, ir, s)
            })
        }
    }
}

/// Masked Merge Combinator
#[synthesize]
pub fn custom_fifo(ingress: [Vr<u32>; N]) -> Vr<u32> {
    ingress
        .masked_merge()
        .map_resolver::<((), FifoS<(u32, U<{ clog2(N) }>), M>)>(|er| {
            let (_, fifo_s) = er.inner;
            range::<M>().fold(Array::from([false; N]), |acc, i| {
                if i.resize() >= fifo_s.len {
                    acc
                } else {
                    acc.set(fifo_s.inner[wrapping_add(fifo_s.raddr, i, M.into_u())].1, true)
                }
            })
        })
        .transparent_fifo()
        .map(|(ip, _idx)| ip)
}
