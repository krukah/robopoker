//! The NLHE bet grid as an approximation of the pure game.
use super::*;
use kicker::*;
use mccfr::*;

/// Chips coarsen to the nearest bet-grid edge under the process-wide [`pokerkit::Translation`];
/// dealt cards coarsen to [`Edge::Draw`]. The grid depends on trailing aggression, which this
/// counts from the path rather than storing in the state.
pub struct NlheGrid;

impl NlheGrid {
    /// The current street's descents of a recorded hand.
    pub fn subgame<R>(recall: &R) -> Prefix<NlheGame>
    where
        R: Recall,
    {
        Self::descents(NlheRules::from(recall.root()), recall.actions().iter().copied())
            .street()
            .into_iter()
            .collect()
    }
}

impl Approximation for NlheGrid {
    type R = NlheRules;
    type E = NlheEdge;

    fn coarsen<P>(r: &NlheRules, past: P, m: Action) -> NlheEdge
    where
        P: IntoIterator<Item = NlheEdge>,
    {
        let depth = past
            .into_iter()
            .take_while(NlheEdge::is_choice)
            .filter(NlheEdge::is_aggro)
            .count();
        NlheEdge::from(
            r.as_ref()
                .translate(m, depth, &pokerkit::Translation::current(), &mut rand::rng())
                .snapped(),
        )
    }

    fn refine(r: &NlheRules, e: NlheEdge) -> Action {
        let game = r.as_ref();
        if e.is_chance() {
            game.reveal(&mut rand::rng())
        } else {
            game.snap(game.actionize(Edge::from(e)))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use deuce::Street;
    use kicker::*;
    use mccfr::*;
    use pokerkit::Arbitrary;

    impl NlheGrid {
        /// Random on-grid hands, plus one with a raise war on the flop.
        fn hands() -> Vec<Witness> {
            let witness = Witness::initial(Game::root().dealer())
                .push(Action::Call(1))
                .push(Action::Check);
            let flop = witness.head().deck().deal(Street::Pref, &mut rand::rng());
            let raised = witness
                .push(Action::Draw(flop))
                .push(Action::Raise(6))
                .push(Action::Raise(18))
                .push(Action::Call(12));
            (0..1 << 8)
                .map(|_| Witness::from(Street::random()))
                .chain(std::iter::once(raised))
                .collect()
        }
    }

    #[test]
    fn descents_match_recall_history() {
        NlheGrid::hands().into_iter().for_each(|witness| {
            assert_eq!(
                NlheGrid::descents(NlheRules::from(witness.root()), witness.actions().iter().copied())
                    .into_iter()
                    .map(|d| d.1)
                    .collect::<Vec<_>>(),
                witness.history().into_iter().map(NlheEdge::from).collect::<Vec<_>>()
            );
        });
    }

    #[test]
    fn subgame_matches_recall_subgame() {
        NlheGrid::hands().into_iter().for_each(|witness| {
            assert_eq!(
                NlheGrid::subgame(&witness).iter().map(|d| d.1).collect::<Vec<_>>(),
                witness.subgame().into_iter().map(NlheEdge::from).collect::<Vec<_>>()
            );
        });
    }

    /// The law along each hand's own path, for edges whose chips were legal as given: the
    /// min-raise lift and the stack cap are where NLHE's grid is not a congruence.
    #[test]
    fn refine_then_coarsen_is_identity() {
        NlheGrid::hands()
            .into_iter()
            .filter(|w| w.head().turn() != Turn::Terminal && w.head().turn() != Turn::Chance)
            .for_each(|witness| {
                let rules = NlheRules::from(witness.head());
                let past = NlheGrid::descents(NlheRules::from(witness.root()), witness.actions().iter().copied());
                let game = witness.head();
                witness
                    .choices()
                    .into_iter()
                    .filter(|edge| edge.is_choice() && game.snap(game.actionize(*edge)) == game.actionize(*edge))
                    .map(NlheEdge::from)
                    .for_each(|edge| {
                        let exact = NlheGrid::refine(&rules, edge);
                        assert_eq!(
                            NlheGrid::coarsen(&rules, past.iter().rev().map(|d| d.1), exact),
                            edge,
                            "{edge:?} → {exact:?}"
                        );
                    });
            });
    }
}
