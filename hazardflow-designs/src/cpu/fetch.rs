//! Fetch stage.

use super::*;
use crate::std::hazard::*;
use crate::std::*;

/// Next PC selector.
///
/// This struct is generated at execute stage, and used for accessing instruction memory at fetch stage.
// TODO: add jump's origin, ...
#[derive(Debug, Clone, Copy)]
pub enum PcSel {
    /// PC + 4.
    ///
    /// Current PC at fetch stage is used for calculation.
    Plus4,

    /// PC.
    ///
    /// It is used when `fencei` instruction is in the decode or execute stage.
    Curr,

    /// Jmp target.
    ///
    /// It comes from the Br/J instructions.
    Jmp(u32),

    /// Exception.
    Exception(u32),
}

/// Fetch stage.
pub fn fetch<const START_ADDR: u32>(
    imem: impl FnOnce(Vr<MemReq>) -> Vr<MemRespWithAddr>,
) -> I<VrH<MemRespWithAddr, (bool, PcSel)>, { Dep::Demanding }> {
    let next_pc = <I<VrH<(HOption<MemRespWithAddr>, PcSel), _>, { Dep::Demanding }>>::source_drop()
        .filter_map(|(p, pc_sel)| match pc_sel {
            PcSel::Jmp(target) | PcSel::Exception(target) => Some(target),
            PcSel::Curr => p.map(|p| p.addr),
            PcSel::Plus4 => p.map(|p| p.addr + 4),
        })
        .reg_fwd_with_init(true, START_ADDR);

    next_pc
        .map(|pc| MemReq::load(pc, MemOpTyp::WU))
        .comb::<I<VrH<MemRespWithAddr, _>, { Dep::Helpful }>>(attach_resolver(imem))
        .map_resolver_drop_with_p::<VrH<MemRespWithAddr, (bool, PcSel)>>(|ip, er| {
            let (kill, pc_sel) = er.inner;
            Ready::new(er.ready || kill, (ip, pc_sel)) // We need `kill` here to extract the mispredicted PC from register, and then filter out them.
        })
        .filter_map_drop_with_r_inner(|resp, (killed, _)| if !killed { Some(resp) } else { None })
}
