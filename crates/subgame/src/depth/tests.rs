//! Integration tests for depth-limited frontier evaluation.
#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::depth::fixture::*;
    use crate::*;
    use mccfr::*;

    const D: usize = 4;

    #[test]
    fn info_count_matches_choices() {
        let info = MiniInfo::from(MiniGame::root());
        assert_eq!(CfrPublic::count(&info), CfrInfo::menu(&info).count());
        [
            info.game::<D>(),
            DepthInfo::<MiniInfo, D>::new(DepthPublic::Pick(info), ()),
        ]
        .iter()
        .for_each(|i| assert_eq!(i.public().count(), i.menu().count(), "{i:?}"));
    }

    #[test]
    fn leaf_encoder_produces_pick_branches_at_leaf() {
        let encoder = DepthEncoder::<_, D>::new(&MiniSolver(MiniEncoder, MiniBlueprint), Prefix::default());
        let profile = LeafMiniProfile;
        let root: DepthGame<_, D> = DepthGame::new(MiniGame::root(), MiniTurn::P0, Some(0));
        let tree = TreeBuilder::<_, _, ExternalSampling>::new(&encoder, &profile, root, 0).build();
        assert!(tree.n() > 4, "tree should have frontier expansion: {} nodes", tree.n());
        let terminals = (0..tree.n())
            .map(|i| tree.at(petgraph::graph::NodeIndex::new(i)))
            .filter(|n| n.game().turn().is_terminal())
            .count();
        assert!(terminals > 0, "should have terminal nodes from frontier resolution");
    }

    #[test]
    fn leaf_game_phase_machine() {
        let game: DepthGame<_, D> = DepthGame::new(MiniGame::root(), MiniTurn::P0, Some(0));
        assert!(matches!(game.phase(), DepthPhase::Delegate));
        assert_eq!(game.turn(), MiniTurn::P0);
        let g1 = game.apply(DepthEdge::Game(MiniEdge::Bet));
        assert!(matches!(g1.phase(), DepthPhase::Delegate));
        assert_eq!(g1.turn(), MiniTurn::P1);
        let g2 = g1.apply(DepthEdge::Game(MiniEdge::Check));
        assert!(g2.at_frontier(), "should be at frontier after both act");
        let conts: Vec<_> = Continuation::all::<D>().collect();
        let payoffs = Payoffs::uniform(0.5);
        let g3 = g2.to_frontier(payoffs);
        assert!(matches!(g3.phase(), DepthPhase::Frontier(_)));
        assert_eq!(g3.turn(), MiniTurn::P0);
        let g4 = g3.apply(DepthEdge::Pick(conts[0]));
        assert!(matches!(g4.phase(), DepthPhase::Internal(_, _)));
        assert_eq!(g4.turn(), MiniTurn::P1);
        let g5 = g4.apply(DepthEdge::Pick(conts[1]));
        assert!(matches!(g5.phase(), DepthPhase::External(_, _, _)));
        assert_eq!(g5.turn(), MiniTurn::Terminal);
        assert_eq!(g5.payoff(MiniTurn::P0), 0.5);
        assert_eq!(g5.payoff(MiniTurn::P1), -0.5);
    }

    #[test]
    fn non_leaf_nodes_produce_game_branches() {
        let encoder = DepthEncoder::<_, D>::new(&MiniSolver(MiniEncoder, MiniBlueprint), Prefix::default());
        let profile = LeafMiniProfile;
        let root: DepthGame<_, D> = DepthGame::new(MiniGame::root(), MiniTurn::P0, Some(0));
        let tree = TreeBuilder::<_, _, ExternalSampling>::new(&encoder, &profile, root, 0).build();
        let root_node = tree.at(petgraph::graph::NodeIndex::new(0));
        assert_eq!(root_node.width(), 2, "root should have 2 game branches");
    }
}
