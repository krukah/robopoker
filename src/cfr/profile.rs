use super::data::Data;
use crate::cfr::bucket::Bucket;
use crate::cfr::edge::Edge;
use crate::cfr::node::Node;
use crate::cfr::player::Player;
use crate::cfr::policy::Policy;
use crate::Probability;
use std::collections::HashMap;

//? don't love how epoch is contagious across Trainer < Minimizer < Profile > >
pub struct Profile(HashMap<Bucket, Policy>);

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

    pub fn strategies(&self) -> impl Iterator<Item = (&Bucket, &Policy)> {
        self.0.iter()
    }

    pub fn get(&self, bucket: &Bucket, edge: &Edge) -> Probability {
        *self
            .0
            .get(bucket)
            .expect("valid bucket")
            .0
            .get(edge)
            .expect("policy initialized for actions")
    }

    // probability calculations
    pub fn weight(&self, node: &Node, edge: &Edge) -> Probability {
        // TODO this
        // should be function of BUCKET not NODE
        // but then how to get Player? only need it for chance nodes anyway...
        // maybe it's a function of Bucket?
        // tension between children being computed before or after weight calls
        // if before, then we need to know the player to call the right strategy

        // fn weight_for_tree_sample(node: &Node, edge: &Edge) -> Probability {
        //     let bucket = node.bucket();
        //     *self.get_ref(bucket, edge)
        // }
        // if after,  we have the luxury of taking uniform over all actions
        //
        // {
        //     let bucket = node.bucket();
        //     *self.get_ref(bucket, edge)
        // }
        match node.player() {
            Player::Chance => 1.0 / node.outgoing().len() as Probability,
            _ => self.get(node.bucket(), edge),
        }
    }
    pub fn cfactual_reach(&self, root: &Node) -> Probability {
        let mut prod = 1.0;
        let mut next = root;
        while let (Some(head), Some(edge)) = (next.parent(), next.incoming()) {
            if head.player() == root.player() {
                prod *= self.cfactual_reach(head);
                break;
            } else {
                prod *= self.weight(head, edge);
            }
            next = head;
        }
        prod
    }
    pub fn expected_reach(&self, node: &Node) -> Probability {
        if let (Some(head), Some(edge)) = (node.parent(), node.incoming()) {
            self.weight(head, edge) * self.expected_reach(head)
        } else {
            1.0
        }
    }
    pub fn relative_reach(&self, root: &Node, leaf: &Node) -> Probability {
        if root.bucket() == leaf.bucket() {
            1.0
        } else {
            let head = leaf.parent().expect("if has parent, then has incoming");
            let edge = leaf.incoming().expect("if has parent, then has incoming");
            self.weight(head, edge) * self.relative_reach(root, head)
        }
    }
    pub fn sampling_reach(&self, _: &Node, _: &Node) -> Probability {
        1.0 / 1.0
    }
}
