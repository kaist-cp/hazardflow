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
    i.map_resolver_inner::<(HOption<MemEP>, Regfile)>(|(wb, rf)| {
        display!("rf[0]: %x", rf[0]);
        display!("rf[1]: %x", rf[1]);
        display!("rf[2]: %x", rf[2]);
        display!("rf[3]: %x", rf[3]);
        display!("rf[4]: %x", rf[4]);
        display!("rf[5]: %x", rf[5]);
        display!("rf[6]: %x", rf[6]);
        display!("rf[7]: %x", rf[7]);
        display!("rf[8]: %x", rf[8]);
        display!("rf[9]: %x", rf[9]);
        display!("rf[10]: %x", rf[10]);
        display!("rf[11]: %x", rf[11]);
        display!("rf[12]: %x", rf[12]);
        display!("rf[13]: %x", rf[13]);
        display!("rf[14]: %x", rf[14]);
        display!("rf[15]: %x", rf[15]);
        display!("rf[16]: %x", rf[16]);
        display!("rf[17]: %x", rf[17]);
        display!("rf[18]: %x", rf[18]);
        display!("rf[19]: %x", rf[19]);
        display!("rf[20]: %x", rf[20]);
        display!("rf[21]: %x", rf[21]);
        display!("rf[22]: %x", rf[22]);
        display!("rf[23]: %x", rf[23]);
        display!("rf[24]: %x", rf[24]);
        display!("rf[25]: %x", rf[25]);
        display!("rf[26]: %x", rf[26]);
        display!("rf[27]: %x", rf[27]);
        display!("rf[28]: %x", rf[28]);
        display!("rf[29]: %x", rf[29]);
        display!("rf[30]: %x", rf[30]);
        display!("rf[31]: %x", rf[31]);

        WbR { wb: wb.and_then(|p| p.wb.map(|reg| Register::new(reg.addr, reg.data))), rf, retire: wb.is_some() }
    })
    .reg_fwd(true)
    .sink_fsm_map(0.repeat(), |ip, rf| {
        let ir = Ready::valid((ip, rf));
        let rf_next = match ip {
            Some(MemEP { wb: Some(r), .. }) => rf.set(r.addr, r.data),
            _ => rf,
        };

        display!("retire: [%b], pc: [%x]", ip.is_some(), ip.map(|x| x.debug_pc).unwrap_or(0));

        (ir, rf_next)
    })
}
