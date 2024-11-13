//! LocalAddr.scala
//!
//! Reference: <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/LocalAddr.scala>

use std::ops::Add;

use crate::gemmini::*;

/// 32 bits garbage address that all bits are set to 1.
pub const GARBAGE_ADDR: usize = 0xFFFFFFFF;

// 6 + 14 + 1 + 11
const LOCAL_ADDR_BITS: usize = 32;

const SP_ADDR_BITS: usize = clog2(SP_BANKS * SP_BANK_ENTRIES);
const ACC_ADDR_BITS: usize = clog2(ACC_BANKS * ACC_BANK_ENTRIES);
const MAX_ADDR_BITS: usize = max(SP_ADDR_BITS, ACC_ADDR_BITS);

/// Number of bits to represent the bank of scratchpad.
pub const SP_BANK_BITS: usize = clog2(SP_BANKS);
const SP_BANK_ROW_BITS: usize = clog2(SP_BANK_ENTRIES);

/// Number of bits to represent the bank of accumulator.
pub const ACC_BANK_BITS: usize = clog2(ACC_BANKS);
/// Number of bits to represent the row of accumulator.
pub const ACC_BANK_ROW_BITS: usize = clog2(ACC_BANK_ENTRIES);

/// Number of rows in the scratchpad.
pub const SP_ROWS: usize = SP_BANKS * SP_BANK_ENTRIES;

/// <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/LocalAddr.scala#L26>
const METADATA_WIDTH: usize = 1 + 1 + 1 + clog2(8);

const GARBAGE_BITS: usize = if LOCAL_ADDR_BITS - MAX_ADDR_BITS >= METADATA_WIDTH + 1 { 1 } else { 0 };

/// Local address. The total number of bits for all fields is 32.
#[derive(Debug, Clone, Copy)]
pub struct LocalAddr {
    /// Is Accumulator Address?
    pub is_acc_addr: bool,
    /// Accumulate
    pub accumulate: bool,
    /// Read Full Accumulator Row
    pub read_full_acc_row: bool,
    /// NormCmd
    pub norm_cmd: U<3>,
    /// Garbage area.
    pub garbage: U<{ LOCAL_ADDR_BITS - MAX_ADDR_BITS - METADATA_WIDTH - 1 }>,
    /// Is garbage address.
    pub is_garbage: bool,
    /// Address Data
    pub data: U<MAX_ADDR_BITS>,
}

impl LocalAddr {
    /// <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/LocalAddr.scala#L105>
    pub fn cast_to_local_addr(value: U<64>) -> Self {
        let result = Self::from(value);
        Self { is_garbage: false, ..result }
    }

    /// <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/LocalAddr.scala#L113>
    pub fn cast_to_sp_addr(value: U<64>) -> Self {
        let result = Self::cast_to_local_addr(value);
        Self { is_acc_addr: false, accumulate: false, read_full_acc_row: false, ..result }
    }

    /// Returns the garbage address
    pub fn garbage() -> Self {
        Self::from(GARBAGE_ADDR.into_u())
    }

    /// <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/LocalAddr.scala#L33>.
    pub fn sp_bank(self) -> U<SP_BANK_BITS> {
        if SP_ADDR_BITS == SP_BANK_ROW_BITS {
            0.into_u()
        } else {
            self.data.clip_const::<{ SP_ADDR_BITS - SP_BANK_ROW_BITS }>(SP_BANK_ROW_BITS)
        }
    }

    /// <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/LocalAddr.scala#L34>.
    pub fn sp_row(self) -> U<SP_BANK_ROW_BITS> {
        self.data.clip_const::<SP_BANK_ROW_BITS>(0)
    }

    /// <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/LocalAddr.scala#L35>.
    pub fn acc_bank(self) -> U<ACC_BANK_BITS> {
        if ACC_ADDR_BITS == ACC_BANK_ROW_BITS {
            0.into_u()
        } else {
            self.data.clip_const::<{ ACC_ADDR_BITS - ACC_BANK_ROW_BITS }>(ACC_BANK_ROW_BITS)
        }
    }

    /// <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/LocalAddr.scala#L36>.
    pub fn acc_row(self) -> U<ACC_BANK_ROW_BITS> {
        self.data.clip_const::<ACC_BANK_ROW_BITS>(0)
    }

    /// Returns scratchpad address.
    pub fn full_sp_addr(self) -> U<SP_ADDR_BITS> {
        self.data.clip_const::<SP_ADDR_BITS>(0)
    }

    /// Returns accumulator address.
    pub fn full_acc_addr(self) -> U<ACC_ADDR_BITS> {
        self.data.clip_const::<ACC_ADDR_BITS>(0)
    }

