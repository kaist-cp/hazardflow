//! DMA command tracker.

use super::*;
use crate::prelude::*;
use crate::std::hazard::*;
use crate::std::*;

/// Command allocation request.
#[derive(Debug, Clone, Copy)]
pub struct AllocReq<T: Copy, const MAX_BYTES: usize>
where [(); clog2(MAX_BYTES + 1)]:
{
    /// Tag.
    pub tag: T,
    /// Bytes to read.
    pub bytes_to_read: U<{ clog2(MAX_BYTES + 1) }>, // U<11>, U<15>
}

/// Command allocation response.
#[derive(Debug, Clone, Copy)]
pub struct AllocResp<const NCMDS: usize>
where [(); clog2(NCMDS)]:
{
    /// Command ID.
    pub cmd_id: U<{ clog2(NCMDS) }>,
}

/// Command completion response.
#[derive(Debug, Clone, Copy)]
pub struct CmdCompletionResp<T: Copy> {
    /// Tag.
    pub tag: T,
}

#[derive(Debug, Clone, Copy)]
struct CmdEntry<T: Copy, const MAX_BYTES: usize>
where [(); clog2(MAX_BYTES + 1)]:
{
    tag: T,
    bytes_left: U<{ clog2(MAX_BYTES + 1) }>,
}

/// DMA Command Tracker.
///
/// # Generics
///
/// - `T`: Tag type.
/// - `NCMDS`: Maximum number of commands in the table.
/// - `MAX_BYTES`: Maximum number of bytes for each command.
pub fn dma_command_tracker<T: Copy, const NCMDS: usize, const MAX_BYTES: usize>(
    alloc: Vr<AllocReq<T, MAX_BYTES>>,
    request_returned: Valid<RequestReturned<NCMDS, MAX_BYTES>>,
) -> (Valid<AllocResp<NCMDS>>, Vr<CmdCompletionResp<T>>)
where
    [(); clog2(NCMDS)]:,
    [(); clog2(MAX_BYTES + 1)]:,
{
    unsafe {
        (alloc, request_returned)
            .fsm::<(Valid<AllocResp<NCMDS>>, Vr<CmdCompletionResp<T>>), Array<HOption<CmdEntry<T, MAX_BYTES>>, NCMDS>>(
                None.repeat(),
                |(alloc, req_ret), er, s| {
                    let empty_id = s.find_idx(|e| e.is_none());
                    let completed_cmd = s
                        .find_idx(|e| e.is_some_and(|cmd| cmd.bytes_left == 0.into_u()))
                        .map(|id| (id, s[id].unwrap().tag));

                    let ir = (Ready::new(empty_id.is_some(), ()), ());

                    // Update for allocation.
                    let s_next = alloc.zip(empty_id).map_or(s, |(alloc, empty_id)| {
                        let e_next = Some(CmdEntry { tag: alloc.tag, bytes_left: alloc.bytes_to_read });
                        s.set(empty_id, e_next)
                    });

                    // Update for read request return.
                    let s_next = req_ret.map_or(s_next, |req_ret| {
                        let e_next = s_next[req_ret.cmd_id]
                            .map(|cmd| CmdEntry { bytes_left: cmd.bytes_left - req_ret.bytes_read, ..cmd });
                        s_next.set(req_ret.cmd_id, e_next)
                    });

                    // Update for completion.
                    let s_next = completed_cmd.filter(|_| er.1.ready).map_or(s_next, |(id, _)| s_next.set(id, None));

                    let ep = (
                        empty_id.map(|id| AllocResp { cmd_id: id }),
                        completed_cmd.map(|(_, tag)| CmdCompletionResp { tag }),
                    );

                    (ep, ir, s_next)
                },
            )
    }
}

/// DMA Command Tracker with default configuration.
/// Used for debugging.
#[synthesize]
pub fn dma_command_tracker_default(
    alloc: Vr<AllocReq<U<6>, 1024>>,
    request_returned: Valid<RequestReturned<2, 1024>>,
) -> (Valid<AllocResp<2>>, Vr<CmdCompletionResp<U<6>>>) {
    dma_command_tracker::<U<6>, 2, 1024>(alloc, request_returned)
}
