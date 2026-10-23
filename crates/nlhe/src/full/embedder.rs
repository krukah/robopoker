use crate::*;
use mccfr::*;
use neural::*;

/// [`FullInfo`]'s own [`Block`] as the input row; the full bet grid as outputs, at each edge's
/// [`NlheEdge::index`].
#[derive(Debug, Default, Clone, Copy)]
pub struct NlheEmbedder;

impl CfrSpec for NlheEmbedder {
    type F = FullInfo;
}

impl Embedder for NlheEmbedder {
    fn inputs(&self) -> usize {
        FullInfo::WIDTH
    }

    fn outputs(&self) -> usize {
        NlheEdge::CHOICES
    }

    fn features(&self, info: &FullInfo) -> Features {
        info.values().collect()
    }

    fn turn(&self, info: &FullInfo) -> NlheTurn {
        NlheTurn::from(info.public().turn())
    }

    fn index(&self, edge: &NlheEdge) -> usize {
        edge.index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deuce::*;
    use kicker::*;
    use pokerkit::*;

    impl NlheEmbedder {
        fn infos() -> Vec<(Game, FullInfo)> {
            (0..1 << 9)
                .map(|_| Witness::from(Street::random()))
                .flat_map(|witness| {
                    witness
                        .walk()
                        .into_iter()
                        .filter(|stride| stride.turn().is_choice())
                        .zip(FullEncoder.retrace(&witness))
                })
                .map(|(stride, choice)| (stride.game(), choice.info().clone()))
                .collect()
        }
    }

    #[test]
    fn rows_fill_the_inputs() {
        NlheEmbedder::infos().into_iter().for_each(|(_, info)| {
            let row = NlheEmbedder.features(&info).into_iter().collect::<Vec<_>>();
            assert_eq!(row.len(), NlheEmbedder.inputs(), "{info:?}");
            assert!(row.iter().all(|x| x.is_finite() && (-1. ..=1.).contains(x)), "{row:?}");
        });
    }

    #[test]
    fn turns_agree_with_the_game() {
        NlheEmbedder::infos()
            .into_iter()
            .for_each(|(game, info)| assert_eq!(NlheEmbedder.turn(&info), NlheTurn::from(game.turn())));
    }

    #[test]
    fn predicates_cover_the_vocabulary() {
        assert_eq!(FullSecret::WIDTH, 4 * (52 + 13 + 4) + rosetta::Feature::all().len());
    }

    #[test]
    fn indices_cover_the_grid() {
        let edges = [Edge::Fold, Edge::Check, Edge::Call, Edge::Shove]
            .into_iter()
            .chain(OPENS.iter().map(|&n| Edge::Open(n)))
            .chain(RAISES.iter().map(|&(n, d)| Edge::Raise(Odds::new(n, d))))
            .map(NlheEdge::from)
            .map(|edge| NlheEmbedder.index(&edge))
            .collect::<Vec<_>>();
        assert_eq!(edges, (0..NlheEmbedder.outputs()).collect::<Vec<_>>());
    }

    #[test]
    fn longest_street_fits() {
        let escalate = |mut witness: Witness| {
            let street = witness.street();
            while witness.head().turn().is_choice() && witness.street() == street {
                let head = witness.head();
                let choices = witness.choices().into_iter().collect::<Vec<_>>();
                let edge = choices
                    .iter()
                    .copied()
                    .filter(Edge::is_raise)
                    .last()
                    .or_else(|| choices.iter().copied().find(Edge::is_shove))
                    .or_else(|| choices.iter().copied().find(|edge| matches!(edge, Edge::Call)))
                    .unwrap_or(Edge::Check);
                witness = witness.push(head.snap(head.actionize(edge)));
            }
            witness
                .plays()
                .into_iter()
                .filter(|(_, _, played)| *played == street)
                .count()
        };
        let longest = Street::all()
            .into_iter()
            .flat_map(|street| (0..1 << 6).map(move |_| Witness::from(street)))
            .map(escalate)
            .max()
            .unwrap_or_default();
        println!("longest street: {longest} edges, {} slots", History::SLOTS);
        assert!(longest <= History::SLOTS);
    }

    #[test]
    #[ignore = "minutes in debug; run in release"]
    fn deep_cfr_fits_full_infosets() {
        let solver = DeepCfr::<FullEncoder, CfrNet<NlheEmbedder>, ExternalSampling>::default().solve(2);
        assert!(solver.tree(0).n() > 1);
    }
}
