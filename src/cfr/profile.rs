use crate::cfr::bucket::Bucket;
use crate::cfr::edge::Edge;
use crate::cfr::info::Info;
use crate::cfr::memory::Memory;
use crate::cfr::node::Node;
use crate::cfr::player::Player;
use crate::Probability;
use crate::Utility;
use std::collections::BTreeMap;

pub struct Profile(BTreeMap<Bucket, BTreeMap<Edge, Memory>>, usize);
impl Profile {
    /// basic constructor
    pub fn empty() -> Self {
        Self(BTreeMap::new(), 0)
    }
    pub fn step(&mut self) -> usize {
        self.1 += 1;
        self.1
    }
    pub fn witness(&mut self, node: &Node) {
        let bucket = node.bucket();
        if !self.0.contains_key(bucket) {
            let edges = node
                .datum()
                .spawn()
                .into_iter()
                .map(|(_, edge)| edge)
                .collect::<Vec<Edge>>();
            let p = 1. / edges.len() as Probability;
            for action in edges {
                self.insert(bucket.to_owned(), action, p);
            }
        }
    }

    // profile and time lookups
    // profile and time lookups
    // profile and time lookups
    // profile and time lookups
    pub fn walker(&self) -> &Player {
        match self.1 % 2 {
            0 => &Player::P1,
            _ => &Player::P2,
        }
    }
    pub fn policy(&self, bucket: &Bucket, edge: &Edge) -> Probability {
        self.0
            .get(bucket)
            .expect("policy bucket/edge has been visited before")
            .get(edge)
            .expect("policy bucket/edge has been visited before")
            .policy
            .to_owned()
    }
    pub fn regret(&self, bucket: &Bucket, edge: &Edge) -> Utility {
        self.0
            .get(bucket)
            .expect("regret bucket/edge has been visited before")
            .get(edge)
            .expect("regret bucket/edge has been visited before")
            .regret
            .to_owned()
    }

    // online minimization via regret matching ++
    // online minimization via regret matching ++
    // online minimization via regret matching ++
    // online minimization via regret matching ++
    pub fn update_regret(&mut self, infoset: &Info) {
        for (ref action, ref regret) in self.regret_vector(infoset) {
            let bucket = infoset.node().bucket();
            let update = self.update(bucket, action);
            update.regret = *regret;
        }
    }
    pub fn update_policy(&mut self, infoset: &Info) {
        for (ref action, ref weight) in self.policy_vector(infoset) {
            let t = self.1;
            let bucket = infoset.node().bucket();
            let update = self.update(bucket, action);
            update.policy = *weight;
            update.advice *= t as Probability;
            update.advice += weight;
            update.advice /= t as Probability + 1.0;
        }
    }

    // write-through memory
    // write-through memory
    // write-through memory
    // write-through memory
    fn insert(&mut self, bucket: Bucket, edge: Edge, probability: Probability) {
        self.0
            .entry(bucket)
            .or_insert_with(BTreeMap::new)
            .entry(edge)
            .or_insert_with(Memory::new)
            .policy = probability;
    }
    fn update(&mut self, bucket: &Bucket, edge: &Edge) -> &mut Memory {
        self.0
            .get_mut(bucket)
            .expect("conditional on update, bucket should be visited")
            .get_mut(edge)
            .expect("conditional on update, action should be visited")
    }

    // update vector calculations
    // update vector calculations
    // update vector calculations
    // update vector calculations
    fn policy_vector(&self, infoset: &Info) -> BTreeMap<Edge, Probability> {
        let regrets = infoset
            .node()
            .outgoing()
            .into_iter()
            .map(|action| (action.to_owned(), self.running_regret(infoset, action)))
            .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
            .collect::<BTreeMap<Edge, Utility>>();
        let summed = regrets.values().sum::<Utility>();
        let vector = regrets
            .into_iter()
            .map(|(a, r)| (a, r / summed))
            .collect::<BTreeMap<Edge, Probability>>();
        vector
    }
    fn regret_vector(&self, infoset: &Info) -> BTreeMap<Edge, Utility> {
        infoset
            .node()
            .outgoing()
            .into_iter()
            .map(|action| (action.to_owned(), self.matched_regret(infoset, action)))
            .collect()
    }

    // regret calculations
    /// regret calculations
    /// regret calculations
    /// regret calculations
    fn matched_regret(&self, infoset: &Info, action: &Edge) -> Utility {
        let running = self.running_regret(infoset, action);
        let instant = self.instant_regret(infoset, action);
        (running + instant).max(Utility::MIN_POSITIVE)
    }
    fn running_regret(&self, infoset: &Info, action: &Edge) -> Utility {
        let bucket = infoset.node().bucket();
        let regret = self.regret(bucket, action);
        regret
    }
    fn instant_regret(&self, infoset: &Info, action: &Edge) -> Utility {
        infoset
            .roots()
            .iter()
            .map(|root| self.gain(root, action))
            .sum::<Utility>()
    }

    // utility calculations
    // utility calculations
    // utility calculations
    // utility calculations
    fn gain(&self, root: &Node, edge: &Edge) -> Utility {
        let expected = self.expected_value(root);
        let cfactual = self.cfactual_value(root, edge);
        cfactual - expected
        // should hoist outside of action/edge loop.
        // label each Node with EV
        // then use that memoized value for CFV
        // memoize via Cell<Option<Utility>>
    }
    fn cfactual_value(&self, root: &Node, edge: &Edge) -> Utility {
        self.cfactual_reach(root)
            * root
                .follow(edge)
                .leaves()
                .iter()
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn expected_value(&self, root: &Node) -> Utility {
        self.expected_reach(root)
            * root
                .leaves()
                .iter()
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn relative_value(&self, root: &Node, leaf: &Node) -> Utility {
        Node::payoff(root, leaf)
            * self.relative_reach(root, leaf)
            * self.sampling_reach(root, leaf)
            * 1.
    }

    // probability calculations
    // probability calculations
    // probability calculations
    // probability calculations
    fn reach(&self, node: &Node, edge: &Edge) -> Probability {
        if node.player() == &Player::Chance {
            1. / node.outgoing().len() as Probability
        } else {
            self.policy(node.bucket(), edge)
        }
    }
    fn cfactual_reach(&self, node: &Node) -> Probability {
        if let (Some(head), Some(edge)) = (node.parent(), node.incoming()) {
            if head.player() == node.player() {
                self.cfactual_reach(head)
            } else {
                self.cfactual_reach(head) * self.reach(head, edge)
            }
        } else {
            1.0
        }
    }
    fn expected_reach(&self, node: &Node) -> Probability {
        if let (Some(head), Some(edge)) = (node.parent(), node.incoming()) {
            self.expected_reach(head) * self.reach(head, edge)
        } else {
            1.0
        }
    }
    fn relative_reach(&self, root: &Node, leaf: &Node) -> Probability {
        if root.bucket() != leaf.bucket() {
            let head = leaf
                .parent()
                .expect("leaf is a descendant of root, therefore has a parent");
            let edge = leaf
                .incoming()
                .expect("leaf is a descendant of root, therefore has a parent");
            self.relative_reach(root, head) * self.reach(head, edge)
        } else {
            1.0
        }
    }
    fn sampling_reach(&self, _: &Node, _: &Node) -> Probability {
        1.0 / 1.0
    }
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (bucket, edges) in &self.0 {
            for (edge, memory) in edges {
                writeln!(f, "{:?} {:?}: {:.4}", bucket, edge, memory)?;
            }
        }
        Ok(())
    }
}
