//! Unified resolution result type.

/// Outcome of running a `Translation` policy against a [`crate::Lattice`] and
/// a [`crate::Scalar`]: `Snap` carries a canonical lattice payload `P`, `Free`
/// hands the caller's off-grid value `F` back verbatim. `Free` is only emitted
/// by Brown-style injection policies (`Exact`, `EpsilonPrune`,
/// `EpsilonHarmonic`).
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Translated<P, F> {
    Snap(P),
    Free(F),
}
