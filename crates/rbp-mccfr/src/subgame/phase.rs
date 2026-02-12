//! Phase of execution within the subgame structure.
/// Phase of execution within the subgame structure.
///
/// The subgame structure has three phases:
/// - `Prefix`: Replaying forced history to build reach calculations
/// - `MetaGame`: Opponent chooses which "world" to enter
/// - `RealGame`: Normal subgame play after world selection
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubPhase {
    /// Replaying forced prefix history (single forced edge per node).
    /// Tuple is (cursor, length) for position and total prefix edges.
    Prefix(usize, usize),
    /// Opponent is choosing among alternative reach distributions.
    Meta,
    /// Normal gameplay within the resolved subgame.
    /// Carries the selected world index for per-world perturbations.
    Real(usize),
}
