//! Game state wrapper for subgame solving.
use super::*;
use crate::*;
use rbp_core::SUBGAME_ALTS;
use rbp_core::Utility;

/// Game state wrapper that adds prefix replay and alternatives at the subgame root.
///
/// The subgame structure follows the Pluribus paper with prefix support:
/// 1. Prefix phase: replay forced history for reach calculations
/// 2. MetaGame phase: opponent selects among K alternatives
/// 3. RealGame phase: normal subgame play proceeds
///
/// # Type Parameters
///
/// - `G`: The inner game type implementing `CfrGame`
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubGame<G>
where
    G: CfrGame,
{
    /// Current phase within the subgame structure.
    phase: SubPhase,
    /// The underlying game state.
    inner: G,
    /// The opponent player who selects alternatives.
    villain: G::T,
}

impl<G> SubGame<G>
where
    G: CfrGame,
{
    /// Creates a new subgame starting from game root with prefix.
    ///
    /// # Arguments
    ///
    /// - `villain`: The player who selects alternatives (non-traverser)
    /// - `length`: Number of edges in the prefix history
    pub fn new(villain: G::T, length: usize) -> Self {
        Self {
            inner: G::root(),
            villain,
            phase: if length > 0 {
                SubPhase::Prefix(0, length)
            } else {
                SubPhase::Meta
            },
        }
    }
    /// Returns the inner game state.
    pub fn inner(&self) -> G {
        self.inner
    }
    /// Returns the current subgame phase.
    pub fn phase(&self) -> SubPhase {
        self.phase
    }
    /// Returns the selected world index, if in Real phase.
    pub fn world(&self) -> Option<usize> {
        match self.phase {
            SubPhase::Real(w) => Some(w),
            _ => None,
        }
    }
    /// Returns available alternative indices as edges.
    pub fn alternative_edges(&self) -> Vec<SubEdge<G::E>> {
        (0..SUBGAME_ALTS).map(SubEdge::World).collect()
    }
}

impl<G> CfrGame for SubGame<G>
where
    G: CfrGame,
{
    type E = SubEdge<G::E>;
    type T = SubTurn<G::T>;
    /// Returns the subgame root (opponent to select alternative).
    fn root() -> Self {
        panic!("SubGame::root() should not be called; use SubGame::new() instead")
    }
    /// Returns whose turn it is.
    ///
    /// During `Prefix` phase, delegates to inner game (replaying history).
    /// During `MetaGame` phase, the opponent acts.
    /// During `RealGame` phase, delegates to inner game.
    fn turn(&self) -> Self::T {
        match self.phase {
            SubPhase::Prefix(..) => SubTurn::Natural(self.inner.turn()),
            SubPhase::Meta => SubTurn::Adverse(self.villain),
            SubPhase::Real(_) => SubTurn::Natural(self.inner.turn()),
        }
    }
    /// Applies an action to transition to the next state.
    fn apply(&self, edge: Self::E) -> Self {
        match (self.phase, edge) {
            (SubPhase::Prefix(i, n), SubEdge::Inner(e)) => Self {
                inner: self.inner.apply(e),
                villain: self.villain,
                phase: if i + 1 >= n {
                    SubPhase::Meta
                } else {
                    SubPhase::Prefix(i + 1, n)
                },
            },
            (SubPhase::Meta, SubEdge::World(w)) => Self {
                inner: self.inner,
                villain: self.villain,
                phase: SubPhase::Real(w),
            },
            (SubPhase::Real(w), SubEdge::Inner(e)) => Self {
                inner: self.inner.apply(e),
                villain: self.villain,
                phase: SubPhase::Real(w),
            },
            _ => panic!("invalid edge for current phase"),
        }
    }
    /// Returns payoff at terminal nodes.
    ///
    /// Subgame phase nodes are never terminal; delegates to inner game.
    fn payoff(&self, turn: Self::T) -> Utility {
        match turn {
            SubTurn::Natural(t) => self.inner.payoff(t),
            SubTurn::Adverse(_) => panic!("subgame phase has no payoff"),
        }
    }
}
