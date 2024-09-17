# Masked Merge

## Specification

<p align="center">
  <img src="../figure/masked_merge.drawio.svg" />
</p>

### `masked_merge` combinator

The `masked_merge` combinator takes `N` valid-ready `Vr<P>` interfaces as the ingress interface.
* We can think of a valid-ready Interface as a valid interface `Valid<P>` with an extra ready signal (Boolean value) in its resolver.
* The transfer happens only when the payload is `Some(P)`, and the ready signal in the resolver is `true`.
* We can represent the ingress interface type as `[Vr<u32>; N]`.
* For more information about the valid-ready interface please refer to the [valid-ready interface](../lang/interface.md#vrh).
* For more information about the compound interface type, please refer to the [compound interface section](../lang/interface.md#compound-interface).
The Masked Merge combinator egress interface is also a valid-ready hazard interface `I<Self::EH, { Dep::Demanding }>`.
* We define the egress hazard as `type EH = VrH<(P, U<{ clog2(N) }>), Array<bool, N>>`.
  * The payload type is a tuple type.
    * `Opt<P>` contains the real data we want to send through the wires.
    * `U<{ clog2(N) }>` is the index of the ingress interfaces represented in bits. `clog2(N)` is the bit-width for representing integer `N`.
    * The payload will be sent to the FIFO queue. 
    * The element in the FIFO queue is a tuple containing the actual data and the index of the ingress interface that sends the data.
  * The resolver is an array of `bool` of size `N`.
    * This resolver is send back from the FIFO queue.
    * It indicates which ingress interfaces are present in the current queue.
    * If there are 4 ingress interfaces and the array is `[true, false, false, true]`, it indicates the ingress interface 1's and ingress interface 2's payloads are not currently in the queue.

The Masked Merge combinator will try to choose the ingress interface whose payload is not in the queue and send it to the FIFO queue in the next clock cycle.

### FIFO Queue

The FIFO Queue ingress interface:
* The payload is a tuple containing the actual data we want to transfer and also the index of the ingress interface of the Masked Merge combinator that sends the data.
* The resolver indicates which ingress interfaces are present in the current queue.

The FIFO Queue egress interface is a simple valid-ready interface `Vr<P>`.

## Modular Design

<p align="center">
  <img src="../figure/masked_merge_module.drawio.svg" />
</p>

**`masked_merge` combinator:**

* It selects one of the ingress interfaces to transfer its payload to the next combinator.
* The selection is based on the current existing elements in the queue.
* We will choose the ingress interface with the smallest index, if there are multiple non-selected ingress interfaces.

**`map_resolver` combinator:**

* It transform its inner egress resolver `((), FifoS)` into `[bool; N]`.
* The `[bool; N]` indicates which ingress interfaces are present in the current queue.
* `FifoS` indicates the current state of the FIFO queue.
* This combinator will leave the payload untouched and transfer it from ingress interface to egress interface.

**`naked_fifo` combinator:**

* This is a primitive combinator offered by the standard library.
* It takes one element from the ingress payload and stores it in the queue every clock cycle.
* It returns the current queue status `FifoS`, including the inner elements of the queue, the reader address, the writer address, and the length of the current queue as the ingress resolver.
* It pops out one element from the queue as the egress payload every clock cycle.
* The egress resolver is a simple ready signal.

**`map` combinator:**

* It transforms the ingress payload `Opt<P, idx>` into `Opt<P>` for filtering out the unnecessary index information.

## Implementation

* We use `u32` as the actual payload type for demonstrating a more concrete example.
* We also set the number of ingress interfaces as 5, the same as the queue size.
* `fifo_s.inner` is the inner elements of the queue.
* We `fold` the inner elements of the queue:
  * The initializer is a Boolean array with all elements as `false` of size 5. 
  * The index of the initializer array indicates the index of the ingress interfaces.
  * We iterate through all the elements within the queue and set the accumulator's value at the index in each queue element to `true`.
  * The final result indicates which ingress interfaces are present in the current queue.
* We send back this resolver to the Masked Merge combinator to make decision for choosing the next ingress interface.
* We filter out the unnecessary index information in the last `map` combinator.
* The implementation of the `masked_merge()` combinator will be explained in the [Implementing Combinators](../advanced/combinator.md) section.

```rust,noplayground
/// Masked Merge Combinator
pub fn m(ingress: [Vr<u32>; 5]) -> Vr<u32> {
    ingress
        .masked_merge()
        .map_resolver::<((), FifoS<(u32, U<{ clog2(5) }>), 5>)>(|er| {
            let (_, fifo_s) = er.inner;
            fifo_s.inner.fold(Array::from([false; 5]), |acc, (_p, idx)| acc.set(idx, true))
        })
        .naked_fifo()
        .map(|(ip, _idx)| ip)
}
```

You can find the full implementation in [masked_merge.rs](https://github.com/kaist-cp/hazardflow/blob/main/hazardflow-designs/src/examples/masked_merge.rs).

Congratulations! You finished the tutorial!
