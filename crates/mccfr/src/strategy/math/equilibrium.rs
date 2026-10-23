use crate::*;
use monge::Density;
use pokerkit::*;
use std::collections::HashMap;

impl<T> Equilibrium for T where T: Profile {}

/// Read-only queries over the averaged (historical weighted) strategy: Nash
/// approximation, frontier evaluation, best-response analysis.
pub trait Equilibrium: Profile {
    /// Historical average probability of a single edge.
    fn averaged_policy(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Probability {
        self.averaged_strategy(info).density(edge)
    }

    /// Distance from Nash: `(BR(P1) + BR(P2)) / 2` for two-player zero-sum,
    /// where `BR(Pi)` is i's best-response utility against the opponent's fixed
    /// average strategy. Zero at equilibrium.
    fn exploitability(&self, tree: Tree<Self::F>) -> Utility {
        let ref partition = tree.partition();
        (0..<TurnOf<Self>>::players())
            .map(<TurnOf<Self>>::from)
            .map(|i| self.optimal_response_payoff(partition, i))
            .sum::<Utility>()
            / <TurnOf<Self>>::players() as Utility
    }

    /// Expected value at an infoset, for frontier evaluation in depth-limited
    /// search and safe subgame solving. `payoff` is already an incremental
    /// (Welford) mean, so it needs no division by visits. EV is a property of
    /// the infoset, not the edge; it lives on the edge because regret, weight
    /// and visits already do.
    fn frontier_payoff(&self, info: &InfoOf<Self>) -> Utility {
        let edge = &info
            .menu()
            .next()
            .expect("has actions, all of which have the same EV! next is good enough");
        self.cum_payoff(info, edge)
    }

    /// Leaf node value: payoff for terminals, frontier EV otherwise.
    ///
    /// decision ancestor's V(I), which is populated during blueprint training. Walks up past
    /// consecutive chance nodes (e.g. multi-street runouts).
    fn terminal_value(&self, node: &Node<Self::F>, hero: TurnOf<Self>) -> Utility {
        let turn = node.game().turn();
        if turn.is_terminal() {
            node.game().payoff(hero)
        } else if turn.is_chance() {
            node.clone()
                .find(|a| !a.node().game().turn().is_chance())
                .map(|a| a.node().info())
                .map(|info| self.frontier_payoff(info))
                .expect("chance node cannot be root of game")
        } else {
            self.frontier_payoff(node.info())
        }
    }

    /// Expected value of an opponent-controlled subtree: recursive *descent*
    /// weighting by the opponent's averaged strategy at each external node.
    fn external_payoff(&self, node: &Node<Self::F>, hero: TurnOf<Self>) -> Utility {
        self.subgamed_payoff(node, hero, None)
    }

    /// Recursive expected value using precomputed best response actions.
    fn response_payoff(
        &self,
        node: &Node<Self::F>,
        hero: TurnOf<Self>,
        br: &HashMap<InfoOf<Self>, EdgeOf<Self>>,
    ) -> Utility {
        self.subgamed_payoff(node, hero, Some(br))
    }

    /// Recursive expected value. With `br` as `None` everyone plays the average
    /// strategy; with `Some`, hero follows the BR actions and opponents don't.
    fn subgamed_payoff(
        &self,
        node: &Node<Self::F>,
        hero: TurnOf<Self>,
        br: Option<&HashMap<InfoOf<Self>, EdgeOf<Self>>>,
    ) -> Utility {
        let n = node.width();
        let kids = node.children();
        let recurse = |x| self.subgamed_payoff(x, hero, br);
        match node.game().turn() {
            _ if n == 0 => self.terminal_value(node, hero),
            t if t == <TurnOf<Self>>::chance() => kids.iter().map(recurse).sum::<Utility>() / n as Utility,
            t if t == hero => match br {
                Some(br) => br
                    .get(node.info())
                    .map(|e| node.step(e))
                    .map(|n| n.expect("hero unreachable without BR"))
                    .as_ref()
                    .map(recurse)
                    .expect("edge seen in BR"),
                None => kids
                    .iter()
                    .map(|x| self.averaged_policy(node.info(), x.incoming().expect("non-root")) * recurse(x))
                    .sum(),
            },
            _ => kids
                .iter()
                .map(|x| self.averaged_policy(node.info(), x.incoming().expect("non-root")) * recurse(x))
                .sum(),
        }
    }

    /// Product of opponent strategy probabilities along the path to `node`.
    ///
    /// Walks **upward** via parent pointers; dual of
    /// [`Solver::external_reach`](crate::Solver::external_reach),
    /// which walks **downward** replaying edges forward.
    fn external_reach(&self, node: &Node<Self::F>, hero: TurnOf<Self>) -> Probability {
        node.decisions()
            .filter(|choice| choice.turn() != hero)
            .map(|choice| self.averaged_policy(choice.info(), choice.edge()))
            .product::<Probability>()
    }

    /// Best response value for `hero` against opponents' average strategy.
    ///
    /// Picks one action per *info set*, not per node, so the response respects
    /// the information structure.
    fn optimal_response_payoff(
        &self,
        partition: &FxHashMap<InfoOf<Self>, InfoSet<Self::F>>,
        hero: TurnOf<Self>,
    ) -> Utility {
        let root = &partition
            .values()
            .next()
            .expect("partition")
            .tree()
            .at(petgraph::graph::NodeIndex::new(0));
        let response = &partition
            .iter()
            .filter(|(_, infoset)| infoset.head().game().turn() == hero)
            .map(|(info, infoset)| (*info, self.optimal_cfactual_choice(infoset, hero)))
            .collect::<HashMap<_, _>>();
        self.response_payoff(root, hero, response)
    }

    /// Counterfactual value of an edge in an info set.
    fn optimal_cfactual_payoff(&self, infoset: &InfoSet<Self::F>, edge: &EdgeOf<Self>, hero: &TurnOf<Self>) -> Utility {
        infoset
            .span()
            .iter()
            .filter_map(|n| n.step(edge))
            .map(|c| self.external_reach(&c, *hero) * self.external_payoff(&c, *hero))
            .sum()
    }

    /// Best action at an info set: argmax over actions of counterfactual value.
    fn optimal_cfactual_choice(&self, infoset: &InfoSet<Self::F>, hero: TurnOf<Self>) -> EdgeOf<Self> {
        infoset
            .info()
            .menu()
            .map(|edge| (edge, self.optimal_cfactual_payoff(infoset, &edge, &hero)))
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("good values"))
            .expect("info set has actions")
            .0
    }
}
