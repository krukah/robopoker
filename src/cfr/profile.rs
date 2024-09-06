use crate::cfr::bucket::Bucket;
use crate::cfr::edge::Edge;
use crate::cfr::info::Info;
use crate::cfr::memory::Memory;
use crate::cfr::node::Node;
use crate::cfr::player::Player;
use crate::Probability;
use crate::Utility;
use std::collections::HashMap;

pub struct Profile(HashMap<Bucket, HashMap<Edge, Memory>>);
impl Profile {
    /// TODO achieve zero-copy on policy/regret vector updates.
    /// just need to align lifetimes?
    pub fn empty() -> Self {
        Self(HashMap::default())
    }

    // online minimization via regret matching ++
    // online minimization via regret matching ++
    // online minimization via regret matching ++
    // online minimization via regret matching ++
    pub fn update_regret(&mut self, infoset: &Info, _: usize) {
        for (action, ref regret) in self.regret_vector(infoset) {
            let bucket = infoset.node().bucket().to_owned();
            let memory = self.memory(bucket, action);
            memory.regret += *regret;
        }
    }
    pub fn update_policy(&mut self, infoset: &Info, t: usize) {
        for (action, ref weight) in self.policy_vector(infoset) {
            let bucket = infoset.node().bucket().to_owned();
            let memory = self.memory(bucket, action);
            memory.policy = *weight;
            memory.advice *= t as Probability;
            memory.advice += weight;
            memory.advice /= t as Probability + 1.0;
        }
    }

    // write-through memory
    // write-through memory
    // write-through memory
    // write-through memory
    pub fn memory(&mut self, bucket: Bucket, edge: Edge) -> &mut Memory {
        self.0
            .entry(bucket)
            .or_insert_with(HashMap::new)
            .entry(edge)
            .or_insert_with(Memory::new)
    }

    // regret and policy lookups
    // regret and policy lookups
    // regret and policy lookups
    // regret and policy lookups
    fn regret(&self, bucket: &Bucket, edge: &Edge) -> Utility {
        self.0
            .get(bucket)
            .expect("bucket/edge has been visited before")
            .get(edge)
            .expect("bucket/edge has been visited before")
            .regret
            .to_owned()
    }
    fn policy(&self, bucket: &Bucket, edge: &Edge) -> Probability {
        self.0
            .get(bucket)
            .expect("bucket/edge has been visited before")
            .get(edge)
            .expect("bucket/edge has been visited before")
            .policy
            .to_owned()
    }

    // regret and policy vector calculations
    // regret and policy vector calculations
    // regret and policy vector calculations
    // regret and policy vector calculations
    fn policy_vector(&self, infoset: &Info) -> HashMap<Edge, Probability> {
        let regrets = infoset
            .node()
            .outgoing()
            .into_iter()
            .map(|action| (action.to_owned(), self.running_regret(infoset, &action)))
            .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
            .collect::<HashMap<Edge, Utility>>();
        let summed = regrets.values().sum::<Utility>();
        let vector = regrets
            .into_iter()
            .map(|(a, r)| (a, r / summed))
            .collect::<HashMap<Edge, Probability>>();
        vector
    }
    fn regret_vector(&self, infoset: &Info) -> HashMap<Edge, Utility> {
        infoset
            .node()
            .outgoing()
            .into_iter()
            .map(|action| (action.to_owned(), self.matched_regret(infoset, action)))
            .collect()
    }
    fn instant_regret(&self, infoset: &Info, action: &Edge) -> Utility {
        infoset
            .roots()
            .iter()
            .map(|root| self.gain(root, action))
            .sum::<Utility>()
    }
    fn running_regret(&self, infoset: &Info, action: &Edge) -> Utility {
        let bucket = infoset.node().bucket();
        let regret = self.regret(bucket, action);
        regret
    }
    fn matched_regret(&self, infoset: &Info, action: &Edge) -> Utility {
        let running = self.running_regret(infoset, action);
        let instant = self.instant_regret(infoset, action);
        (running + instant).max(Utility::MIN_POSITIVE)
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
        self.cfactual_reach(root) * self.terminal_value(root.follow(edge))
    }
    fn expected_value(&self, root: &Node) -> Utility {
        self.expected_reach(root) * self.terminal_value(root)
    }
    fn terminal_value(&self, root: &Node) -> Utility {
        root.leaves()
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
    fn probability(&self, node: &Node, edge: &Edge) -> Probability {
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
                self.cfactual_reach(head) * self.probability(head, edge)
            }
        } else {
            1.0
        }
    }
    fn expected_reach(&self, node: &Node) -> Probability {
        if let (Some(head), Some(edge)) = (node.parent(), node.incoming()) {
            self.expected_reach(head) * self.probability(head, edge)
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
            self.relative_reach(root, head) * self.probability(head, edge)
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
        write!(f, "INFOSETS {}", self.0.len())
    }
}
