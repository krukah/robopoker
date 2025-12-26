use crate::mccfr::*;
use crate::*;
use petgraph::graph::NodeIndex;

/// Per-tree cache storing precomputed reach products and subtree values.
/// Used to optimize regret_vector computation from O(R × D × H) to O(N + R × E × H).
pub struct TreeCache {
    utility: Vec<Utility>,
    reaches: Vec<Probability>,
    sampled: Vec<Probability>,
}

impl TreeCache {
    pub fn new(n: usize) -> Self {
        Self {
            reaches: vec![1.0; n],
            sampled: vec![1.0; n],
            utility: vec![0.0; n],
        }
    }
    pub fn reach(&self, index: NodeIndex) -> Probability {
        self.reaches[index.index()]
    }
    pub fn sample(&self, index: NodeIndex) -> Probability {
        self.sampled[index.index()]
    }
    pub fn value(&self, index: NodeIndex) -> Utility {
        self.utility[index.index()]
    }
    pub fn set_reach(&mut self, index: NodeIndex, val: Probability) {
        self.reaches[index.index()] = val;
    }
    pub fn set_sample(&mut self, index: NodeIndex, val: Probability) {
        self.sampled[index.index()] = val;
    }
    pub fn set_value(&mut self, index: NodeIndex, val: Utility) {
        self.utility[index.index()] = val;
    }
    /// Top-down BFS pass: compute reach and sample products for each node.
    pub fn fill_reaches<T, E, G, I, P>(&mut self, tree: &Tree<T, E, G, I>, profile: &P)
    where
        T: TreeTurn,
        E: TreeEdge,
        G: TreeGame<E = E, T = T>,
        I: TreeInfo<E = E, T = T>,
        P: Profile<T = T, E = E, G = G, I = I>,
    {
        let walker = profile.walker();
        let chance = T::chance();
        for index in tree.bfs() {
            if let Some((parent, edge)) = tree.at(index).up() {
                let turn = parent.game().turn();
                self.set_reach(
                    index,
                    if turn == chance {
                        self.reach(parent.index()) * 1.0
                    } else {
                        self.reach(parent.index()) * profile.matching(parent.info(), edge)
                    },
                );
                self.set_sample(
                    index,
                    if turn == chance || turn == walker {
                        self.sample(parent.index()) * 1.0
                    } else {
                        self.sample(parent.index()) * profile.sampling(parent.info(), edge)
                    },
                );
            }
        }
    }
    /// Bottom-up postorder pass: compute subtree value sums for each node.
    pub fn fill_values<T, E, G, I, P>(&mut self, tree: &Tree<T, E, G, I>, profile: &P)
    where
        T: TreeTurn,
        E: TreeEdge,
        G: TreeGame<E = E, T = T>,
        I: TreeInfo<E = E, T = T>,
        P: Profile<T = T, E = E, G = G, I = I>,
    {
        let walker = profile.walker();
        for index in tree.postorder() {
            let value = self.node_value(tree.at(index), walker);
            self.set_value(index, value);
        }
    }
    /// Compute value for a single node: leaf payoff or sum of children.
    fn node_value<T, E, G, I>(&self, node: Node<T, E, G, I>, walker: T) -> Utility
    where
        T: TreeTurn,
        E: TreeEdge,
        G: TreeGame<E = E, T = T>,
        I: TreeInfo<E = E, T = T>,
    {
        match node.children().len() {
            0 => {
                self.reach(node.index()) * node.game().payoff(walker)
                    / self.sample(node.index()).max(POLICY_MIN)
            }
            _ => node
                .children()
                .into_iter()
                .map(|c| self.value(c.index()))
                .sum(),
        }
    }
}
