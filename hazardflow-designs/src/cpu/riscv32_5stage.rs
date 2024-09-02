//! RISCV 5-stage pipeline CPU

use super::*;
use crate::std::*;

const START_ADDR: u32 = 0x80000000;

/// Core that can execute RISC-V instructions
#[synthesize]
pub fn core(
    imem: impl FnOnce(Vr<MemReq>) -> Vr<MemRespWithAddr>,
    dmem: impl FnOnce(Vr<MemReq>) -> Vr<MemRespWithAddr>,
) {
    fetch::<START_ADDR>(imem).comb(decode).comb(exe).comb(move |ingress| mem(ingress, dmem)).comb(wb)
}