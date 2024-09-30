# Masked Merge

In this tutorial, we will use the `masked_merge` combinator to implement a custom arbiter that does not select the recently chosen ingress interface again.

## Specification

The `masked_merge` combinator performs the following operation (`U<M>` is a bitvector of width `M`):

<p align="center">
  <img src="../figure/masked-merge.svg" width=80%/>
</p>

<!-- TODO: to explain masked merge, we don't need to think of FIFO queue. It can be an example of using masked merge, but the current text *assumes* there should be a FIFO queue: "It indicates which ingress interfaces are present in the current queue." -->

- It takes N valid-ready interfaces (`Vr<P>` = `I<VrH<P, ()>`).
- It returns a valid-ready hazard interface (`I<VrH<(P, U<{ clog2(N) }>), U<N>>>`).
  + The `U<{ clog2(N) }>` in the payload indicates the index of selected ingress interface.
  + The `U<N>` in the resolver indicates the mask bits for the selection.
- It will select from the ingress interface which has valid payload with the mask bit has `false`.
  + For example, if `N` is 4 and the mask bits are `[true, true, false, true]`, then it will try to select from the third ingress interface.
  + If multiple ingress interfaces are selectable, the one with the smallest index is chosen.

<!-- * We can think of a valid-ready Interface as a valid interface `Valid<P>` with an extra ready signal (Boolean value) in its resolver. -->
<!-- * The transfer happens only when the payload is `Some(P)`, and the ready signal in the resolver is `true`. -->
<!-- * We can represent the ingress interface type as `[Vr<u32>; N]`. -->
<!-- * For more information about the valid-ready interface please refer to the [valid-ready interface](../lang/interface.md#vrh). -->
<!-- * For more information about the compound interface type, please refer to the [compound interface section](../lang/interface.md#compound-interface). -->
<!--
The Masked Merge combinator egress interface is also a valid-ready hazard interface `I<Self::EH, { Dep::Demanding }>`.
* We define the egress hazard as `type EH = VrH<(P, U<{ clog2(N) }>), Array<bool, N>>`.
  * The payload type is a tuple type.
    * `Option<P>` contains the real data we want to send through the wires.
    * `U<{ clog2(N) }>` is the index of the ingress interfaces represented in bits. `clog2(N)` is the bit-width for representing integer `N`.
    * The payload will be sent to the FIFO queue. 
    * The element in the FIFO queue is a tuple containing the actual data and the index of the ingress interface that sends the data.
  * The resolver is an array of `bool` of size `N`.
    * This resolver is sent back from the FIFO queue.
    * It indicates which ingress interfaces are present in the current queue.
    * If there are 4 ingress interfaces and the array is `[true, false, false, true]`, it indicates the ingress interface 1's and ingress interface 2's payloads are not currently in the queue.
-->

## Modular Design

It explain the example use-case of the `masked_merge` combinator.

In this example, a FIFO is placed after `masked_merge` to track which ingress interface the recent transfers occurred on.
This information is used to prevent consecutive selections from the same ingress interface.

<p align="center">
  <img src="../figure/masked-merge-use-case.svg" width=95% />
</p>

**`masked_merge` combinator:**

- It selects one of the ingress interfaces to transfer its payload to the next combinator.
- The selection is based on the mask bits (`U<N>`) from the egress resolver.
- If multiple ingress interfaces are selectable, the one with the smallest index is chosen.

**`map_resolver` combinator:**

- It transforms the inner resolver signal from the FIFO state (`FifoS`) to the mask bits (`U<N>`).
  + `FifoS` indicates the current state of the FIFO, including elements and head/tail indices.
  + The `i`-th element of mask bits (`U<N>`) indicates that one of the payloads currently in the FIFO has been selected from the `i`-th ingress interface of `M0`.
- It does not touch the forward signals and the ready signal.
<!--
- The `U<N>` indicates which ingress interfaces are present in the current FIFO.
-->

**`transparent_fifo` combinator:**

- It takes a payload from the ingress interface when the FIFO is not full.
- It returns the oldest payload in the FIFO to the egress interface.
- It sends the FIFO's fullness (`Ready`) and the current FIFO state (`FifoS`) as the ingress resolver.

<!-- The FIFO Queue ingress interface:
* The payload is a tuple containing the actual data we want to transfer and also the index of the ingress interface of the Masked Merge combinator that sends the data.
* The resolver indicates which ingress interfaces are present in the current queue.

The FIFO Queue egress interface is a simple valid-ready interface `Vr<P>`. -->

**`map` combinator:**

- It drops the unnecessary index information in the payload.

## Implementation

- We use `u32` as the actual payload type for demonstrating a more concrete example.
- We also set the number of ingress interfaces as 5, the same as the FIFO size.
- `fifo_s.inner` is the inner elements of the FIFO.
- We `fold` the inner elements of the FIFO in the `map_resolver` combinator:
  - The initializer is a Boolean array with all elements as `false` of size 5. 
  - The index of the initializer array indicates the index of the ingress interfaces.
  - We iterate through all the elements within the FIFO and set the accumulator's value at the index in each FIFO element to `true`.
  - The final result indicates which ingress interfaces are present in the current FIFO.
- We send back this resolver to the `masked_merge` combinator to make decision for choosing the next ingress interface.
- We filter out the unnecessary index information in the last `map` combinator.
- The implementation of the `masked_merge()` combinator will be explained in the [Implementing Combinators](../advanced/combinator.md) section.

```rust,noplayground
/// Example module using `masked_merge` combinator.
pub fn m(ingress: [Vr<u32>; 5]) -> Vr<u32> {
    ingress
        .masked_merge()
        .map_resolver::<((), FifoS<(u32, U<{ clog2(5) }>), 5>)>(|er| {
            let (_, fifo_s) = er.inner;
            fifo_s.inner.fold(Array::from([false; 5]), |acc, (_p, idx)| acc.set(idx, true))
        })
        .transparent_fifo()
        .map(|(ip, _idx)| ip)
}
```

You can find the full implementation in [masked_merge.rs](https://github.com/kaist-cp/hazardflow/blob/main/hazardflow-designs/src/examples/masked_merge.rs).

Congratulations! You finished the tutorial!
