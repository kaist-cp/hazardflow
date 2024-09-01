//! Rocc related code

use super::*;

/// <https://github.com/chipsalliance/rocket-chip/blob/master/src/main/scala/rocket/CSR.scala#L18>
#[derive(Debug, Clone, Copy)]
pub struct MStatus {
    pub debug: bool,
    pub cease: bool,
    pub wfi: bool,
    pub isa: U<32>,

    pub dprv: U<2>, // effective prv for data accesses
    pub dv: bool,   // effective v for data accesses
    pub prv: U<2>,
    pub v: bool,

    pub sd: bool,
    pub zero2: U<23>,
    pub mpv: bool,
    pub gva: bool,
    pub mbe: bool,
    pub sbe: bool,
    pub sxl: U<2>,
    pub uxl: U<2>,
    pub sd_rv32: bool,
    pub zero1: U<8>,
    pub tsr: bool,
    pub tw: bool,
    pub tvm: bool,
    pub mxr: bool,
    pub sum: bool,
    pub mprv: bool,
    pub xs: U<2>,
    pub fs: U<2>,
    pub mpp: U<2>,
    pub vs: U<2>,
    pub spp: U<1>,
    pub mpie: bool,
    pub ube: bool,
    pub spie: bool,
    pub upie: bool,
    pub mie: bool,
    pub hie: bool,
    pub sie: bool,
    pub uie: bool,
}

/// RoCC Instruction
///
/// <https://github.com/chipsalliance/rocket-chip/blob/master/src/main/scala/tile/LazyRoCC.scala#L18>
#[derive(Debug, Clone, Copy)]
pub struct RoCCInstruction {
    pub funct: Funct,
    pub rs2: U<5>,
    pub rs1: U<5>,
    pub xd: U<1>,
    pub xs1: U<1>,
    pub xs2: U<1>,
    pub rd: U<5>,
    pub opcode: U<7>,
}

/// RoCC Command
///
/// <https://github.com/chipsalliance/rocket-chip/blob/master/src/main/scala/tile/LazyRoCC.scala#L29>
/// TODO: Add fields which are inherent in `CoreBundle` in Chisel
#[derive(Debug, Clone, Copy)]
pub struct RoCCCommand<const X_LEN: usize> {
    pub inst: RoCCInstruction,
    pub rs1: U<X_LEN>,
    pub rs2: U<X_LEN>,
    pub status: MStatus,
}
