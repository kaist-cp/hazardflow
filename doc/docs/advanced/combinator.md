# Implementing Your Own Combinators

The HazardFlow HDL standard library provides some primitive combinators, however we need to implement our custom combinators sometimes for specific logic.
In fact, the HazardFlow HDL standard library is also implemented in the same way as what we will introduce in this section.

We introduced the `masked_merge` combinator specification in the [Tutorial](../tutorial/masked_merge.md) section,
but we did not give out the concrete implementation.

The `masked_merge` combinator is a custom combinator which is not provided by the HazardFlow HDL standard library,
thus we need to implement its logic by ourselves.

## Implementation

To define a custom combinator for interface type `[Vr<P>; N]`, we need to define a custom trait first.

```rust,noplayground
/// Masked merge trait
trait MaskedMergeExt<P: Copy + Default, const N: usize>: Interface
where [(); clog2(N)]:
{
    /// Hazard type
    type EH: Hazard;

    /// Fair Mux
    fn masked_merge(self) -> I<Self::EH, { Dep::Demanding }>;
}
```

* The custom trait's name is `MaskedMergeExt`.
  * It specifies the general payload type `P` need to be constrained by the `Copy` and `Default` traits.
  * `const N: usize` specifies the number of ingress interfaces.
  * The `MaskedMergeExt` need to extend the `Interface trait`, since we need to `impl` this trait later for Implementing our custom combinator.
  * `where [(); clog2(N)]` is telling the HazardFlow HDL compiler that `clog2(N)` is a constant.
* We define the egress hazard as `Hazard` type, which is a hazard protocol with a given payload, resolver, and the `ready` function.
* `fn masked_merge(self) -> I<Self::EH, { Dep::Demanding }>` defines the combinator's name `masked_merge` and specifies the egress hazard is `EH`.

We can define the combinational logic now.

```rust,noplayground
impl<P: Copy + Default, const N: usize> MaskedMergeExt<P, N> for [Vr<P>; N]
where [(); clog2(N)]:
{
    type EH = VrH<(P, U<{ clog2(N) }>), Array<bool, N>>;

    fn masked_merge(self) -> I<Self::EH, { Dep::Demanding }> {
        unsafe {
            self.fsm::<I<VrH<(P, U<{ clog2(N) }>), Array<bool, N>>, { Dep::Demanding }>, ()>((), |ip, er, s| {
                if !er.ready {
                    let ir = Ready::new(false, ()).repeat();
                    let ep = None;
                    return (ep, ir, s);
                }

                let ep_idx = ip.zip(er.inner).find_idx(|(p, selected)| p.is_some() && !selected);
                let ep = if let Some(idx) = ep_idx { Some((ip[idx].unwrap(), idx)) } else { None };

                let ir = Ready::invalid().repeat::<N>().set_cond(ep.is_some(), ep_idx.unwrap(), Ready::valid(()));
                (ep, ir, s)
            })
        }
    }
}
```

* We `impl` the custom trait `MaskedMergeExt` for the compound interface type `[Vr<P>; N]`.
* We define the egress hazard as `VrH<(P, U<{ clog2(N) }>), Array<bool, N>>`
  * The egress interface is a valid-ready interface with valid-ready hazard.
  * `U<{ clog2(N) }>` is the bit representation of the index of the ingress interfaces.
  * The inner resolver is `Array<bool, N>` which indicates the index of the ingress interfaces are present in the current queue.
* `unsafe` code block is necessary for Implementing your own custom combinator, since `fsm` function need to be in the `unsafe` code block.
* The `fsm` function specifies the egress interface type is `I<VrH<(P, U<{ clog2(N) }>), Array<bool, N>`.
* The `fsm` function takes two inputs and returns the egress interface.
  * The initial state of the combinator.
  * An anonymous function takes three inputs.
    * The ingress payload `ip`. 
      The type is `Array<HOption<P>, N>`, which is all the ingress interfaces' payload.
    * The egress resolver `er`.
      The type is `Ready<Array<bool, N>>`, which is a ready signal with an array that indicates the index of the ingress interfaces present in the current queue.
    * The initial state `s`.
      The type is simply a `()`, since we do not have a state in this combinator.
  * The anonymous function returns a tuple including:
    * The egress payload `ep`.
      The type is `HOption<(P, Array<bool, _>)>`, which is the actual data we want to transfer and also the index of the ingress interface.
      Note that the bit representation of an unsigned integer is an array of `bool`.
    * The ingress resolver `ip`.
      The type is `Array<Ready<()>, N>`, which is an array of ready signals with size `N`.
      It is a collection of the ready signal for each ingress interface.
* If the egress resolver ready signal is `false`, which means the egress interface is not ready to transfer the payload, we are not transferring any payload.
  * We set the ingress resolver's ready signal as `false` 
  * We set the egress payload as `None`.
* If the egress resolver ready signal is `true`, which means the egress interface is ready to transfer the payload.
  * We find the first index of the ingress interfaces whose payload is `Some(P)` and also not in the current queue.
    * `zip` function zips two arrays and returns a new array whose element is a tuple of both elements from the two arrays.
    * `find_idx` finds the index of first element that satisfies given condition.
  * We set the egress payload as a tuple containing the selected ingress interface's actual payload and its index.
  * We set the selected ingress resolver ready signal as `true` and the rest of the ingress interfaces' ready signal as `false`.



