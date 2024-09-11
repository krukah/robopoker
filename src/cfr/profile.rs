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
    pub fn epochs(&self) -> usize {
        self.1
    }
    pub fn walker(&self) -> &Player {
        match self.1 % 2 {
            0 => &Player::P1,
            _ => &Player::P2,
        }
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
    /// only used for Tree sampling in Monte Carlo Trainer.
    /// assertions remain valid as long as Trainer::children is consistent
    /// with external sampling rules, where this fn is used to
    /// emulate the "opponent" strategy. the opponent is just whoever is not
    /// the traverser
    pub fn policy(&self, node: &Node, edge: &Edge) -> Probability {
        assert!(node.player() != &Player::Chance);
        assert!(node.player() != self.walker());
        self.0
            .get(node.bucket())
            .expect("policy bucket/edge has been visited before")
            .get(edge)
            .expect("policy bucket/edge has been visited before")
            .policy
            .to_owned()
    }
    // online minimization via regret matching ++
    // online minimization via regret matching ++
    // online minimization via regret matching ++
    // online minimization via regret matching ++
    pub fn update_regret(&mut self, infoset: &Info) {
        assert!(infoset.node().player() == self.walker());
        assert!(infoset.node().outgoing().len() == 3);
        let bucket = infoset.node().bucket();
        for (ref action, ref regret) in self.regret_vector(infoset) {
            let update = self.update(bucket, action);
            update.regret = *regret;
        }
    }
    pub fn update_policy(&mut self, infoset: &Info) {
        assert!(infoset.node().player() == self.walker());
        assert!(infoset.node().outgoing().len() == 3);
        let epochs = (self.epochs()) >> 1; //@no-tail-increment
        let bucket = infoset.node().bucket();
        self.normalize(bucket);
        for (ref action, ref policy) in self.policy_vector(infoset) {
            let update = self.update(bucket, action);
            update.policy = *policy;
            update.advice *= epochs as Probability;
            update.advice += policy;
            update.advice /= epochs as Probability + 1.0;
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
    fn normalize(&mut self, bucket: &Bucket) {
        let sum = self
            .0
            .get(bucket)
            .expect("conditional on normalize, bucket should be visited")
            .values()
            .map(|m| m.policy)
            .sum::<Probability>();
        for edge in self
            .0
            .get_mut(bucket)
            .expect("conditional on normalize, bucket should be visited")
            .values_mut()
        {
            edge.policy /= sum;
        }
    }

    // update vector calculations
    // update vector calculations
    // update vector calculations
    // update vector calculations
    fn regret_vector(&self, infoset: &Info) -> BTreeMap<Edge, Utility> {
        assert!(infoset.node().player() == self.walker());
        assert!(infoset.node().outgoing().len() == 3);
        infoset
            .node()
            .outgoing()
            .into_iter()
            .map(|action| (action.to_owned(), self.matched_regret(infoset, action)))
            .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
            .collect()
    }
    fn policy_vector(&self, infoset: &Info) -> BTreeMap<Edge, Probability> {
        assert!(infoset.node().player() == self.walker());
        assert!(infoset.node().outgoing().len() == 3);
        let regrets = infoset
            .node()
            .outgoing()
            .into_iter()
            .map(|action| (action.to_owned(), self.running_regret(infoset, action)))
            .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
            .collect::<BTreeMap<Edge, Utility>>();
        let denominator = regrets.values().sum::<Utility>();
        regrets
            .into_iter()
            .map(|(a, r)| (a, r / denominator))
            .collect::<BTreeMap<Edge, Probability>>()
    }

    /// regret calculations
    /// regret calculations
    /// regret calculations
    /// regret calculations
    fn matched_regret(&self, infoset: &Info, action: &Edge) -> Utility {
        let running = self.running_regret(infoset, action);
        let instant = self.instant_regret(infoset, action);
        running + instant
    }
    fn running_regret(&self, infoset: &Info, action: &Edge) -> Utility {
        let bucket = infoset.node().bucket();
        self.regret(bucket, action)
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
            assert!(node.outgoing().len() == 1);
            1.
        } else {
            self.0
                .get(node.bucket())
                .expect("policy bucket/edge has been visited before")
                .get(edge)
                .expect("policy bucket/edge has been visited before")
                .policy
                .to_owned()
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
        1.0
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
