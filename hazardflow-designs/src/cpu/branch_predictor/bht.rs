//! Branch history table.

use crate::std::*;

/// 2-bit saturation counter.
#[derive(Debug, Default, Clone, Copy)]
pub enum SatCounter {
    /// Strongly not taken.
    StronglyNotTaken,

    /// Weakly not taken.
    #[default]
    WeaklyNotTaken,

    /// Weakly taken.
    WeaklyTaken,

    /// Strongly taken.
    StronglyTaken,
}

impl SatCounter {
    /// Increments the counter.
    pub fn increment(self) -> Self {
        match self {
            SatCounter::StronglyNotTaken => SatCounter::WeaklyNotTaken,
            SatCounter::WeaklyNotTaken => SatCounter::WeaklyTaken,
            SatCounter::WeaklyTaken => SatCounter::StronglyTaken,
            SatCounter::StronglyTaken => SatCounter::StronglyTaken,
        }
    }

    /// Decrements the counter.
    pub fn decrement(self) -> Self {
        match self {
            SatCounter::StronglyNotTaken => SatCounter::StronglyNotTaken,
            SatCounter::WeaklyNotTaken => SatCounter::StronglyNotTaken,
            SatCounter::WeaklyTaken => SatCounter::WeaklyNotTaken,
            SatCounter::StronglyTaken => SatCounter::WeaklyTaken,
        }
    }

    /// Predicts the branch is taken or not.
    pub fn predict(self) -> bool {
        match self {
            SatCounter::StronglyNotTaken | SatCounter::WeaklyNotTaken => false,
            SatCounter::WeaklyTaken | SatCounter::StronglyTaken => true,
        }
    }
}

/// BHT.
#[derive(Debug, Default, Clone, Copy)]
pub struct Bht<const N: usize>
where [(); clog2(N)]:
{
    /// Entries.
    pub entries: Array<SatCounter, N>,
}

impl<const N: usize> Bht<N>
where [(); clog2(N)]:
{
    /// Predicts the branch is taken or not based on the given PC.
    pub fn predict(self, _pc: U<32>) -> bool {
        todo!("Assignment 2")
    }

    /// Updates as the branch was mispredicted.
    pub fn update(self, _pc: U<32>) -> Self {
        todo!("Assignment 2")
    }
}
