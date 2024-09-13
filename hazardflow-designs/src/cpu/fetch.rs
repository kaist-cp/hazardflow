//! Fetch stage.

use super::*;
use crate::std::hazard::*;
use crate::std::*;

/// Next PC selector.
#[derive(Debug, Clone, Copy)]
pub enum PcSel {
    /// Fetch the next predicted PC.
    ///
    /// If the branch predictor hits, fetch the predicted target address; otherwise, fetch current PC + 4.
    Predict,

    /// Fetch the redirected PC.
    ///
    /// It comes from the branch/jump target misprediction or exception.
    Redirect(u32),
}

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
    let next_pc = <I<VrH<(HOption<FetEP>, PcSel), _>, { Dep::Demanding }>>::source_drop()
        .filter_map(|(p, pc_sel)| match pc_sel {
            PcSel::Redirect(target) => Some(target),
            PcSel::Predict => p.map(|p| p.imem_resp.addr + 4),
        })
        .reg_fwd_with_init(true, START_ADDR);

    next_pc
        .map(|pc| MemReq::load(pc, MemOpTyp::WU))
        .comb::<I<VrH<MemRespWithAddr, _>, { Dep::Helpful }>>(attach_resolver(imem))
        .map(|imem_resp| FetEP { imem_resp })
        .map_resolver_drop_with_p::<VrH<FetEP, DecR>>(|ip, er| {
            let DecR { kill, pc_sel } = er.inner;
            Ready::new(er.ready || kill, (ip, pc_sel)) // We need `kill` here to extract the mispredicted PC from register, and then filter out them.
        })
        .filter_map_drop_with_r_inner(|resp, er| if !er.kill { Some(resp) } else { None })
}
