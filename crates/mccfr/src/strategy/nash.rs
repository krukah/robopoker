use crate::*;
use rbp_core::*;
use rbp_transport::Density;
use std::collections::HashMap;

impl<T> CfrNash for T where T: RefProf {}

/// Read-only Nash strategy queries and exploitability analysis.
///
/// Provides the averaged (historical weighted) strategy distribution
/// used for Nash equilibrium approximation, frontier evaluation, and
/// best-response analysis. Blanket-implemented for all [`Profile`] types.
pub trait CfrNash: RefProf {
    /// Calculate historical average for a single edge.
    fn averaged_policy(&self, info: &Self::I, edge: &Self::E) -> Probability {
        self.averaged_distribution(info).density(edge)
    }

    /// Computes the exploitability of the current average strategy.
    ///
    /// Exploitability measures how far the strategy is from Nash equilibrium.
    /// For a two-player zero-sum game:
    ///
    /// `exploitability = (BR(P1) + BR(P2)) / 2`
    ///
    /// where `BR(Pi)` is the expected utility that player i can achieve by
    /// playing a best response against the opponent's fixed average strategy.
    ///
    /// A Nash equilibrium has exploitability of 0. Lower values indicate
    /// strategies closer to equilibrium.
    fn exploitability(&self, tree: Tree<Self::T, Self::E, Self::G, Self::I>) -> Utility {
        let ref partition = tree.partition();
        (0..Self::T::players())
            .map(Self::T::from)
            .map(|i| self.optimal_response_payoff(partition, i))
            .sum::<Utility>()
            / Self::T::players() as Utility
    }

    /// Returns the expected value at an information set.
    ///
    /// The `payoff` field is an incremental mean (updated via Welford's method),
    /// so it is directly readable without division by visits. Used for frontier
    /// evaluation in depth-limited search and safe subgame solving.
    ///
    /// TODO
    /// expected value is actually a property of an INFOSET not an EDGE
    /// but it's convenient to store it bound to the EDGE where we already
    /// keep track of regret, weight, visits
    fn frontier_payoff(&self, info: &Self::I) -> Utility {
        let edge = &info
            .choices()
            .next()
            .expect("has actions, all of which have the same EV! next is good enough");
        self.cum_payoff(info, edge)
    }

    /// Leaf node value: payoff for terminals, frontier EV otherwise.
    ///
    /// Three cases for childless leaves (width == 0):
    /// 1. **Terminal** (fold/showdown): game payoff
    /// 2. **Chance frontier** (depth-limited at street boundary): nearest
    ///    decision ancestor's V(I), which is populated during blueprint training.
    ///    Walks up past consecutive chance nodes (e.g. multi-street runouts).
    /// 3. **Decision frontier** (sampler chose not to expand): this node's V(I)
    fn terminal_value(&self, node: &Node<Self::T, Self::E, Self::G, Self::I>, hero: Self::T) -> Utility {
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

    /// Expected value at external (opponent-controlled) subtree.
    ///
    /// **Recursive descent** through tree children, weighting by opponent's
    /// averaged strategy at each external node. Handles terminal, frontier,
    /// and chance nodes. Contrast with upward iteration in [`CfrNash::averaged`].
    fn external_payoff(&self, node: &Node<Self::T, Self::E, Self::G, Self::I>, hero: Self::T) -> Utility {
        self.subgamed_payoff(node, hero, None)
    }

    /// Recursive expected value using precomputed best response actions.
    fn response_payoff(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
        hero: Self::T,
        br: &HashMap<Self::I, Self::E>,
    ) -> Utility {
        self.subgamed_payoff(node, hero, Some(br))
    }

    /// Recursive expected value computation.
    /// When `br` is `None`, uses average strategy at both hero and opponent nodes.
    /// When `br` is `Some`, hero follows BR actions while opponents use average strategy.
    fn subgamed_payoff(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
        hero: Self::T,
        br: Option<&HashMap<Self::I, Self::E>>,
    ) -> Utility {
        let n = node.width();
        let kids = node.children();
        let recurse = |x| self.subgamed_payoff(x, hero, br);
        match node.game().turn() {
            _ if n == 0 => self.terminal_value(node, hero),
            t if t == Self::T::chance() => kids.iter().map(recurse).sum::<Utility>() / n as Utility,
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

    /// Product of external (opponent) strategy probabilities along path to node.
    ///
    /// Walks **upward** from node to root via parent pointers, filtering
    /// to opponent decision points and multiplying averaged probabilities.
    /// Dual of [`Solver::external_reach`](super::super::solver::Solver::external_reach),
    /// which walks **downward** from root replaying edges forward.
    fn external_reach(&self, node: &Node<Self::T, Self::E, Self::G, Self::I>, hero: Self::T) -> Probability {
        node.decisions()
            .filter(|(t, _, _)| *t != hero)
            .map(|(_, ref i, ref e)| self.averaged_policy(i, e))
            .product::<Probability>()
    }

    /// Best response value: optimal play for `hero` against opponents' average strategy.
    /// Respects info set structure by choosing one action per info set, not per node.
    fn optimal_response_payoff(
        &self,
        partition: &HashMap<Self::I, InfoSet<Self::T, Self::E, Self::G, Self::I>>,
        hero: Self::T,
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
    fn optimal_cfactual_payoff(
        &self,
        infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>,
        edge: &Self::E,
        hero: &Self::T,
    ) -> Utility {
        infoset
            .span()
            .iter()
            .filter_map(|n| n.step(edge))
            .map(|c| self.external_reach(&c, *hero) * self.external_payoff(&c, *hero))
            .sum()
    }

    /// Best action at an info set: argmax over actions of counterfactual value.
    fn optimal_cfactual_choice(&self, infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>, hero: Self::T) -> Self::E {
        infoset
            .info()
            .choices()
            .map(|edge| (edge, self.optimal_cfactual_payoff(infoset, &edge, &hero)))
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("good values"))
            .expect("info set has actions")
            .0
    }
}
