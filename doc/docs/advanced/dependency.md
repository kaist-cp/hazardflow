# Dependency Types

We defined the "dependency type" based on the dependency from the backward signal to the forward signal.
The concept was inspired from the [BaseJump STL](https://dl.acm.org/doi/pdf/10.1145/3195970.3199848).

It has two kinds of dependency types: `Helpful` and `Demanding`.

- `Helpful`: forward signals does not depend on the backward signals.
- `Demanding`: forward signals depends on the backward signals, and they satisfy the condition that if the payload is `Some`, the ready condition is true.

> Note that the dependency type does not consider the inter-interface backward dependency.
It is exposed in the IO types for `lfork` combinator.

It is defined as variants of an enum `Dep`:

```rust,noplayground
/// Dependency type of a hazard interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ConstParamTy)]
pub enum Dep {
    /// The payload (`Fwd`) does not depend on the resolver (`Bwd`).
    Helpful = 0,

    /// The payload (`Fwd`) depends on the resolver (`Bwd`), and they satisfy the condition that if the payload is
    /// `Some`, `Hazard::ready(p, r)` is true.
    ///
    /// It is a bug to make the payload depend on the resolver but break the condition.
    Demanding = 1,
}
```

We annotate the dependency type to the hazard interface.

```rust,noplayground
/// Hazard interface.
#[derive(Debug)]
pub struct I<H: Hazard, const D: Dep>;
```

The benefit of using dependency type is that it is useful to guarantee the transfer happens or not in the interface.
We will explain more with the example implementation of interface combinators.

## Examples: Types for Interface Combinators

The most common example of dependency type is the IO of interface combinators.

### `filter_map`

It takes an `I<VrH<P, R>, D>` and returns an `I<VrH<EP, R>, D>`.

### `reg_fwd`

It takes an `I<VrH<P, ()>, D>` and returns an `I<VrH<P, ()>, { Dep::Helpful }>`.

### `lfork`

It takes an `I<VrH<P, ()>, D>` and returns two `I<VrH<P, ()>, D>`.

### `join`

It takes `Vr<P1>`, `Vr<P2>` and returns an `Vr<(P1, P2)>`.
