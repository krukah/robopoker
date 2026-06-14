//! Integration tests for depth-limited frontier evaluation.
#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::*;
    use fulcrum::*;
    use monge::Support;
    use regret::*;

    const D: usize = 4;

    // ── minimal 2-street game ────────────────────────────────────────

    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
    enum MiniTurn {
        P0,
        P1,
        Chance,
        Terminal,
    }
    impl CfrTurn for MiniTurn {
        fn chance() -> Self {
            Self::Chance
        }

        fn terminal() -> Self {
            Self::Terminal
        }

        fn players() -> usize {
            2
        }
    }
    impl From<usize> for MiniTurn {
        fn from(i: usize) -> Self {
            match i % 2 {
                0 => Self::P0,
                _ => Self::P1,
            }
        }
    }

    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    enum MiniEdge {
        Bet,
        Check,
    }
    impl Support for MiniEdge {}
    impl CfrEdge for MiniEdge {}

    #[derive(Debug, Clone, Copy)]
    struct MiniGame {
        street: usize,
        actions: usize,
    }
    impl CfrGame for MiniGame {
        type E = MiniEdge;
        type T = MiniTurn;

        fn root() -> Self {
            Self { street: 0, actions: 0 }
        }

        fn turn(&self) -> Self::T {
            if self.street >= 2 {
                MiniTurn::Terminal
            } else if self.actions >= 2 {
                MiniTurn::Chance
            } else if self.actions.is_multiple_of(2) {
                MiniTurn::P0
            } else {
                MiniTurn::P1
            }
        }

        fn apply(&self, _edge: Self::E) -> Self {
            if self.turn() == MiniTurn::Chance {
                Self {
                    street: self.street + 1,
                    actions: 0,
                }
            } else {
                Self {
                    street: self.street,
                    actions: self.actions + 1,
                }
            }
        }

        fn payoff(&self, turn: Self::T) -> Utility {
            match turn {
                MiniTurn::P0 => 1.0,
                MiniTurn::P1 => -1.0,
                _ => 0.0,
            }
        }

        fn depth(&self) -> usize {
            if self.turn() == MiniTurn::Chance { self.street + 1 } else { self.street }
        }
    }

    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    struct MiniInfo {
        street: usize,
        actions: usize,
    }
    impl CfrInfo for MiniInfo {
        type E = MiniEdge;
        type T = MiniTurn;
        type X = MiniInfo;
        type Y = ();

        fn public(&self) -> Self::X {
            *self
        }

        fn secret(&self) -> Self::Y {}
    }
    impl CfrPublic for MiniInfo {
        type E = MiniEdge;
        type T = MiniTurn;

        fn choices(&self) -> impl Iterator<Item = Self::E> + use<> {
            [MiniEdge::Bet, MiniEdge::Check].into_iter()
        }

        fn subgame(&self) -> Vec<Self::E> {
            vec![]
        }
    }

    struct MiniEncoder;
    impl CfrEncoder for MiniEncoder {
        type T = MiniTurn;
        type E = MiniEdge;
        type G = MiniGame;
        type I = MiniInfo;

        fn seed(&self, game: &Self::G) -> Self::I {
            MiniInfo {
                street: game.street,
                actions: game.actions,
            }
        }

        fn info(
            &self,
            _tree: &Tree<Self::T, Self::E, Self::G, Self::I>,
            (_edge, game, _parent): Leaf<Self::E, Self::G>,
        ) -> Self::I {
            self.seed(&game)
        }

        fn resume<P>(&self, _past: P, game: &Self::G) -> Self::I
        where
            P: IntoIterator<Item = Self::E>,
        {
            self.seed(game)
        }
    }

    // ── minimal profiles ─────────────────────────────────────────────

    struct MiniBlueprint;
    impl CfrRule for MiniBlueprint {
        type T = MiniTurn;
        type E = MiniEdge;
        type G = MiniGame;
        type I = MiniInfo;
    }
    impl RefProf for MiniBlueprint {
        fn t(&self) -> usize {
            1
        }

        fn sum_regret(&self) -> Utility {
            0.0
        }

        fn cum_weight(&self, _: &Self::I, _: &Self::E) -> Probability {
            1.0
        }

        fn cum_regret(&self, _: &Self::I, _: &Self::E) -> Utility {
            1.0
        }

        fn cum_payoff(&self, _: &Self::I, _: &Self::E) -> Utility {
            0.5
        }

        fn cum_visits(&self, _: &Self::I, _: &Self::E) -> u32 {
            1
        }
    }

    /// Bundles encoder + blueprint for depth-limited solving tests.
    struct MiniSolver(MiniEncoder, MiniBlueprint);
    impl CfrEncoder for MiniSolver {
        type T = MiniTurn;
        type E = MiniEdge;
        type G = MiniGame;
        type I = MiniInfo;

        fn seed(&self, game: &Self::G) -> Self::I {
            self.0.seed(game)
        }

        fn info(&self, tree: &Tree<Self::T, Self::E, Self::G, Self::I>, branch: Leaf<Self::E, Self::G>) -> Self::I {
            self.0.info(tree, branch)
        }

        fn resume<P>(&self, past: P, game: &Self::G) -> Self::I
        where
            P: IntoIterator<Item = Self::E>,
        {
            self.0.resume(past, game)
        }
    }
    impl<const N: usize> DepthSampler<N> for MiniSolver {
        type Blueprint = MiniBlueprint;

        fn blueprint(&self) -> &MiniBlueprint {
            &self.1
        }

        fn payoffs(&self, prefix: &Prefix<Self::T, Self::E>, game: &Self::G, _: Self::T) -> Payoffs<N> {
            Payoffs::uniform(self.1.frontier_payoff(&self.resume(prefix.into_iter().edges(), game)))
        }
    }

    struct LeafMiniProfile;
    impl CfrRule for LeafMiniProfile {
        type T = MiniTurn;
        type E = DepthEdge<MiniEdge, D>;
        type G = DepthGame<MiniGame, D>;
        type I = DepthInfo<MiniInfo, D>;
    }
    impl RefProf for LeafMiniProfile {
        fn t(&self) -> usize {
            1
        }

        fn sum_regret(&self) -> Utility {
            0.0
        }

        fn cum_weight(&self, _: &Self::I, _: &Self::E) -> Probability {
            1.0
        }

        fn cum_regret(&self, _: &Self::I, _: &Self::E) -> Utility {
            1.0
        }

        fn cum_payoff(&self, _: &Self::I, _: &Self::E) -> Utility {
            0.5
        }

        fn cum_visits(&self, _: &Self::I, _: &Self::E) -> u32 {
            1
        }
    }
    impl CfrSampling for LeafMiniProfile {
        fn increment(&mut self) {}
        fn walker(&self) -> Self::T {
            MiniTurn::P0
        }
    }

    // ── tests ────────────────────────────────────────────────────────

    #[test]
    fn leaf_encoder_produces_pick_branches_at_leaf() {
        let encoder = DepthEncoder::<_, D>::new(&MiniSolver(MiniEncoder, MiniBlueprint), vec![]);
        let profile = LeafMiniProfile;
        let root: DepthGame<_, D> = DepthGame::new(MiniGame::root(), MiniTurn::P0, Some(0));
        let tree = TreeBuilder::<_, _, _, _, _, _, ExternalSampling>::new(&encoder, &profile, root, 0).build();
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
        let encoder = DepthEncoder::<_, D>::new(&MiniSolver(MiniEncoder, MiniBlueprint), vec![]);
        let profile = LeafMiniProfile;
        let root: DepthGame<_, D> = DepthGame::new(MiniGame::root(), MiniTurn::P0, Some(0));
        let tree = TreeBuilder::<_, _, _, _, _, _, ExternalSampling>::new(&encoder, &profile, root, 0).build();
        let root_node = tree.at(petgraph::graph::NodeIndex::new(0));
        assert_eq!(root_node.width(), 2, "root should have 2 game branches");
    }
}
