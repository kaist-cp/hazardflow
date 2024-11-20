//! FIFO.

use super::*;

/// State for `N`-sized FIFO.
#[derive(Debug, Clone, Copy)]
pub struct FifoS<P: Copy, const N: usize>
where
    [(); clog2(N)]:,
    [(); clog2(N + 1)]:,
{
    /// Inner elements.
    pub inner: Array<P, N>,
    /// Read address.
    pub raddr: U<{ clog2(N) }>,
    /// Write address.
    pub waddr: U<{ clog2(N) }>,
    /// Length.
    pub len: U<{ clog2(N + 1) }>,
}

impl<P: Copy, const N: usize> Default for FifoS<P, N>
where
    [(); clog2(N)]:,
    [(); clog2(N + 1)]:,
{
    fn default() -> Self {
        Self { inner: unsafe { x() }, raddr: U::from(0), waddr: U::from(0), len: U::from(0) }
    }
}

impl<P: Copy, const N: usize> FifoS<P, N>
where
    [(); clog2(N)]:,
    [(); clog2(N + 1)]:,
    [(); clog2(N) + 1]:,
{
    /// Returns the head of the FIFO.
    pub fn head(self) -> HOption<P> {
        if self.len == 0.into_u() {
            None
        } else {
            Some(self.inner[self.raddr])
        }
    }

    /// Returns inner elements with valid bit.
    pub fn inner_with_valid(self) -> Array<HOption<P>, N> {
        range::<N>().map(|i| {
            if i.resize() >= self.len {
                None
            } else {
                Some(self.inner[wrapping_add::<{ clog2(N) }>(self.raddr, i, N.into_u())])
            }
        })
    }
}

impl<P: Copy, R: Copy, const D: Dep> I<VrH<P, R>, D> {
    /// FIFO queue with `N` entries.
    ///
    /// This queue is fully pipelined, which means it can accept a new element every cycle.
    ///
    /// - Payload: If an ingress transfer happens, the ingress payload is enqueued. If an egress transfer happens, the
    ///     egress payload is dequeued. The front (dequeue-side) element is outputted as an egress payload.
    /// - Resolver: The ingress ready signal is true if the queue can accept new elements, i.e. not full. The inner
    ///     value `R` of the resolver is preserved.
    ///
    /// | Interface | Ingress      | Egress       |
    /// | :-------: | ------------ | ------------ |
    /// |  **Fwd**  | `HOption<P>` | `HOption<P>` |
    /// |  **Bwd**  | `Ready<R>`   | `Ready<R>`   |
    pub fn fifo<const N: usize>(self) -> I<VrH<P, R>, { Dep::Helpful }>
    where
        [(); clog2(N) + 1]:,
        [(); clog2(N + 1) + 1]:,
    {
        self.map_resolver_inner::<(R, _)>(|er| er.0).transparent_fifo()
    }
}

impl<const D: Dep, const N: usize, P: Copy, R: Copy> I<VrH<P, (R, FifoS<P, N>)>, D>
where
    [(); clog2(N)]:,
    [(); clog2(N + 1)]:,
{
    /// A variation of [`I::fifo`] that additionally outputs the internal FIFO state to the ingress resolver.
    ///
    /// - Payload: The same behavior as [`I::fifo`].
    /// - Resolver: The same behavior as [`I::fifo`], but additionally the FIFO state `FifoS<P, N>` is outputted.
    ///
    /// | Interface | Ingress                   | Egress       |
    /// | :-------: | ------------------------- | ------------ |
    /// |  **Fwd**  | `HOption<P>`              | `HOption<P>` |
    /// |  **Bwd**  | `Ready<(R, FifoS<P, N>)>` | `Ready<R>`   |
    // TODO: add `flow` and `pipe` parameters.
    // - `flow`: reduce latency when FIFO is empty.
    // If flow bit is valid and FIFO is empty, then ingress payload goes out as egress payload directly.
    // - `pipe`: reduce latency when FIFO is full.
    // If pipe bit is valid and FIFO is full, ingress payload can come in if egress payload goes out.
    // Refer to below link for more details:
    // <https://github.com/chipsalliance/chisel/blob/v3.2.1/src/main/scala/chisel3/util/Decoupled.scala#L235-L246>
    pub fn transparent_fifo(self) -> I<VrH<P, R>, { Dep::Helpful }>
    where
        [(); clog2(N) + 1]:,
        [(); clog2(N + 1) + 1]:,
    {
        self.multi_headed_transparent_fifo().map_resolver_inner(|r| (r, U::from(1))).filter_map(|s| {
            if s.len == 0.into_u() {
                None
            } else {
                Some(s.inner[s.raddr])
            }
        })
    }

    /// A variation of [`I::transparent_fifo`] that outputs the FIFO state instead of the front element as the egress payload,
    /// and takes an additional egress resolver signal representing how many elements will be popped.
    ///
    /// - Payload: The same behavior as [`I::transparent_fifo`], but the FIFO state `FifoS<P, N>` is outputted instead.
    /// - Resolver: The same behavior as [`I::transparent_fifo`], but additionally takes a `U<{ clog2(N + 1) }>` that
    ///     represents how many elements to pop.
    ///
    /// | Interface | Ingress                   | Egress                            |
    /// | :-------: | ------------------------- | --------------------------------- |
    /// |  **Fwd**  | `HOption<P>`              | `HOption<FifoS<P, N>>`            |
    /// |  **Bwd**  | `Ready<(R, FifoS<P, N>)>` | `Ready<(R, U<{ clog2(N + 1) }>)>` |
    #[allow(clippy::type_complexity)]
    pub fn multi_headed_transparent_fifo(self) -> I<VrH<FifoS<P, N>, (R, U<{ clog2(N + 1) }>)>, { Dep::Helpful }>
    where
        [(); clog2(N) + 1]:,
        [(); clog2(N + 1) + 1]:,
    {
        unsafe {
            self.fsm::<FifoS<P, N>, { Dep::Helpful }, VrH<FifoS<P, N>, (R, U<{ clog2(N + 1) }>)>>(
                FifoS::default(),
                |ip, er, s| {
                    let FifoS { inner, raddr, waddr, len } = s;
                    let pop = er.inner.1;

                    let empty = len == U::from(0);
                    let full = len == U::from(N);

                    let enq = ip.is_some() && !full;
                    let deq = er.ready && !empty;

                    let ep = Some(s);
                    let ir = Ready::new(!full, (er.inner.0, s));

                    let inner_next = if enq { inner.set(waddr, ip.unwrap()) } else { inner };
                    let len_next = (len + U::from(enq).resize() - if deq { pop.resize() } else { 0.into_u() }).resize();
                    let raddr_next =
                        if deq { wrapping_add::<{ clog2(N) }>(raddr, pop.resize(), N.into_u()) } else { raddr };
                    let waddr_next = if enq { wrapping_inc::<{ clog2(N) }>(waddr, N.into_u()) } else { waddr };

                    let s_next = FifoS { inner: inner_next, raddr: raddr_next, waddr: waddr_next, len: len_next };

                    (ep, ir, s_next)
                },
            )
        }
    }
}

