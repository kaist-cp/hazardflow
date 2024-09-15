//! Writeback stage.

use super::*;

/// Register file.
pub type Regfile = Array<u32, REGS>;

/// Register.
#[derive(Debug, Clone, Copy)]
pub struct Register {
    /// Address.
    pub addr: U<{ clog2(REGS) }>,

    /// Data.
    pub data: u32,
}

impl Register {
    /// Creates a new register.
    pub fn new(addr: U<{ clog2(REGS) }>, data: u32) -> Self {
        Self { addr, data }
    }
}

/// Hazard from writeback stage to memory stage.
#[derive(Debug, Clone, Copy, Default)]
pub struct WbR {
    /// Bypassed data from WB.
    pub bypass_from_wb: HOption<Register>,

    /// Register file.
    pub rf: Regfile,
}

impl WbR {
    /// Creates a new writeback register.
    pub fn new(bypass_from_wb: HOption<Register>, rf: Regfile) -> Self {
        Self { bypass_from_wb, rf }
    }
}

/// Writeback stage.
pub fn wb(i: I<VrH<MemEP, WbR>, { Dep::Demanding }>) {
    i.map_resolver_inner::<(HOption<MemEP>, Regfile)>(|(wb, rf)| WbR::new(wb.and_then(|p| p.wb), rf))
        .reg_fwd(true)
        .sink_fsm_map(0.repeat(), |ip, rf| {
            let ir = Ready::valid((ip, rf));
            let rf_next = match ip {
                Some(MemEP { wb: Some(r), .. }) => rf.set(r.addr, r.data),
                _ => rf,
            };

            if let Some(p) = ip {
                match p.wb {
                    Some(r) => {
                        display!(
                            "retire=[1] pc=[%x] inst=[%x] write=[r%d=%x]",
                            ip.map(|x| x.debug_pc).unwrap_or(0),
                            ip.map(|x| x.debug_inst).unwrap_or(0),
                            r.addr,
                            r.data
                        );
                    }
                    None => {
                        display!(
                            "retire=[1] pc=[%x] inst=[%x]",
                            ip.map(|x| x.debug_pc).unwrap_or(0),
                            ip.map(|x| x.debug_inst).unwrap_or(0)
                        );
                    }
                }
            } else {
                display!("retire=[0]");
            }

            (ir, rf_next)
        })
}
