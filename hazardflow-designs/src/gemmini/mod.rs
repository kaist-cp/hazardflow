//! Gemmini

use crate::prelude::*;
use crate::std::*;

pub mod arithmetic;
pub mod configs;
pub mod execute;
pub mod ffis;
pub mod isa;
pub mod load;
pub mod local_addr;
pub mod reservation_station;
pub mod sram;
pub mod store;

use arithmetic::*;
use configs::*;
use execute::*;
use ffis::*;
use isa::*;
use load::*;
use reservation_station::*;
use sram::*;
use store::*;

/// Set of `Reservation Station`, `Load`, `Execute`, `Store`, `Scratchpad` modules
/// TODO: Handle TLB
pub fn gemmini_core(
    cmd: Vr<GemminiCmd>,
    _tlb_accessor: impl FnOnce([Vr<TlbResp>; 2]) -> [Valid<TlbReq>; 2],
) -> RsCompleted {
    // Split SRAM
    let (dma, exe) = module_split(sram);
    let (dma_read, dma_write) = module_split(|i1, i2| dma((i1, i2)));
    let (spad, acc) = module_split(|i1, i2| exe((i1, i2)));
    let (spad_read, spad_write) = module_split(|i1, i2| spad((i1, i2)));
    let (acc_read, acc_write) = module_split(|i1, i2| acc((i1, i2)));

    // Split reservation station
    let (rs_alloc, rs_get_completed_id) = module_split(|i1, i2| (reservation_station(i1, i2), ()));
    let (RsIssues { ld: ld_cmd, ex: ex_cmd, st: st_cmd }, rs_completed, _rs_busy) = rs_alloc(cmd);

    // Load controller. TODO: Do not use magic number
    let load_completed_id = ld_cmd
        .map(|issued| GemminiCmd { rob_id: Some(issued.rob_id), ..issued.cmd })
        .comb(move |cmd| load::<256, 32768>(cmd, dma_read));

    // Execute module. TODO: Do not use magic number
    let exe_completed_id = ex_cmd
        .map(|issued| GemminiCmd { rob_id: Some(issued.rob_id), ..issued.cmd })
        .comb(move |cmd| execute::<1, 16, 1, 16, 2>(cmd, spad_read, spad_write, acc_read, acc_write));

    // Store controller. TODO: Do not use magic number
    let store_completed_id = st_cmd
        .map(|issued| GemminiCmd { rob_id: Some(issued.rob_id), ..issued.cmd })
        .comb(move |cmd| store::<256, 32768>(cmd, dma_write));

    // Loop back the completed id to the reservation station
    [exe_completed_id.discard_into_vr(), load_completed_id, store_completed_id]
        .merge()
        .always_into_valid()
        .into_helpful()
        .comb(rs_get_completed_id);

    rs_completed
}
