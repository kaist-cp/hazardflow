//! Fetch stage.

use super::*;
use crate::std::hazard::*;
use crate::std::*;

/// Payload from fetch stage to decode stage.
#[derive(Debug, Clone, Copy)]
pub struct FetEP {
    /// IMEM response.
    pub imem_resp: MemRespWithAddr,
}

/// Fetch stage.
pub fn fetch<const START_ADDR: u32>(
    imem: impl FnOnce(Vr<MemReq>) -> Vr<MemRespWithAddr>,
) -> I<VrH<FetEP, DecR>, { Dep::Demanding }> {
    let next_pc = <I<VrH<(HOption<FetEP>, DecR), _>, { Dep::Demanding }>>::source_drop()
        .filter_map(|(p, decr)| {
            let DecR { redirect } = decr;

            match redirect {
                Some(target) => Some(target),
                None => p.map(|p| p.imem_resp.addr + 4),
            }
        })
        .reg_fwd_with_init(true, START_ADDR);

    next_pc
        .map(|pc| MemReq::load(pc, MemOpTyp::WU))
        .comb::<I<VrH<MemRespWithAddr, _>, { Dep::Helpful }>>(attach_resolver(imem))
        .map(|imem_resp| FetEP { imem_resp })
        .map_resolver_drop_with_p::<VrH<FetEP, DecR>>(|ip, er| {
            let DecR { redirect } = er.inner;
            // We need `kill` here to extract the mispredicted PC from register, and then filter out them.
            Ready::new(er.ready || redirect.is_some(), (ip, er.inner))
        })
        .filter_map_drop_with_r_inner(|resp, er| if er.redirect.is_none() { Some(resp) } else { None })
}
