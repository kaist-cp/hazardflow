# Modules

<!-- Modules help structure the design of complex hardware systems, like processors, by breaking them into smaller, manageable, and reusable components. -->

We consider a module as a function takes the ingress interface and returns the egress interface.

```rust,noplayground
m: impl FnOnce(I) -> E
```

We can construct a module as a class of interface combinators. Please refer to the [Interface Combinators](./combinator.md) for more information.

## Combine Black Box Module to Interface

The `comb` method within the interface trait is used to combine the black box module to the given interface and return the egress interface.
```rust,noplayground
fn comb<E: Interface>(self, m: impl FnOnce(Self) -> E) -> E {
    m(self)
}
```

- Applying the given interface to the module is essentially applying the module function `m` to the ingress interface.
- It is useful when we want to combine multiple modules together.

For example, we can combine multiple stage modules with `comb` in the CPU core.

```rust,noplayground
pub fn core(
    imem: impl FnOnce(Vr<MemReq>) -> Vr<MemRespWithAddr>,
    dmem: impl FnOnce(Vr<MemReq>) -> Vr<MemRespWithAddr>,
) {
    fetch::<START_ADDR>(imem)
        .comb(decode)
        .comb(exe)
        .comb(move |ingress| mem(ingress, dmem))
        .comb(wb)
}
```

- `imem` and `dmem` are modules for instruction memory and data memory, respectively.
- We chain the 5 sub-modules `fetch`, `decode`, `exe`, `mem`, and `wb` by using the `comb` method.

## Module Combinators

We provide some handy module combinators that manipulates modules.

### `from_fn`

TODO(@minseong)

### `seq`

TODO(@minseong)

### `flip`

TODO(@minseong)

<!--TODO: should we introduce the following function?-->
<!---->
<!--/// Generate an array of modules.-->
<!--/// TODO: Modify `f` to be `f: impl FnOnce(n: usize) -> T`.-->
<!--#[magic(module::from_fn)]-->
<!--pub fn from_fn<I: Interface, O: Interface, J: Interface, T, const N: usize>(f: T) -> [fn(I, J) -> (O, J); N]-->
<!--where T: FnOnce(I, J) -> (O, J) {-->
<!--    todo!()-->
<!--}-->
<!---->
<!--/// Generate a 1D systolic array from an array of modules.-->
<!--#[magic(module::seq)]-->
<!--pub fn seq<I: Interface, O: Interface, J: Interface, const N: usize>(-->
<!--    ms: [fn(I, J) -> (O, J); N],-->
<!--) -> impl FnOnce([I; N], J) -> ([O; N], J) {-->
<!--    // This should be primitive?-->
<!--    |is, j| todo!()-->
<!--}-->
<!---->
<!--/// Flip module input and output.-->
<!--pub fn flip<I1: Interface, I2: Interface, O1: Interface, O2: Interface, T>(f: T) -> impl FnOnce(I2, I1) -> (O2, O1)-->
<!--where T: FnOnce(I1, I2) -> (O1, O2) {-->
<!--    move |i2, i1| {-->
<!--        let (o1, o2) = f(i1, i2);-->
<!--        (o2, o1)-->
<!--    }-->
<!--}-->
