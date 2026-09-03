//! Game state for off-tree-augmented (nested) subgames.
use super::*;
use mccfr::*;
use pokerkit::Utility;

/// Wraps an [`Augmentable`] game with an entry flag and the off-tree payload
/// to splice. `entry` is carried as state (not derived from the path) because
/// under the world/depth solver composition the encoder never gets a per-node
/// seed hook — it's set when the nesting solver builds the entry game and by
/// each per-world restrict, cleared by every `apply`. See insight #1.
///
/// `derive` handles `Copy`/`Clone`/`Debug`: even though `off: G::Off` is an
/// associated type, the struct's `where G: Augmentable` clause propagates into
/// the derived impls, and `G: Augmentable` makes `G::Off: Copy` (via
/// [`OffPayload`]) known — so the field bounds are satisfied.
#[derive(Debug, Clone, Copy)]
pub struct NestGame<G>
where
    G: Augmentable,
{
    inner: G,
    off: G::Off,
    entry: bool,
}

impl<G> NestGame<G>
where
    G: Augmentable,
{
    /// A non-entry node carrying the (already-committed) off-tree payload.
    pub fn new(inner: G, off: G::Off) -> Self {
        Self {
            inner,
            off,
            entry: false,
        }
    }
    /// The entry node — the one decision point whose menu is augmented.
    pub fn entry(inner: G, off: G::Off) -> Self {
        Self {
            inner,
            off,
            entry: true,
        }
    }
    /// The wrapped canonical game.
    pub fn inner(&self) -> &G {
        &self.inner
    }
    /// The off-tree payload threaded through this subgame.
    pub fn off(&self) -> G::Off {
        self.off
    }
    /// Whether this node is the augmented entry.
    pub fn is_entry(&self) -> bool {
        self.entry
    }
}

impl<G> CfrGame for NestGame<G>
where
    G: Augmentable,
{
    type E = NestEdge<G::E, G::Off>;
    type T = G::T;

    fn root() -> Self {
        unreachable!("NestGame must be constructed via new()/entry() with an off-tree payload")
    }

    fn turn(&self) -> Self::T {
        self.inner.turn()
    }

    fn apply(&self, edge: Self::E) -> Self {
        let inner = match edge {
            NestEdge::Game(e) => self.inner.apply(e),
            NestEdge::Off(o) => self.inner.augment(o),
        };
        Self {
            inner,
            off: self.off,
            entry: false,
        }
    }

    fn payoff(&self, turn: Self::T) -> Utility {
        self.inner.payoff(turn)
    }

    fn depth(&self) -> usize {
        self.inner.depth()
    }
}
