//! DMA related modules

pub mod dma_command_tracker;

use crate::prelude::*;
use crate::std::*;

/// DMA Read Response
/// This struct is used in `load` module
#[derive(Debug, Clone, Copy)]
pub struct RequestReturned<const NCMDS: usize, const MAX_BYTES: usize>
where
    [(); clog2(NCMDS)]:,
    [(); clog2(MAX_BYTES + 1)]:,
{
    /// Number of bytes to read.
    pub bytes_read: U<{ clog2(MAX_BYTES + 1) }>,
    /// Command ID.
    pub cmd_id: U<{ clog2(NCMDS) }>,
}
