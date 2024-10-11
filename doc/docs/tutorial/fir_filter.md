# FIR Filter

In this tutorial, we will use HazardFlow HDL to describe an [FIR (finite impulse response) filter](https://en.wikipedia.org/wiki/Finite_impulse_response), which is very commonly used in digital signal processing applications.

## Specification

The FIR filter of order *N* performs the following operation:

<center><img src="../figure/fir-filter-spec.svg" width="85%"></center>

It receives input signals from the stream *x* and outputs the weighted sum of the most recent *N+1* input signals to the stream *y*.
It can be expressed with the following formula:

\\[ y[n] = b_0 x[n] + b_1 x[n-1] + ... + b_N x[n-N] = \sum^{N}_{i=0} b_i x[n-i] \\]

where \\( x[n] \\) and \\( y[n] \\) represent the input and output signals, \\( N \\) represents the filter order, and \\( b_i \\) represents the *i*-th filter coefficient.

For example, the IO signals of a FIR filter of order 2 with coefficients [4, 2, 3] are as follows:

| n   | x[n] | y[n]                 |
| --- | ---- | -------------------- |
| 0   | 1    | 4·1 + 2·0 + 3·0 = 4  |
| 1   | 4    | 4·4 + 2·1 + 3·0 = 18 |
| 2   | 3    | 4·3 + 2·4 + 3·1 = 23 |
| 3   | 2    | 4·2 + 2·3 + 3·4 = 26 |
| 4   | 7    | 4·7 + 2·2 + 3·3 = 41 |
| 5   | 0    | 4·0 + 2·7 + 3·2 = 20 |

For more details, please refer to [Wikipedia](https://en.wikipedia.org/wiki/Finite_impulse_response).

## Modular Design

We could represent the FIR filter of order 2 in modular way as follows:

<center><img src="../figure/fir-filter-modular.svg" width="80%"></center>

As in the above figure, it can be divide into 3 submodules: `window`, `weight`, and `sum`.

**`window` submodule:**

* It serves as a sliding window, always returning the latest 3 valid input signals as an array.
* For example, if 1, 4, 3 are given as input signals, `[1, 0, 0]`, `[4, 1, 0]`, `[3, 4, 1]` will be returned as output signals.
<!-- * If input signal is invalid at a certain cycle, it will be ignored. -->

**`weight` submodule:**

* It keeps the weight vector `[b0, b1, b2]` persistent throughout the program.
* It takes the input vector `[v0, v1, v2]` and returns the output vector `[b0·v0, b1·v1, b2·v2]`.
* This submodule can be simply represented by a `map` combinator.

**`sum` submodule:**

* It takes the input vector and returns the sum of them as an output vector.

## Implementation

Based on the above submodules, we can implement the FIR filter in a concise and modular way:

```rust,noplayground
fn fir_filter(input: Valid<u32>) -> Valid<u32> {
    let weight = Array::<u32, 3>::from([4, 2, 3]);

    input
        .window::<3>()
        .map(|ip| ip.zip(weight).map(|(e, wt)| e * wt))
        .sum()
}
```

We can describe the FIR filter with `window`, `map`, and `sum` combinators in the HazardFlow HDL and we assume the input interface `Valid<u32>` is provided.
`Valid<u32>`, which is an alias of [`I<ValidH<u32, ()>>`](../lang/interface.md#validh) is a **valid interface** where its payload is `u32`, the resolver is empty `()`, and its `ready` function always returns `true`.
In other words, as long as the input interface's forward signal is `Some(u32)` at a specific clock cycle, the receiver receives a valid payload.
We can interpret this input interface as a stream of signal values flowing through the wires.

**`window` combinator:**

The `window` combinator is defined as follows:

```rust,noplayground
impl<P: Copy + Default> Valid<P> {
    fn window<const N: usize>(self) -> Valid<Array<P, N>> {
        self.fsm_map(P::default().repeat::<{ N - 1 }>(), |ip, s| {
            let ep = ip.repeat::<1>().append(s).resize::<N>();
            let s_next = ep.clip_const::<{ N - 1 }>(0);
            (ep, s_next)
        })
    }
}
```

It takes an `Valid<P>` and returns `Valid<Array<P, N>>`.
It tracks the latest `N` valid input signals.
The [`fsm_map` interface combinator](https://kaist-cp.github.io/hazardflow/docs/hazardflow_designs/std/hazard/struct.I.html#method.fsm_map) is provided by the HazardFlow HDL standard library.
It computes the egress payload and the next state based on the ingress payload and the current state, and updates the state when the ingress tranfser happens.
The initial state is defined as `P::default().repeat::<{ N - 1 }>()` in our example.
The anonymous function is where we specify the fsm logic from the `(ingress payload, current state)` to the `(egress payload, next state)`.

<!-- 
* Ingress interface is `Valid<P>`.
* Egress interface is `Valid<Array<P, N>>`, where `N` is the size of the FIR filter.
* It should have a state to keep tracking of the latest valid input and return the latest `N` valid values every clock cycle. -->

<!-- * `impl<P: Copy + Default> Valid<P>` is how we define a custom combinator for the input interface `Valid<P>`, where `P` should be able to be copied and should have a default value.
* Then we define the `window` combinator as `pub fn window<const N: usize>(self) -> Valid<Array<P, N>>`, where `N` is the size of the FIR filter, and the egress interface's type is `Valid<Array<P, N>>`. -->
<!-- * The egress interface's payload is `Option<Array<P, N>>`, an optional type of array with `P` type elements, and the array size is `N`. The resolver is empty `()`. -->
<!-- * The anonymous function takes the ingress payload and the current state as inputs and returns the egress payload and next state.
  * The `append` function concats two arrays together.
  * The `ip.repeat::<1>()` function transforms `ip` into an array of one element `ip`.
  * The `clip_const::<N>(0)` function clips the array from index 0 of size `N`.
  * Note that in HazardFlow HDL, the array index is in descending order from left to right, for more details please refer to the [signal](./signal.md) section. -->

**`map` combinator:**

The `map` combinator is used to represent the `weight` submodule.

```rust,noplayground
map(|ip| ip.zip(weight).map(|(e, wt)| e * wt))
```

It takes an `Valid<Array<u32, N>>` and returns an egress hazard interface `Valid<Array<u32, N>>`.
It transforms the `i`-th element of ingress payload `ip[i]` into `ip[i] * weight[i]`, and leaves the resolver as untouched.
The [`map` interface combinator](https://kaist-cp.github.io/hazardflow/docs/hazardflow_designs/std/hazard/struct.I.html#method.map) is provided by the HazardFlow HDL standard library.
We can interpret it as stateless version of `fsm_map`.
In the application-specific logic in `map` interface combinator, we use `zip` and `map` methods for manipulating the ingress payload signal.

<!-- It takes an `Valid<Array<u32, N>>` and returns another `Valid<Array<u32, N>>`. -->
<!-- 
* `impl<const M: usize, const N: usize> Valid<Array<U<M>, N>>` is how we define a custom combinator for the input interface `Valid<Array<U<M>, N>>`. -->
<!-- * The `map` combinator is a primitive combinator provided by the HazardFlow HDL standard library.
The anonymous function transforms the ingress payload to the egress payload in the same clock cycle.
We can interpret it as `fsm_map` with the `()` state. -->
<!-- * `zip` function zips two `Array` together and returns new `Array` that has tuples from two given arrays as elements.
* We use `map` combinator again to multiply two elements from the tuple and return the egress payload. -->

**`sum` combinator:**

The `sum` combinator is defined as follows:

```rust,noplayground
impl<const N: usize> Valid<Array<u32, N>> {
    fn sum(self) -> Valid<u32> {
        self.map(|ip| ip.fold(0, |acc, e| acc + e))
    }
}
```

It takes an `Valid<Array<u32, N>>` and returns `Valid<u32>`.
It transforms the ingress payload to sum of them.
In the application-specific logic in `map` interface combinator, we use `fold` method which aggregates the data within array of signal.

You can find the implementation in [fir_filter.rs](https://github.com/kaist-cp/hazardflow/blob/main/hazardflow-designs/src/examples/fir_filter.rs).

You can generate the Verilog codes with the following commands:

```bash
# Generate a separate Verilog file for each submodule.
$ cargo run --release -- --target fir_filter --deadcode --wire-cache

# Generate an integrated Verilog file combining all submodules.
$ cargo run --release -- --target fir_filter --deadcode --wire-cache --merge
```
