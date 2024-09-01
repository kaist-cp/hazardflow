//! Flatten.

use super::*;

impl<P: Copy, R: Copy, const D: Dep> I<ValidH<HOption<P>, R>, D> {
    /// Flattens the payload.
    ///
    /// - Payload: Flattened.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress               | Egress       |
    /// | :-------: | --------------------- | ------------ |
    /// |  **Fwd**  | `HOption<HOption<P>>` | `HOption<P>` |
    /// |  **Bwd**  | `R`                   | `R`          |
    pub fn flatten(self) -> I<ValidH<P, R>, D> {
        self.filter_map(|p| p)
    }
}

impl<P: Copy, R: Copy, const D: Dep> I<VrH<HOption<P>, R>, D> {
    /// Flattens the payload.
    ///
    /// - Payload: Flattened.
    /// - Resolver: Preserved.
    ///
    /// | Interface | Ingress               | Egress       |
    /// | :-------: | --------------------- | ------------ |
    /// |  **Fwd**  | `HOption<HOption<P>>` | `HOption<P>` |
    /// |  **Bwd**  | `Ready<R>`            | `Ready<R>`   |
    pub fn flatten(self) -> I<VrH<P, R>, D> {
        self.filter_map(|p| p)
    }
}
