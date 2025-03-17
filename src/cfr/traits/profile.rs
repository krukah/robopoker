use super::edge::Edge;
use super::edge::EdgeSet;
use super::node::Node;
use super::node::NodeSet;
use super::policy::Policy;
use super::turn::Turn;
use crate::Probability;
use crate::Utility;

/// To get weights for edges under certain
/// information sets, we need to be able to map
/// buckets/infosets to policies, i.e.
/// Bucket -> Policy -> Edge -> Probability
pub trait Profile {
    /// lookup policy at this infoset (assume you have one)
    fn policy<P, D, E>(&self, info: &D) -> P
    where
        E: Edge,
        D: EdgeSet<E>,
        P: Policy<E>,
    {
        todo!("self.get(info.decision::<E, D>()).expect('policy)")
    }
    /// decide who's walking the tree rn
    fn walker<T>(&self) -> &T
    where
        T: Turn,
    {
        todo!("iterations % 2")
    }
    /// local transition probability. if we don't have a policy
    /// lookup for this Info, we will take the uniform distribution
    /// over node.info::<I>().choices::<J, E>(). i'll leave the
    /// Info type up to the implementor and beyond the scope of this trait.
    fn outgoing_reach<N, E>(&self, node: N, edge: E) -> Probability
    where
        N: Node,
        E: Edge,
    {
        todo!("lookup policy via self.get(node.info::<D>().decision::<E, D>()).expect(edge)")
    }
    /// assuming we start at a given head Node,
    /// and that that node is our walker,
    /// then what is the Utility of this leaf Node?
    fn terminal_value<N, E, T, I>(&self, root: N, leaf: N) -> Utility
    where
        N: Node,
        E: Edge,
        T: Turn,
        I: Iterator<Item = N>,
    {
        todo!("ferry game.payouts, w.r.t. player whose turn it is at root");
    }

    /// Conditional on being in a given Infoset,
    /// what is the Probability of
    /// visiting this particular leaf Node,
    /// given the distribution offered by Profile?
    fn relative_reach<N, E>(&self, root: N, leaf: N) -> Probability
    where
        N: Node,
        E: Edge,
    {
        if root == leaf {
            1.0
        } else {
            match leaf.parent::<E>() {
                None => unreachable!("tail must be downstream of head"),
                Some((parent, incoming)) => {
                    1.0 * self.relative_reach::<N, E>(root, parent)
                        * self.outgoing_reach::<N, E>(parent, incoming)
                }
            }
        }
    }

    /// If we were to play by the Profile,
    /// up to this Node in the Tree,
    /// then what is the probability of visiting this Node?
    fn expected_reach<N, E>(&self, root: N) -> Probability
    where
        N: Node,
        E: Edge,
    {
        match root.parent::<E>() {
            None => 1.0,
            Some((parent, incoming)) => {
                1.0 * self.expected_reach::<N, E>(parent)
                    * self.outgoing_reach::<N, E>(parent, incoming)
            }
        }
    }

    /// If, counterfactually, we had played toward this infoset,
    /// then what would be the Probability of us being in this infoset?
    /// i.e. assuming our opponents played according to distributions from Profile, but we did not.
    ///
    /// This function also serves as a form of importance sampling.
    /// MCCFR requires we adjust our reach in counterfactual
    /// regret calculation to account for the under- and over-sampling
    /// of regret across different Infosets.
    fn cfactual_reach<N, E, T>(&self, node: N) -> Probability
    where
        N: Node,
        E: Edge,
        T: Turn,
    {
        match node.parent::<E>() {
            None => 1.0,
            Some((parent, incoming)) => {
                self.cfactual_reach::<N, E, T>(parent)
                    * if self.walker::<T>() != parent.turn::<T>() {
                        self.outgoing_reach::<N, E>(parent, incoming)
                    } else {
                        1.0
                    }
            }
        }
    }