    /// <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/LocalAddr.scala#L43>
    pub fn is_garbage(self) -> bool {
        self.is_acc_addr
            && self.accumulate
            && self.read_full_acc_row
            && self.data.all(|v| v)
            && if GARBAGE_BITS > 0 { self.is_garbage } else { true }
    }

    /// <https://github.com/ucb-bar/gemmini/blob/master/src/main/scala/gemmini/LocalAddr.scala#L41-L42>
    pub fn is_same_addr(self, other: Self) -> bool {
        (self.is_acc_addr == other.is_acc_addr) && (self.data == other.data)
    }

    /// Make garbage LocalAddr. All bits are set to 1.
    pub fn make_this_garbage(self) -> Self {
        Self {
            is_acc_addr: true,
            accumulate: true,
            read_full_acc_row: true,
            norm_cmd: true.repeat::<3>(),
            garbage: true.repeat::<{ LOCAL_ADDR_BITS - MAX_ADDR_BITS - METADATA_WIDTH - 1 }>(),
            is_garbage: true,
            data: true.repeat::<MAX_ADDR_BITS>(),
        }
    }

    /// Returns whether `self` is less than `other`.
    // TODO: Implement this with rust std trait.
    pub fn lt(self, other: LocalAddr) -> bool {
        self.is_acc_addr == other.is_acc_addr
            && if self.is_acc_addr {
                self.full_acc_addr() < other.full_acc_addr()
            } else {
                self.full_sp_addr() < other.full_sp_addr()
            }
    }

    /// Return whether `self` is less or equal than `other`.
    // TODO: Implement this with rust std trait.
    pub fn le(self, other: LocalAddr) -> bool {
        self.is_acc_addr == other.is_acc_addr
            && if self.is_acc_addr {
                self.full_acc_addr() <= other.full_acc_addr()
            } else {
                self.full_sp_addr() <= other.full_sp_addr()
            }
    }

    /// Adds `self` and `other` and also returns overflow has occurred or not.
    pub fn add_with_overflow(self, other: U<MAX_ADDR_BITS>) -> (LocalAddr, bool) {
        let data = self.data + other;
        let overflow = if self.is_acc_addr { data[ACC_ADDR_BITS] } else { data[SP_ADDR_BITS] };

        (LocalAddr { data: data.resize(), ..self }, overflow)
    }

    /// Subs `self` and `other` and returns both new address and underflow.
    pub fn floor_sub(self, other: U<MAX_ADDR_BITS>, floor: U<MAX_ADDR_BITS>) -> (LocalAddr, bool) {
        let underflow = self.data.resize() < (floor + other);
        let data = if underflow { floor } else { self.data - other };

        (LocalAddr { data, ..self }, underflow)
    }
}

impl Add<U<MAX_ADDR_BITS>> for LocalAddr {
    type Output = LocalAddr;

    fn add(self, rhs: U<MAX_ADDR_BITS>) -> Self::Output {
        LocalAddr { data: (self.data + rhs).resize(), ..self }
    }
}

impl From<U<64>> for LocalAddr {
    /// ### Reterive 32 bits address.
    /// - `let addr: U<32> = value[31:0]`
    /// - `value[63:32]` means the number of rows and columns.
    ///
    /// ### Address scheme.
    /// - is_acc_addr: `addr[31]`
    /// - accumulate: `addr[30]`
    /// - read_full_acc_row: `addr[29]`
    /// - norm_cmd: `addr[28:26]`
    /// - garbage: `addr[25:15]`
    /// - is_garbage: `addr[14]`
    /// - data: `addr[13:0]`
    fn from(value: U<64>) -> Self {
        let addr = value.clip_const::<32>(0);

        Self {
            is_acc_addr: addr[31],
            accumulate: addr[30],
            read_full_acc_row: addr[29],
            norm_cmd: addr.clip_const::<3>(26),
            garbage: addr.clip_const::<{ LOCAL_ADDR_BITS - MAX_ADDR_BITS - METADATA_WIDTH - 1 }>(15),
            is_garbage: addr[14],
            data: addr.clip_const::<MAX_ADDR_BITS>(0),
        }
    }
}

impl From<HOption<U<64>>> for LocalAddr {
    fn from(value: HOption<U<64>>) -> Self {
        match value {
            Some(v) => LocalAddr::from(v),
            None => LocalAddr::garbage(),
        }
    }
}

impl From<LocalAddr> for U<32> {
    fn from(value: LocalAddr) -> Self {
        let u = 0.into_u::<32>();
        let u = u.set(31, value.is_acc_addr);
        let u = u.set(30, value.accumulate);
        let u = u.set(29, value.read_full_acc_row);
        let u = u.set_range(26, value.norm_cmd);
        let u = u.set_range(15, value.garbage);
        let u = u.set(14, value.is_garbage);
        u.set_range(0, value.data)
    }
}
