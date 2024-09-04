//! Writeback stage.

use super::*;
use crate::prelude::*;
use crate::std::clog2;
use crate::std::hazard::*;
use crate::std::valid_ready::*;

/// Number of registers.
pub const REGS: usize = 32;
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
    /// Writeback.
    ///
    /// It contains the writeback address and data.
    pub wb: HOption<Register>,

    /// Register file.
    pub rf: Regfile,

    /// Indicates that the writeback stage is valid or not.
    pub retire: bool,
}

/// Writeback stage.
pub fn wb(i: I<VrH<MemEP, WbR>, { Dep::Demanding }>) {
    i.map_resolver_inner::<(HOption<MemEP>, Regfile)>(|(wb, rf)| WbR {
        wb: wb.and_then(|p| p.wb.map(|reg| Register::new(reg.addr, reg.data))),
        rf,
        retire: wb.is_some(),
    })
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
                    display!("retire=[1] pc=[%x] write=[r%d=%x]", ip.map(|x| x.debug_pc).unwrap_or(0), r.addr, r.data)
                }
                None => display!("retire=[1] pc=[%x]", ip.map(|x| x.debug_pc).unwrap_or(0)),
            }
        } else {
            display!("retire=[0] pc=[%x]", ip.map(|x| x.debug_pc).unwrap_or(0))
        }

        (ir, rf_next)
    })
}
