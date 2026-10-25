use crate::*;
use mccfr::*;

/// Encodes NLHE nodes as [`FullInfo`]: no tables, so Deep CFR on NLHE trains without a database.
#[derive(Debug, Default, Clone, Copy)]
pub struct FullEncoder;

impl CfrSpec for FullEncoder {
    type F = FullInfo;
}

impl Encoding for FullEncoder {
    fn public<P>(&self, game: &NlheGame, past: P) -> FullPublic
    where
        P: IntoIterator<Item = NlheEdge>,
    {
        let game = game.as_ref();
        let history = past
            .into_iter()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<History>();
        let ledger = Ledger::new(game.pot(), game.to_call(), game.stacks(), game.effective());
        FullPublic::new(history, game.choices(history.aggression()), ledger, game.turn(), game.button())
    }

    fn secret(&self, game: &NlheGame) -> FullSecret {
        FullSecret::new(game.as_ref().arrangement().canonical())
    }
}

impl Retrace for FullEncoder {}

#[cfg(test)]
mod tests {
    use super::*;
    use deuce::*;
    use kicker::*;
    use neural::*;
    use pokerkit::Arbitrary;

    #[test]
    fn retrace_agrees_with_the_game() {
        (0..1 << 10)
            .map(|_| Witness::from(Street::random()))
            .for_each(|witness| {
                witness
                    .walk()
                    .into_iter()
                    .enumerate()
                    .filter(|(_, stride)| stride.turn().is_choice())
                    .zip(FullEncoder.retrace(&witness))
                    .for_each(|((depth, stride), choice)| {
                        let info = choice.info();
                        let game = stride.game();
                        let subgame = info
                            .public()
                            .history()
                            .edges()
                            .map(Edge::from)
                            .fold(Subgame::default(), Subgame::flow);
                        assert_eq!(info.public().history().length(), depth);
                        assert_eq!(info.street(), game.street());
                        assert_eq!(info.public().turn(), game.turn());
                        assert_eq!(
                            info.public().ledger(),
                            Ledger::new(game.pot(), game.to_call(), game.stacks(), game.effective())
                        );
                        assert!(
                            CfrInfo::menu(info)
                                .map(Edge::from)
                                .eq(game.choices(subgame.aggression()))
                        );
                    });
            });
    }

    #[test]
    fn cards_are_canonical_and_split_by_street() {
        (0..1 << 10)
            .map(|_| Witness::from(Street::random()))
            .for_each(|witness| {
                let cards = FullEncoder.secret(&NlheGame::from(witness.head())).cards();
                assert_eq!(cards.observation(), Observation::from(Isomorphism::from(witness.head().sweat())));
                assert_eq!(cards.street(), witness.street());
            });
    }

    #[test]
    fn deep_cfr_walks_full_infosets() {
        let solver = DeepCfr::<FullEncoder, MockNet<FullInfo>, ExternalSampling>::default().solve(2);
        let tree = solver.tree(0);
        println!("one traversal: {} nodes", tree.n());
        assert!(tree.partition().len() > 1);
    }
}
