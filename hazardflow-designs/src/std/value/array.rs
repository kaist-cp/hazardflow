//! Array.

use core::ops::*;

use hazardflow_macro::magic;

use super::*;
use crate::std::clog2;

/// An array of signals.
#[derive(Debug, Clone, Copy)]
#[magic(array::array)]
pub struct Array<V: Copy, const N: usize> {
    _marker: core::marker::PhantomData<V>,
}

impl<V: Copy + Default, const N: usize> Default for Array<V, N> {
    fn default() -> Self {
        V::default().repeat()
    }
}
impl<V: Copy + Default, const N: usize> Array<V, N> {
    /// Folds the array into a single value.
    ///
    /// The fold order is not guaranteed, so the operation `f` must be associative.
    // TODO: Currently this is just an alias of `fold` with default value. Implement this magic when needed.
    // TODO: When implementing this magic, make sure to check the constraints below.
    //
    // Tree fold
    //
    // This operation folds an array with 2^K elements by constructing a fold tree(with height K) as below:
    // ```text
    //   O   O    ...  O   O
    //    \ / (op)      \ / (op)
    //     O     ...     O
    //
    //           ...
    //
    //           \/
    //           O
    // ````
    //
    // This operation can generated better verilog, but need to be used carefully
    //
    // 1. Associativity of the operation
    //
    // Unlike the `Array::fold`, which is foldleft, the order of operation will rearranged
    // arbitrarily. So if the operation is not associative, the result might be different from
    // expected.
    //
    // 2. Number of elements
    //
    // In order to construct the fold tree in a readable way in verilog (which is nested for loop),
    // we only allow use of this api only when length is power of 2 (ex. 1, 2, 4, 8, ...).
    // You should manually resize to use this api for arrays that does not satisfy the constraint
    //
    // #[magic(array::tree_fold)]
    pub fn fold_assoc<F: FnOnce(V, V) -> V>(self, f: F) -> V {
        self.fold(V::default(), f)
    }

    /// Finds the index of the first element that satisfies the given condition.
    pub fn find_idx(self, f: impl Fn(V) -> bool) -> HOption<U<{ clog2(N) }>> {
        self.enumerate().map(|(idx, elt)| if f(elt) { Some(idx) } else { None }).fold_assoc(|lhs, rhs| lhs.or(rhs))
    }
}

impl<V: Copy, const N: usize> Array<V, N> {
    /// Returns a new array with the `idx`-th element set to `elt`.
    #[magic(array::set)]
    pub fn set<Idx: Into<U<{ clog2(N) }>>>(self, _idx: Idx, _elt: V) -> Array<V, N> {
        todo!()
    }

    /// Returns a new array with the `idx`-th element set to `elt` if `cond` is true.
    pub fn set_cond(self, cond: bool, idx: U<{ clog2(N) }>, elt: V) -> Array<V, N> {
        if cond {
            self.set(idx, elt)
        } else {
            self
        }
    }

    /// Returns a new clipped array of size `M` starting from `index`.
    #[magic(array::clip_const)]
    pub fn clip_const<const M: usize>(self, _index: usize) -> Array<V, M> {
        todo!();
    }

    /// Returns a new array that has tuples from the two given arrays as elements.
    #[magic(array::zip)]
    pub fn zip<W: Copy>(self, _other: Array<W, N>) -> Array<(V, W), N> {
        todo!()
    }

    /// Returns a new array whose elements are enumerated with their indices.
    pub fn enumerate(self) -> Array<(U<{ clog2(N) }>, V), N> {
        range::<N>().zip(self)
    }

    /// Transforms elements of `self` using `f`.
    #[magic(array::map)]
    pub fn map<W: Copy, F: FnOnce(V) -> W>(self, _f: F) -> Array<W, N> {
        todo!()
    }

    /// Folds the array into a single value.
    ///
    /// The fold order is from left to right. (i.e. `foldl`)
    #[magic(array::fold)]
    pub fn fold<B: Copy, F: FnOnce(B, V) -> B>(self, _init: B, _f: F) -> B {
        todo!()
    }

