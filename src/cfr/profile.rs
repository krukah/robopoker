use crate::cfr::bucket::Bucket;
use crate::cfr::edge::Edge;
use crate::cfr::node::Node;
use crate::cfr::player::Player;
use crate::cfr::policy::Policy;
use crate::Probability;
use std::collections::HashMap;

//? don't love how epoch is contagious across Trainer < Minimizer < Profile > >
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

    // probability calculations
    pub fn weight(&self, node: &Node, edge: &Edge) -> Probability {
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
    pub fn cfactual_reach(&self, root: &Node) -> Probability {
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
    pub fn expected_reach(&self, node: &Node) -> Probability {
        match node.parent() {
            None => 1.0,
            Some(from) => {
                let edge = node.incoming().expect("has parent");
                self.weight(from, edge) * self.expected_reach(from)
            }
        }
    }
    pub fn relative_reach(&self, root: &Node, leaf: &Node) -> Probability {
        if root.bucket() == leaf.bucket() {
            1.0
        } else {
            let node = leaf.parent().expect("if has parent, then has incoming");
            let from = leaf.incoming().expect("if has parent, then has incoming");
            self.weight(node, from) * self.relative_reach(root, node)
        }
    }
    pub fn sampling_reach(&self, _: &Node, _: &Node) -> Probability {
        1.0 / 1.0
    }
}