impl<const D: Dep, const N: usize, P: Copy> I<VrH<P, FifoS<P, N>>, D>
where
    [(); clog2(N)]:,
    [(); clog2(N + 1)]:,
{
    /// A variation of [`I::transparent_fifo`] that has valid-ready egress interface.
    ///
    /// - Payload: The same behavior as [`I::transparent_fifo`].
    /// - Resolver: The same behavior as [`I::transparent_fifo`], but unnecessary unit type is removed in the ingress
    ///     resolver signal.
    ///
    /// | Interface | Ingress              | Egress       |
    /// | :-------: | -------------------- | ------------ |
    /// |  **Fwd**  | `HOption<P>`         | `HOption<P>` |
    /// |  **Bwd**  | `Ready<FifoS<P, N>>` | `Ready<()>`  |
    pub fn transparent_fifo(self) -> Vr<P>
    where
        [(); clog2(N) + 1]:,
        [(); clog2(N + 1) + 1]:,
    {
        self.multi_headed_transparent_fifo().map_resolver_inner(|_| U::from(1)).filter_map(|s| {
            if s.len == 0.into_u() {
                None
            } else {
                Some(s.inner[s.raddr])
            }
        })
    }

    /// A variation of [`I::multi_headed_transparent_fifo`] that has valid-ready egress interface.
    ///
    /// - Payload: The same behavior as [`I::multi_headed_transparent_fifo`].
    /// - Resolver: The same behavior as [`I::multi_headed_transparent_fifo`], but unnecessary unit type is removed in
    ///     the ingress resolver signal.
    ///
    /// | Interface | Ingress               | Egress                       |
    /// | :-------: | --------------------- | ---------------------------- |
    /// |  **Fwd**  | `HOption<P>`          | `HOption<FifoS<P, N>>`       |
    /// |  **Bwd**  | `Ready<FifoS<P, N>>`  | `Ready<U<{ clog2(N + 1) }>>` |
    #[allow(clippy::type_complexity)]
    pub fn multi_headed_transparent_fifo(self) -> I<VrH<FifoS<P, N>, U<{ clog2(N + 1) }>>, { Dep::Helpful }>
    where
        [(); clog2(N) + 1]:,
        [(); clog2(N + 1) + 1]:,
    {
        unsafe {
            self.fsm::<FifoS<P, N>, { Dep::Helpful }, VrH<FifoS<P, N>, U<{ clog2(N + 1) }>>>(
                FifoS::default(),
                |ip, er, s| {
                    let FifoS { inner, raddr, waddr, len } = s;
                    let pop = er.inner;

                    let empty = len == U::from(0);
                    let full = len == U::from(N);

                    let enq = ip.is_some() && !full;
                    let deq = er.ready && !empty;

                    let ep = Some(s);
                    let ir = Ready::new(!full, s);

                    let inner_next = if enq { inner.set(waddr, ip.unwrap()) } else { inner };
                    let len_next = (len + U::from(enq).resize() - if deq { pop.resize() } else { 0.into_u() }).resize();
                    let raddr_next =
                        if deq { wrapping_add::<{ clog2(N) }>(raddr, pop.resize(), N.into_u()) } else { raddr };
                    let waddr_next = if enq { wrapping_inc::<{ clog2(N) }>(waddr, N.into_u()) } else { waddr };

                    let s_next = FifoS { inner: inner_next, raddr: raddr_next, waddr: waddr_next, len: len_next };

                    (ep, ir, s_next)
                },
            )
        }
    }
}
