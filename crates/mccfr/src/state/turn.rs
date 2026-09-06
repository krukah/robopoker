/// Classifies a game tree node as a player decision, a chance node, or a
/// terminal — the three cases CFR traversal treats differently.
///
/// `From<usize>` maps a player index to its turn (0 = P1, 1 = P2, …).
pub trait CfrTurn:
    Clone + Copy + PartialEq + Eq + Send + Sync + std::fmt::Debug + std::hash::Hash + From<usize>
{
    /// The chance variant: card deals, dice rolls.
    fn chance() -> Self;
    /// The terminal variant: game over, payoffs available.
    fn terminal() -> Self;
    fn players() -> usize;

    fn is_chance(&self) -> bool {
        &Self::chance() == self
    }

    fn is_terminal(&self) -> bool {
        &Self::terminal() == self
    }

    fn is_opponent(&self, hero: &Self) -> bool {
        self != hero && !self.is_chance() && !self.is_terminal()
    }
}
