//! Branch target buffer.

use crate::std::*;

/// BTB.
#[derive(Debug, Default, Clone, Copy)]
pub struct Btb<const N: usize>
where [(); clog2(N)]:
{
    /// Entries.
    pub entries: Array<HOption<u32>, N>,
}

impl<const N: usize> Btb<N>
where [(); clog2(N)]:
{
    /// Predicts the target address based on the given PC.
    pub fn predict(self, _pc: U<32>) -> HOption<u32> {
        todo!("Assignment 2")
    }

    /// Updates as the target address was mispredicted.
    pub fn update(self, _pc: U<32>, _target: u32) -> Self {
        todo!("Assignment 2")
    }
}
