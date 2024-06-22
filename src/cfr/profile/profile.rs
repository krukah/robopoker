use crate::cfr::profile::policy::Policy;
use crate::cfr::tree::rps::action::Edge;
use crate::cfr::tree::rps::bucket::Bucket;
use crate::cfr::tree::rps::node::Node;
use crate::cfr::tree::rps::player::Player;
use crate::Probability;
use crate::Utility;
use std::collections::HashMap;

pub struct Profile(pub HashMap<Bucket, Policy>);

impl Profile {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get_ref(&self, bucket: &Bucket, edge: &Edge) -> &f32 {
        self.0
            .get(bucket)
            .expect("valid bucket")
            .0
            .get(edge)
            .expect("policy initialized for actions")
    }
    pub fn get_mut(&mut self, bucket: &Bucket, edge: &Edge) -> &mut f32 {
        self.0
            .get_mut(bucket)
            .expect("valid bucket")
            .0
            .get_mut(edge)
            .expect("policy initialized for actions")
    }
    pub fn set_val(&mut self, bucket: Bucket, edge: Edge, value: f32) {
        self.0
            .entry(bucket)
            .or_insert_with(Policy::new)
            .0
            .insert(edge, value);
    }

    pub fn gain(&self, root: &Node, edge: &Edge) -> Utility {
        let cfactual = self.cfactual_value(root, edge);
        let expected = self.expected_value(root);
        cfactual - expected
    }

    // monte carlo, external sampling variant. need to add traverser, maybe Profile { traverser }
    #[allow(dead_code)]
    fn leaves<'a>(&self, node: &'a Node) -> Vec<&'a Node> {
        match node.children().len() {
            0 => vec![&node],
            _ => node
                .children()
                .iter()
                .map(|child| self.leaves(child))
                .flatten()
                .collect(),
        }
    }
    fn sample<'a>(&self, node: &'a Node) -> Vec<&'a Node> {
        // external sampling explores ALL possible actions for the traverser and ONE possible action for chance & opps
        let traverser = node.player(); //  self.traverser: &Player
        if 0 == node.children().len() {
            vec![&node]
        } else if traverser == node.player() {
            self.expand(node)
        } else {
            self.select(node)
        }
    }
    fn expand<'a>(&self, node: &'a Node) -> Vec<&'a Node> {
        // implicitly we're at a node belonging to the traverser
        node.children()
            .iter()
            .map(|child| self.sample(child))
            .flatten()
            .collect()
    }
    fn select<'a>(&self, node: &'a Node) -> Vec<&'a Node> {
        // now wer'e at an opp or chance node
        use rand::distributions::Distribution;
        use rand::distributions::WeightedIndex;
        let mut rng = rand::thread_rng();
        let ref weights = node
            .outgoing()
            .iter()
            .map(|edge| self.weight(node, edge))
            .collect::<Vec<Probability>>();
        let distribution = WeightedIndex::new(weights).expect("same length");
        let index = distribution.sample(&mut rng);
        let child = *node.children().get(index).expect("valid index");
        self.sample(child)
    }

    // provided
    fn cfactual_value(&self, root: &Node, edge: &Edge) -> Utility {
        1.0 * self.cfactual_reach(root)
            * self
                .sample(root.follow(edge))
                .iter() //                                  duplicated calculation
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn expected_value(&self, root: &Node) -> Utility {
        1.0 * self.expected_reach(root)
            * self
                .sample(root)
                .iter() //                                  duplicated calculation
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn relative_value(&self, root: &Node, leaf: &Node) -> Utility {
        1.0 * self.relative_reach(root, leaf)
            * self.sampling_reach(root, leaf)
            * leaf.data.payoff(root.player())
    }

    // probability calculations
    fn weight(&self, node: &Node, edge: &Edge) -> Probability {
        match node.player() {
            Player::Chance => {
                let n = node.outgoing().len();
                1.0 / n as Probability
            }
            _ => {
                let bucket = node.bucket();
                *self.get_ref(bucket, edge)
            }
        }
    }
    fn cfactual_reach(&self, root: &Node) -> Probability {
        let mut prod = 1.0;
        let mut next = root;
        while let Some(from) = next.parent() {
            let edge = next.incoming().expect("has parent");
            if from.player() == root.player() {
                prod *= self.cfactual_reach(from);
                break;
            } else {
                prod *= self.weight(from, edge);
            }
            next = from;
        }
        prod
    }
    fn expected_reach(&self, node: &Node) -> Probability {
        match node.parent() {
            None => 1.0,
            Some(from) => {
                let edge = node.incoming().expect("has parent");
                self.weight(from, edge) * self.expected_reach(from)
            }
        }
    }
    fn relative_reach(&self, root: &Node, leaf: &Node) -> Probability {
        if root.bucket() == leaf.bucket() {
            1.0
        } else {
            let node = leaf.parent().expect("if has parent, then has incoming");
            let from = leaf.incoming().expect("if has parent, then has incoming");
            self.weight(node, from) * self.relative_reach(root, node)
        }
    }
    fn sampling_reach(&self, _: &Node, _: &Node) -> Probability {
        1.0 / 1.0
    }
}
