# Signal

Signal is a collection of types that can be transferred through wires in HazardFlow HDL. We divide these types into two categories: **scalar** and **compound**.
The HazardFlow HDL reuse some data types from the Rust programming language.
Normally we can consider a data type implements [`Copy` trait in Rust](https://doc.rust-lang.org/std/marker/trait.Copy.html) as a signal type in the HazardFlow HDL.

> The `Copy` trait in Rust and the signal type in HazardFlow are not in a perfect 1:1 relationship.
While the signal type in HazardFlow must implement Rust's `Copy` trait, there are some types like function pointers (e.g., `fn() -> i32`) that implement the `Copy` trait but are not treated as signal types in HazardFlow.

For more information of the Rust data types, please refer to [The Rust Programming Language Book](https://doc.rust-lang.org/beta/book/ch03-02-data-types.html).

## Scalar Types

A scalar type represents a single value.

### Boolean

We interpret the Boolean type the same as the `bool` type in the Rust programming language. In HazardFlow HDL, we also interpret the Boolean value as `0` or `1` and can be sent through the wires when the value is `True` or `False`. 

### Unsigned Integer

In the HazardFlow HDL, we support arbitrary bit-width unsigned integers. We define it as `U<N>`, where `N` is the bit-width. We provide a handful of primitive functions for supporting unsigned integers' arithmetic operations, type conversion, and also ordering between different unsigned integers.
* Arithmetic operations
  * Basic arithmetic operations `+`, `-`, `*` are supported, and logical right shift and logical left shift are supported. 
  * Some of the arithmetic operations might lead to bit-width changes in the final result.
* Type conversion
  * We support converting Rust `i32`, `u32`, `u8`, `usize`, `u128`, and `bool` into `U<N>`.
  * We can convert `U<N>` into `u32`, `u8`, and also `bool`.
* Ordering
  * We provide ordering functions for developers to easily compare two unsigned integers.

## Compound Types

Compound types can group multiple values into one type.

### Enum

We interpret the `enum` type in HazardFlow HDL the same as Rust, the pattern matching feature is supported. The `enum` type gives you a way of saying a value is one of a possible set of values.

#### Example: HOption

Similar to the `Option` type in Rust, the `HOption` type in HazardFlow HDL is also an `enum` type. We define `HOption` type as:

```rust,noplayground
#[derive(Debug, Clone, Copy, HEq)]
enum HOption<T: Copy> {
    /// No value.
    None,
    /// Some value of type `T`.
    Some(T),
}
```

We provide almost the same handy functions as Rust when we operate on the `HOption` type.
For example, the `map` function in the `HOption` type applies a function `f` on the inner type of `HOption` if the value is `Some(T)` else returns `None`.
`and_then` function often used to chain fallible operations that may return `None` and it flattens the result, avoiding nested `HOption`.

### Tuple

The tuple type is the same as the tuple type in Rust, and as long as the types within a tuple are signals, then we can treat the tuple type as a signal type that can be sent through the wires too. For example, `(U<32>, bool, HOption<U<8>>)` can be considered as a signal type.

### Struct

Similar to the tuple type, if every field of a `struct` is a signal type, we can consider the `struct` as a signal type.
For example, the AXI stream can be constructued as follows:

```rust,noplayground
#[derive(Debug, Clone, Copy)]
struct Axis {
    data: U<512>, // TDATA signal
    keep: U<64>,  // TKEEP signal
    last: bool,   // TLAST signal
    user: bool,   // TUSER signal
}
```

### Array

The `Array` type is primitive in the HazardFlow HDL. We can define an `N` size sequence of `V` type data as `Array<V, N>`. The `Array` type comes with a handful of handy functions, including indexing, clipping an array, zipping two arrays together, etc.

#### Example: Unsigned Integer

In HazardFlow HDL, we represent unsigned integer as an `Array` of `bool` with bit-width `N`. When `bool` is `true`, we interpret it as a bit with value `1`, `false` as `0`.

```rust,noplayground
type U<const N: usize> = Array<bool, N>;
```

<!--TODO: We might need this for doc.rs-->
<!--We provide a handful of primitive functions for supporting unsigned integer's arithmetic operations, converting different bit-width integers into `U<N>`, converting `U<N>` into different bit-width integers, and also ordering between different unsigned integers.-->
<!--* Arithmetic operations-->
<!--  * The `add` operation adds up two unsigned integers. -->
<!--    * We define it as `fn add(self, _rhs: U<N>) -> U<{ N + 1 }`. `N` is the bit-width. -->
<!--    * The sum of two N bits unsigned integer could end up with `N + 1` bits. -->
<!--    * If you want to truncate the result into `N` bits, consider using `trunk_add`.-->
<!--  * The `sub` operation subtracts two unsigned integers.-->
<!--    * We define it as `fn sub(self, _other: U<N>) -> U<N>`. Subtracting two `N` bits unsigned integers always gets a `N` bit-width result.-->
<!--  * The `mul` operation multiplies two unsigned integers.-->
<!--    * We define it as `fn mul(self, _other: U<M>) -> U<{ N + M }>`.-->
<!--    * The result of the multiplication could end up with `N + M` bit-width.-->
<!--  * The `shr` and the `shl` operations shift the unsigned integer to the right and left respectively.-->
<!--    * We define them as `fn shr(self, _rhs: usize) -> U<N>` and `fn shl(self, _lhs: usize) -> U<N>`. This will keep the same bit-width as the original unsigned integer.-->
<!--    * Note that if we cast a signed integer into an unsigned integer in the HazardFlow HDL and operate shift operations, we lose the sign information of the original signed integer since the shift operation is logical.-->
<!--* Type conversion-->
<!--  * We support converting `i32`, `u32`, `u8`, `usize`, `u128`, and `bool` into `U<N>`, `N` is the bit-width also the length of the Boolean array.-->
<!--  * Note that `bool` can be converted to `U<1>`, since we interpret unsigned integer type as `Array<bool, N>` in the HazardFlow HDL.-->
<!--  * We can convert `U<N>` into `u32`, `u8`.-->
<!--* Ordering-->
<!--  * We provide ordering functions for developers to easily compare two unsigned integers. These functions are less than `lt`, less or equal `le`, greater then `gt`, and greater or equal `ge`.-->

