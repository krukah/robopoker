//! Unified resolution result type.

/// Outcome of running a `Translation` policy against a
/// [`crate::Lattice`] and a [`crate::Scalar`].
///
/// - [`Self::Snap`] — resolved to a canonical lattice payload (the
///   common case under all current policies).
/// - [`Self::Free`] — left off-grid; carries the verbatim observation
///   value supplied by the caller. Only emitted by Brown-style
///   injection policies (`Exact`, `EpsilonPrune`, `EpsilonHarmonic`),
///   which are not yet wired up.
///
/// Type parameters:
/// - `P` is the lattice payload type (e.g. `Size`, `Edge`, `()`)
/// - `F` is the off-grid value type chosen by the caller (e.g. `Chips`,
///   `Action`) — handed back unchanged in the [`Self::Free`] arm.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Translated<P, F> {
    Snap(P),
    Free(F),
}
