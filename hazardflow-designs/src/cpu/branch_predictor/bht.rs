//! Branch history table.

use super::*;
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
pub struct Bht {
    /// BHT entries.
    #[allow(unused)]
    entries: Array<SatCounter, BHT_ENTRIES>,
}

impl Bht {
    /// Predicts the direction of a branch instruction with the given PC.
    ///
    /// Returns `true` if the branch is prediction as taken; otherwise, returns `false`.
    pub fn predict(self, _pc: u32) -> bool {
        todo!("assignment 2")
    }

    /// Returns the updated BHT when a branch misprediction occurs at the given PC.
    ///
    /// It updates the entry corresponding to the given PC.
    pub fn update(self, _pc: u32) -> Self {
        todo!("assignment 2")
    }
}
