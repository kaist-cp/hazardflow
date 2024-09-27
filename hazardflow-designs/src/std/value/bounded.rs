//! Bounded.

use super::*;
use crate::std::*;

/// A bounded unsigned integer in `0..MAX` with bitwidth `WIDTH`.
#[derive(Debug, Clone, Copy, Default)]
pub struct BoundedU<const MAX: usize, const WIDTH: usize = { clog2(MAX) }> {
    /// Value
    value: U<WIDTH>,
}

impl<const MAX: usize, const WIDTH: usize> BoundedU<MAX, WIDTH> {
    /// Creates a new bounded unsigned integer.
    pub fn new(value: U<WIDTH>) -> Self {
        Self { value }
    }

    /// Increments the value.
    pub fn incr(self) -> Self
    where [(); WIDTH + 1]: {
        let incr = (self.value + 1.into_u()).resize();
        if incr == MAX.into_u() {
            Self::default()
        } else {
            BoundedU { value: incr }
        }
    }

    /// Returns the value.
    pub fn value(self) -> U<WIDTH> {
        self.value
    }
}

/// Returns (`a` + `b`) mod `max`.
///
/// When using this method, make sure that max(`a`, `b`) < `max` and `max` <= 2^`N`.
pub fn wrapping_add<const N: usize>(a: U<N>, b: U<N>, max: U<{ N + 1 }>) -> U<N> {
    let out = if a.resize::<{ N + 1 }>() >= max - b.resize::<{ N + 1 }>() { a + b - max } else { a + b };
    out.resize::<N>()
}

/// Increases `value` in range \[0, `max` - 1].
///
/// When using this method, make sure that `max` <= 2^`N`.
pub fn wrapping_inc<const N: usize>(value: U<N>, max: U<{ N + 1 }>) -> U<N> {
    wrapping_add::<N>(value, U::from(1), max)
}

/// Returns `a.trunk_add(b)` if `a + b < max`, 0 otherwise.
pub fn floor_add<const N: usize>(a: U<N>, b: U<N>, max: U<{ N + 1 }>) -> U<N> {
    if a + b >= max {
        0.into_u()
    } else {
        a.trunk_add(b)
    }
}
