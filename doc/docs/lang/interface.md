# Interface

Interface is a communication protocol between modules.
HazardFlow HDL provides a **Hazard** that abstracts communication protocol with arbitrary transfer conditions (e.g., valid-ready protocol).
Like signals, interfaces also support compound types such as tuples and structs.

<!-- Hazard and interface are the most fundamental concepts of the HazardFlow HDL.
We define a `struct` implementing the `Interface` trait and containing the `Hazard` as a **Hazard Interface**, the building block for describing hardware behavior. -->

## Hazard

### Handshake

In hardware semantics, a communication protocol is described as a handshake mechanism.

As an example, the most commonly used valid-ready protocol is described as below:

<p align="center">
  <img src="../figure/handshake.drawio.svg" />
</p>

* The sender computes `Valid` signal and `Payload` signal each clock cycle.
* The receiver computes `Ready` signal each clock cycle.
* The `Valid` and `Ready` signals are shared between the sender and the receiver.
* When both `Valid` and `Ready` signals are `true`, we define it as **Transfer happens**.
* Note that the `Payload` is always flowing through the wires.

Wave form diagram:

<p align="center">
  <img src="../figure/wave_form.drawio.svg" />
</p>

* At cycle 2, the sender turns on the valid bit until cycle 4.
* The receiver turns on the ready bit at cycle 3 until cycle 4.
* The transfer happens at cycle 3 since only when both the valid bit and the ready bit are turned on.

### Specification

In HazardFlow HDL, we abstracted any arbitraty communication protocol into `Hazard` trait.
It describes the necessary information for communication: payload, resolver, and ready function.

<!-- We define hazard as a protocol with given payload, resolver, and ready function.
This protocol describes the necessary information between sender and receiver and their transfer condition. -->

```rust,noplayground
pub trait Hazard {
    type P: Copy;
    type R: Copy;

    fn ready(p: Self::P, r: Self::R) -> bool;
}
```

For any hazard type `H`, its member type and functions has the following meaning:

* `H::P`: Payload signal type.
* `H::R`: Resolver signal type.
* `H::ready`: Indicates the receiver is ready to receive with current pair of payload and resolver.

### Examples

We provide a few handy primitive hazard interfaces for developers.

#### Valid

Valid hazard represents a communication without backpressure, its ready function always returns `true`.
Its specification is as follows:

```rust,noplayground
pub struct ValidH<P: Copy, R: Copy>;

impl<P: Copy, R: Copy> Hazard for ValidH<P, R> {
    type P = P;
    type R = R;

    fn ready(p: P, r: R) -> bool {
        true
    }
}
```

* The payload type of the Valid Hazard Interface is `HOption<P>`.
* The resolver type of the Valid Hazard Interface is `R`.
* When the payload is valid, which means it is `Some(P)`, transfer happens since the `ready` function always returns `true`.
* Specially, when the resolver is `()` and the payload signal does not depend on the resolver signal, we define the Valid Hazard Interface as
  ```rust,noplayground
  pub type Valid<P> = I<ValidH<P, ()>, { Dep::Helpful }>;
  ```

<!-- For more information about dependency, please refer to the [dependency section](../advanced/dependency.md). -->

#### And

We define an **And** hazard `AndH<H: Hazard>`, whose resolver type is `Ready<H::R>`. `Ready<R>` is a `struct` containing both a resolver and a ready signal in HazardFlow HDL. The interface containing the And Hazard is an And Hazard Interface.

```rust,noplayground
pub struct AndH<H: Hazard>;

pub struct Ready<R: Copy> {
    pub ready: bool,
    pub inner: R,
}

impl<H: Hazard> Hazard for AndH<H> {
    type P = H::P;
    type R = Ready<H::R>;

    fn ready(p: H::P, r: Ready<H::R>) -> bool {
        r.ready && H::ready(p, r.inner)
    }
}
```

* The payload type of the And Hazard Interface is `HOption<P>`.
* The resolver type of the And Hazard Interface is `Ready<R>`.
* When the payload is valid, which means the payload is `Some(P)`, the ready signal in the resolver is `true`, and the `ready` function returns `true`, then transfer happens.

#### Valid-Ready

When the resolver is `()`, we combine the Valid Hazard and And Hazard and form the **Valid-Ready** hazard `VrH<P, R = ()>`. We define the Valid-Ready Hazard as `pub type VrH<P, R = ()> = AndH<ValidH<P, R>>`. The interface containing the Valid Ready Hazard is a Valid-Ready Interface.

We can represent the Valid-Ready protocol with using And and Valid protocol as follows:

```rust,noplayground
pub struct VrH<P: Copy> = AndH<ValidH<P, ()>>;
```

* The payload type of the Valid-Ready Interface is `HOption<P>`.
* The resolver type of the Valid-Ready Interface is `Ready<()>`.
* When the payload is valid, which means the payload is `Some(P)`, and the ready signal in the resolver is `true`, then transfer happens.
* Specially, we define the Valid-Ready Interface as `pub type Vr<P, const D: Dep = { Dep::Helpful }> = I<VrH<P>, D>`

## Interface

We define the interface as a protocol with forward signal, backward signal, and some other methods.
The other methods are related to the [combinator](./combinator.md) and [module](./module.md), please refer to the corresponding section.
Any `struct` implements the interface protocol we can consider it as an interface.

```rust,noplayground
pub trait Interface {
    type Fwd: Copy;
    type Bwd: Copy;

    ...
}
```

* Forward signal
  * This specifies the forward signal type.
* Backward signal
  * This specifies the backward signal type.
* Other functions
  * These functions are related to the [combinator](./combinator.md) and [module](./module.md), please refer to these sections for further reading.

### Hazard Interface

<p align="center">
  <img src="../figure/interface.drawio.svg" />
</p>

If a `struct` implements the interface trait and also contains a hazard, we consider it as a **hazard interface**. In the HazardFlow HDL, we define it as `I<H, D>`, where `H` is the hazard, and `D` is the dependency type of hazard protocol. For more information of the dependency, please refer to the [dependency section](../advanced/dependency.md).

```rust,noplayground
pub struct I<H: Hazard, D: Dep>;

impl<H: Hazard, const D: Dep> Interface for I<H, D> {
    type Fwd = HOption<H::P>,
    type Bwd = H::R,
}
```

* The interface's forward signal is an `HOption` type of hazard payload. 
* The backward signal is the hazard's resolver.
* When the forward signal is `Some(p)` means the sender is sending a valid payload, else it is sending an invalid payload signal. 
* When we have `payload.is_some_and(|p| H::ready(p, r))`, the transfer happens.

### Compound Interface

Compound types such as tuple, struct, and array also implement the `Interface` trait.
These types are useful when we use "1-to-N" or "N-to-1" combinators.

For example, array of interfaces also implements `Interface` trait as follows:

```rust,noplayground
impl<If: Interface, const N: usize> Interface for [If; N] {
    type Fwd = Array<If::Fwd, N>;
    type Bwd = Array<If::Bwd, N>;
}
```

* The forward signal of the array interface is the array of the interface's forward signal.
* The backward signal of the array interface is the array of the interface's backward signal.

As another example, tuple of interfaces also implements `Interface` trait as follows (in actual implementation, it is implemented as a macro):

```rust,noplayground
impl<If1: Interface, If2: Interface> Interface for (If1, If2) {
    type Fwd = (If1::Fwd, If2::Fwd);
    type Bwd = (If1::Bwd, If2::Bwd);
}
```

* The forward signal of the array interface is the tuple of the interface's forward signal.
* The backward signal of the array interface is the tuple of the interface's backward signal.