    /// Assuming we start at root Node,
    /// and that we sample the Tree according to Profile,
    /// how much Utility do we expect upon
    /// visiting this Node?
    fn expected_value<N, E, T, I>(&self, root: N) -> Utility
    where
        N: Node,
        E: Edge,
        T: Turn,
        I: Iterator<Item = N>,
    {
        assert!(self.walker::<T>() == root.turn::<T>());
        self.expected_reach::<N, E>(root)
            * root
                .descendants::<I>()
                .map(|leaf| self.terminal_value::<N, E, T, I>(root, leaf))
                .sum::<Utility>()
    }

    /// assuming we start at a given head Node,
    /// and that we sample the tree according to Profile,
    /// how much Utility does
    /// this leaf Node backpropagate up to us?
    fn intended_value<N, E, T, I>(&self, root: N, leaf: N) -> Utility
    where
        N: Node,
        E: Edge,
        T: Turn,
        I: Iterator<Item = N>,
    {
        // should the relative reach calculation use head at all? may be double counted at self.cfactual.profile.cfactual_reach(head). maybe use expected_reach instead?
        assert!(self.walker::<T>() == root.turn::<T>());
        1.0 * 1.0
            * 1.0
            * self.expected_value::<N, E, T, I>(root)
            * self.relative_reach::<N, E>(root, leaf)
            / self.cfactual_reach::<N, E, T>(leaf)
    }

    /// If, counterfactually,
    /// we had intended to get ourselves in this infoset,
    /// then what would be the expected Utility of this leaf?
    fn cfactual_value<N, E, T, I>(&self, root: N, edge: &E) -> Utility
    where
        N: Node,
        E: Edge,
        T: Turn,
        I: Iterator<Item = N>,
    {
        // maybe use expected_reach instead? cfactual_reach may double count at bayesian_value in numerator
        assert!(self.walker::<T>() == root.turn::<T>());
        self.cfactual_reach::<N, E, T>(root)
            * root
                .follow(edge)
                .expect("edge belongs to outgoing")
                .descendants::<I>()
                .map(|leaf| self.intended_value::<N, E, T, I>(root, leaf))
                .sum::<Utility>()
    }

    /// Conditional on being in this Infoset,
    /// distributed across all its head Nodes,
    /// with paths weighted according to our Profile:
    /// if we follow this Edge 100% of the time,
    /// what is the expected marginal increase in Utility?
    fn info_gain<N, E, T, I>(&self, roots: I, edge: &E) -> Utility
    where
        N: Node,
        E: Edge,
        T: Turn,
        I: Iterator<Item = N>,
    {
        roots
            .inspect(|root| assert!(self.walker::<T>() == root.turn::<T>()))
            .map(|root| self.node_gain::<N, E, T, I>(root, edge))
            .sum::<Utility>()
    }
    fn node_gain<N, E, T, I>(&self, root: N, edge: &E) -> Utility
    where
        N: Node,
        E: Edge,
        T: Turn,
        I: Iterator<Item = N>,
    {
        self.cfactual_value::<N, E, T, I>(root, edge) - self.expected_value::<N, E, T, I>(root)
    }

    /// Using our current strategy Profile,
    /// compute the regret vector
    /// by calculating the marginal Utitlity
    /// missed out on for not having followed
    /// every walkable Edge at this Infoset/Node/Bucket
    fn regret_vector<N, P, I, D, E, T>(&self, infoset: I) -> P
    where
        N: Node,
        E: Edge,
        T: Turn,
        I: NodeSet<N>,
        D: EdgeSet<E>,
        P: Policy<E>,
    {
        infoset
            .clone()
            .next()
            .expect("non-empty infoset")
            .outgoing::<E, D>()
            .map(|edge| (edge, self.info_gain::<N, E, T, I>(infoset.clone(), &edge)))
            .map(|(e, r)| (e, r.max(crate::REGRET_MIN)))
            .map(|(e, r)| (e, r.min(crate::REGRET_MAX)))
            .inspect(|(e, r)| log::trace!("{:16} ! {:>10}", format!("{:?}", e), r))
            .inspect(|(_, r)| assert!(!r.is_nan()))
            .inspect(|(_, r)| assert!(!r.is_infinite()))
            .collect::<P>()
    }
}
