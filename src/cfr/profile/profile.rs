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

    // provided
    fn cfactual_value(&self, root: &Node, edge: &Edge) -> Utility {
        1.0 * self.cfactual_reach(root)
            * root //                                       suppose you're here on purpose, counterfactually
                .follow(edge) //                            suppose you're here on purpose, counterfactually
                .descendants() //                           O(depth) recursive downtree
                .iter() //                                  duplicated calculation
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn expected_value(&self, root: &Node) -> Utility {
        1.0 * self.strategy_reach(root)
            * root
                .descendants() //                           O(depth) recursive downtree
                .iter() //                                  duplicated calculation
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn relative_value(&self, root: &Node, leaf: &Node) -> Utility {
        1.0 * self.relative_reach(root, leaf)
            * self.sampling_reach(root, leaf)
            * leaf.data.payoff(root.data.player())
    }
    // probability calculations
    fn weight(&self, node: &Node, edge: &Edge) -> Probability {
        match node.data.player() {
            Player::Chance => {
                let n = node.outgoing().len();
                1.0 / n as Probability
            }
            _ => {
                let bucket = node.data.bucket();
                *self.get_ref(bucket, edge)
            }
        }
    }
    fn cfactual_reach(&self, root: &Node) -> Probability {
        let mut prod = 1.0;
        let mut next = root;
        while let Some(from) = next.parent() {
            let edge = next.incoming().expect("has parent");
            if from.data.player() == root.data.player() {
                prod *= self.cfactual_reach(from);
                break;
            } else {
                prod *= self.weight(from, edge);
            }
            next = from;
        }
        prod
    }

    fn strategy_reach(&self, node: &Node) -> Probability {
        match node.parent() {
            None => 1.0,
            Some(from) => {
                let edge = node.incoming().expect("has parent");
                self.weight(from, edge) * self.strategy_reach(from)
            }
        }
    }
    fn relative_reach(&self, root: &Node, leaf: &Node) -> Probability {
        if root.data.bucket() == leaf.data.bucket() {
            1.0
        } else {
            let from = leaf.parent().expect("if has parent, then has incoming");
            let edge = leaf.incoming().expect("if has parent, then has incoming");
            self.weight(from, edge) * self.relative_reach(root, from)
        }
    }
    fn sampling_reach(&self, _: &Node, _: &Node) -> Probability {
        1.0 / 1.0
    }
}
