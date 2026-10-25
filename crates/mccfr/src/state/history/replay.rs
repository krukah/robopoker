//! The full play-through from the game root.
use crate::*;

/// The full play-through from game root to "now", spanning any chance
/// transitions between streets. A [`Trail`] tagged [`Fresh`].
pub type Replay<G> = Trail<G, Fresh>;

impl<G> Replay<G>
where
    G: CfrGame,
{
    /// The descents of an edge sequence played from the game root. Chance resolves through
    /// [`CfrGame::apply`], so a lossy chance edge deals at random.
    pub fn trace<I>(edges: I) -> Self
    where
        I: IntoIterator<Item = G::E>,
    {
        edges
            .into_iter()
            .scan(G::root(), |game, edge| {
                let descent = Descent(game.turn(), edge);
                *game = game.apply(edge);
                Some(descent)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::state::fixture::*;
    use crate::*;

    #[test]
    fn trace_pairs_each_edge_with_its_turn() {
        let replay = Replay::<G>::trace([E(0), E(1), E(2), E(3)]);
        assert_eq!(
            replay.as_slice(),
            &[
                Descent(T::Choice, E(0)),
                Descent(T::Choice, E(1)),
                Descent(T::Chance, E(2)),
                Descent(T::Choice, E(3))
            ]
        );
        assert_eq!(replay.current_street(), vec![E(3)]);
    }
}
