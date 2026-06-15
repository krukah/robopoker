//! Game state augmented with frontier continuation choices.
//!
//! At depth-limited frontier leaves (street boundaries), instead of
//! returning a single blueprint EV, the game enters the L×L continuation
//! normal-form game:
//!
//! ```text
//! Frontier chance node
//!   DepthPhase::Frontier(payoffs)          — entered frontier, 0 picks
//!     DepthPhase::Internal(payoffs, k)     — internal picked k
//!       DepthPhase::External(payoffs, k, j) — external picked j, resolved
//! ```
use super::*;
use mccfr::*;
use pokerkit::Utility;

#[derive(Debug, Clone, Copy)]
pub struct DepthGame<G, const D: usize>
where
    G: CfrGame,
{
    inner: G,
    origin: Option<usize>,
    internal: G::T,
    phase: DepthPhase<D>,
}

impl<G, const D: usize> DepthGame<G, D>
where
    G: CfrGame,
{
    pub fn new(inner: G, internal: G::T, origin: Option<usize>) -> Self {
        Self {
            inner,
            origin,
            internal,
            phase: DepthPhase::Delegate,
        }
    }

    pub fn inner(&self) -> &G {
        &self.inner
    }

    pub fn phase(&self) -> &DepthPhase<D> {
        &self.phase
    }

    pub fn origin(&self) -> Option<usize> {
        self.origin
    }

    pub fn internal(&self) -> G::T {
        self.internal
    }
    /// The other player (derived from `internal` in a 2-player game).
    fn external(&self) -> G::T {
        (0..G::T::players())
            .map(G::T::from)
            .filter(|t| !t.is_terminal() && !t.is_chance())
            .find(|t| t != &self.internal())
            .expect("two-player game")
    }
    /// Transition from Delegate to Frontier with computed payoffs.
    pub fn to_frontier(self, payoffs: Payoffs<D>) -> Self {
        Self {
            phase: DepthPhase::Frontier(payoffs),
            ..self
        }
    }
    /// True iff the base game is at a chance node beyond the origin depth.
    /// Returns `false` when `origin` is `None` (frontier detection disabled).
    pub fn at_frontier(&self) -> bool {
        matches!(self.phase, DepthPhase::Delegate)
            && self.inner.turn().is_chance()
            && self.origin.is_some_and(|o| self.inner.depth() > o)
    }
    /// True iff in the middle of the frontier normal-form game (not yet resolved).
    pub fn is_choosing(&self) -> bool {
        matches!(self.phase, DepthPhase::Frontier(_) | DepthPhase::Internal(_, _))
    }
}

impl<G, const D: usize> CfrGame for DepthGame<G, D>
where
    G: CfrGame,
{
    type E = DepthEdge<G::E, D>;
    type T = G::T;

    fn root() -> Self {
        unreachable!("DepthGame must be constructed via new() with an explicit internal player")
    }

    fn turn(&self) -> Self::T {
        match &self.phase {
            DepthPhase::Delegate => self.inner.turn(),
            DepthPhase::Frontier(_) => self.internal(),
            DepthPhase::Internal(_, _) => self.external(),
            DepthPhase::External(_, _, _) => G::T::terminal(),
        }
    }

    fn apply(&self, edge: Self::E) -> Self {
        match (&self.phase, edge) {
            (DepthPhase::Delegate, DepthEdge::Game(e)) => Self {
                inner: self.inner.apply(e),
                ..*self
            },
            (DepthPhase::Frontier(payoffs), DepthEdge::Pick(k)) => Self {
                phase: DepthPhase::Internal(*payoffs, k),
                ..*self
            },
            (DepthPhase::Internal(payoffs, k), DepthEdge::Pick(j)) => Self {
                phase: DepthPhase::External(*payoffs, *k, j),
                ..*self
            },
            _ => unreachable!("invalid frontier transition: {:?} + {:?}", self.phase(), edge),
        }
    }

    fn payoff(&self, turn: Self::T) -> Utility {
        match &self.phase {
            DepthPhase::Delegate => self.inner.payoff(turn),
            DepthPhase::External(payoffs, k, j) => {
                let val = payoffs.get(*k, *j);
                if turn == self.internal() { val } else { -val }
            }
            _ => unreachable!("payoff called in unresolved phase"),
        }
    }

    fn depth(&self) -> usize {
        self.inner.depth()
    }

    fn is_frontier(&self) -> bool {
        self.at_frontier()
    }
}