    /// Tests if any element matches a predicate.
    // TODO: Use tree fold?
    pub fn any<F: Fn(V) -> bool>(self, f: F) -> bool {
        self.fold(false, |acc, elt| acc | f(elt))
    }

    /// Tests if every element matches a predicate.
    /// TODO: Use tree fold?
    pub fn all<F: Fn(V) -> bool>(self, f: F) -> bool {
        self.fold(true, |acc, elt| acc & f(elt))
    }

    /// Resizes the given array.
    #[magic(array::resize)]
    pub fn resize<const M: usize>(self) -> Array<V, M> {
        todo!()
    }

    /// Chunks the array into an array of arrays.
    #[magic(array::chunk)]
    pub fn chunk<const M: usize>(self) -> Array<Array<V, M>, { N / M }> {
        todo!();
    }

    /// Returns a new array with the two given arrays appended.
    #[magic(array::append)]
    pub fn append<const M: usize>(self, _other: Array<V, M>) -> Array<V, { N + M }> {
        todo!();
    }

    /// Returns a new array with the `M` elements starting from `index` set to the elements of `other`.
    #[magic(array::set_range)]
    pub fn set_range<const M: usize>(self, _index: usize, _other: Array<V, M>) -> Array<V, N> {
        todo!();
    }

    /// Returns a Cartesian product of the two arrays.
    pub fn cartesian_product<W: Copy, const M: usize>(self, other: Array<W, M>) -> Array<(V, W), { N * M }> {
        self.map(|self_elt| other.map(|other_elt| (self_elt, other_elt))).concat()
    }

    /// Reverses the array.
    pub fn reverse(self) -> Array<V, N>
    where [(); clog2(N)]: {
        range::<N>().map(|idx| self[U::from(N - 1) - idx])
    }
}

impl<V: Copy, const N: usize, const M: usize> Array<Array<V, N>, M> {
    /// Concatenates the array of arrays into a 1D array.
    #[magic(array::concat)]
    pub fn concat(self) -> Array<V, { M * N }> {
        todo!();
    }
}

/// Returns an array containing `0..N`.
// TODO: make it into macro
// TODO: allow different starting point (FROM..START)
#[magic(array::range)]
pub fn range<const N: usize>() -> Array<U<{ clog2(N) }>, N> {
    todo!("compiler magic")
}

impl<V: Copy, const N: usize> From<[V; N]> for Array<V, N> {
    #[magic(array::from)]
    fn from(_value: [V; N]) -> Self {
        todo!();
    }
}

impl<V: Copy, const N: usize, const M: usize> Index<U<N>> for Array<V, M> {
    type Output = V;

    #[magic(array::index)]
    fn index(&self, _idx: U<N>) -> &V {
        todo!()
    }
}

impl<V: Copy, const N: usize> PartialEq for Array<V, N> {
    #[magic(array::eq)]
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }

    #[allow(clippy::partialeq_ne_impl)]
    #[magic(array::ne)]
    fn ne(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl<V: Copy, const M: usize> Index<usize> for Array<V, M> {
    type Output = V;

    #[magic(array::index)]
    fn index(&self, _idx: usize) -> &V {
        todo!()
    }
}

impl<V: Copy, const N: usize> BitOr for Array<V, N> {
    type Output = Self;

    #[magic(array::bitor)]
    fn bitor(self, _rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<V: Copy, const N: usize> BitAnd for Array<V, N> {
    type Output = Self;

    #[magic(array::bitand)]
    fn bitand(self, _rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<V: Copy, const N: usize> BitXor for Array<V, N> {
    type Output = Self;

    #[magic(array::bitxor)]
    fn bitxor(self, _rhs: Self) -> Self {
        todo!();
    }
}

/// Repeat.
pub trait RepeatExt: Copy {
    /// Returns an array with the given value repeated `N` times.
    fn repeat<const N: usize>(self) -> Array<Self, N>;
}

impl<T: Copy> RepeatExt for T {
    #[magic(array::repeat)]
    fn repeat<const N: usize>(self) -> Array<Self, N> {
        todo!()
    }
}
