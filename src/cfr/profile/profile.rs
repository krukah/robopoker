use crate::cfr::profile::strategy::Policy;
use crate::cfr::traits::action::Edge;
use crate::cfr::traits::bucket::Bucket;
use crate::cfr::traits::player::Player;
use crate::cfr::tree::node::Node;
use crate::Probability;
use crate::Utility;
use std::collections::HashMap;

pub(crate) struct Profile(pub HashMap<Bucket, Policy>);

impl Profile {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn gain(&self, root: &Node, action: &Edge) -> Utility {
        let cfactual = self.cfactual_value(root, action);
        let expected = self.expected_value(root);
        cfactual - expected
    }
    pub fn set(&mut self, bucket: Bucket, action: Edge, value: Utility) {
        self.0
            .entry(bucket)
            .or_insert_with(Policy::new)
            .0
            .insert(action, value);
    }
    pub fn get_ref(&self, bucket: &Bucket, action: &Edge) -> &Utility {
        self.0
            .get(bucket)
            .expect("valid bucket")
            .0
            .get(action)
            .expect("policy initialized for actions")
    }
    pub fn get_mut(&mut self, bucket: &Bucket, action: &Edge) -> &mut Utility {
        self.0
            .get_mut(bucket)
            .expect("valid bucket")
            .0
            .get_mut(action)
            .expect("policy initialized for actions")
    }

    // provided
    fn cfactual_value(&self, root: &Node, action: &Edge) -> Utility {
        1.0 * self.cfactual_reach(root)
            * root //                                       suppose you're here on purpose, counterfactually
                .follow(action) //                          suppose you're here on purpose, counterfactually
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
            * leaf.payoff(root.player())
    }
    // probability calculations
    fn weight(&self, node: &Node, action: &Edge) -> Probability {
        match node.player() {
            Player::Chance => {
                let n = node.outgoing().len();
                1.0 / n as Probability
            }
            Player::P1 | Player::P2 => {
                let bucket = node.bucket();
                *self.get_ref(bucket, action)
            }
        }
    }
    fn cfactual_reach(&self, node: &Node) -> Probability {
        match node.parent() {
            None => 1.0,
            Some(from) => {
                if node.player() == from.player() {
                    self.cfactual_reach(from)
                } else {
                    self.cfactual_reach(from)
                        * self.weight(from, node.incoming().expect("has parent"))
                }
            }
        }
    }
    fn strategy_reach(&self, node: &Node) -> Probability {
        match node.parent() {
            None => 1.0,
            Some(from) => {
                self.strategy_reach(from) * self.weight(from, node.incoming().expect("has parent"))
            }
        }
    }
    fn relative_reach(&self, root: &Node, leaf: &Node) -> Probability {
        if root.bucket() == leaf.bucket() {
            1.0
        } else {
            let node = leaf.parent().expect("if has parent, then has incoming");
            let edge = leaf.incoming().expect("if has parent, then has incoming");
            self.relative_reach(root, node) * self.weight(node, edge)
        }
    }
    fn sampling_reach(&self, _: &Node, _: &Node) -> Probability {
        1.0 / 1.0
    }
}
