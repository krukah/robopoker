//! The canonical abstract-space walk.
use crate::*;

/// A walk whose moves are the game's own edges — the abstract-space
/// [`Walk`], and the one every game gets for free.
///
/// Games with a lossless move type of their own (NLHE's `kicker::Action`)
/// implement [`Walk`] on their history types directly; `Line` is what you
/// reach for when the edge sequence is all there is.
#[derive(Clone)]
pub struct Line<G>
where
    G: CfrGame,
{
    root: G,
    moves: Vec<G::E>,
}

impl<G> Default for Line<G>
where
    G: CfrGame,
{
    fn default() -> Self {
        Self::new(G::root(), Vec::new())
    }
}

impl<G> Line<G>
where
    G: CfrGame,
{
    pub fn new(root: G, moves: Vec<G::E>) -> Self {
        Self { root, moves }
    }

    pub fn push(&mut self, edge: G::E) {
        self.moves.push(edge);
    }
}

impl<G> FromIterator<G::E> for Line<G>
where
    G: CfrGame,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = G::E>,
    {
        Self::new(G::root(), iter.into_iter().collect())
    }
}

impl<G> Walk for Line<G>
where
    G: CfrGame,
{
    type G = G;
    type Exact = G::E;

    fn root(&self) -> G {
        self.root
    }

    fn moves(&self) -> &[G::E] {
        &self.moves
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use monge::Support;
    use pokerkit::Utility;

    /// Three-variant turn; the walk turns terminal at depth 3, and depth 2
    /// is a chance node so `current_street` has a boundary to find.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    enum T {
        Choice,
        Chance,
        Terminal,
    }

    impl From<usize> for T {
        fn from(_: usize) -> Self {
            T::Choice
        }
    }

    impl CfrTurn for T {
        fn chance() -> Self {
            T::Chance
        }

        fn terminal() -> Self {
            T::Terminal
        }

        fn players() -> usize {
            2
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct E(u8);

    impl Support for E {}

    impl CfrEdge for E {}

    /// A counter game: state is depth, every edge advances by one.
    #[derive(Copy, Clone, Debug, PartialEq)]
    struct G(u8);

    impl CfrGame for G {
        type E = E;
        type T = T;

        fn root() -> Self {
            G(0)
        }

        fn turn(&self) -> T {
            match self.0 {
                2 => T::Chance,
                d if d >= 4 => T::Terminal,
                _ => T::Choice,
            }
        }

        fn apply(&self, _: E) -> Self {
            G(self.0 + 1)
        }

        fn payoff(&self, _: T) -> Utility {
            Utility::from(self.0)
        }
    }

    fn line() -> Line<G> {
        Line::new(G::root(), vec![E(0), E(1), E(2), E(3)])
    }

    #[test]
    fn states_bracket_the_moves() {
        assert_eq!(line().states(&mut Ident::default()), vec![G(0), G(1), G(2), G(3), G(4)]);
    }

    #[test]
    fn head_is_the_last_state() {
        assert_eq!(line().head(&mut Ident::default()), G(4));
    }

    #[test]
    fn ident_coarsening_is_the_move_list() {
        assert_eq!(line().edges(&mut Ident::default()), vec![E(0), E(1), E(2), E(3)]);
    }

    /// The law: refine is a section of coarsen.
    #[test]
    fn refine_then_coarsen_is_identity() {
        let ref mut lens = Ident::default();
        let refined = lens.refine(&G::root(), E(7));
        assert_eq!(lens.coarsen(&G::root(), refined), pokerkit::Translated::Snap(E(7)));
    }

    /// The bridge: a Walk materializes into the Descent vocabulary, so the
    /// existing DescentStream projections apply to it unchanged.
    #[test]
    fn descents_feed_descent_stream() {
        assert_eq!(
            line().descents(&mut Ident::default()).as_slice(),
            &[
                Descent(T::Choice, E(0)),
                Descent(T::Choice, E(1)),
                Descent(T::Chance, E(2)),
                Descent(T::Choice, E(3)),
            ]
        );
        assert_eq!(line().descents(&mut Ident::default()).current_street(), vec![E(3)]);
    }
}
