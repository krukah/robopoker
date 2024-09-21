use crate::cfr::bucket::Bucket;
use crate::cfr::edge::Edge;
use crate::cfr::info::Info;
use crate::cfr::memory::Memory;
use crate::cfr::node::Node;
use crate::cfr::player::Player;
use crate::play::continuation::Continuation;
use crate::Probability;
use crate::Utility;
use std::collections::BTreeMap;

pub struct Profile(BTreeMap<Bucket, BTreeMap<Edge, Memory>>, usize);
impl Profile {
    pub fn empty() -> Self {
        Self(BTreeMap::new(), 0)
    }
    /// increment Epoch counter
    /// and return current count
    pub fn step(&mut self) -> usize {
        self.1 += 1;
        self.1
    }
    /// idempotent initialization of Profile
    /// at a given Node.
    ///
    /// if we've already visited this Infoset,
    /// then we can skip over it.
    ///
    /// otherwise, we initialize the strategy
    /// at this Node with uniform distribution
    /// over its spawned support:
    /// Data -> Vec<(Data, Edge)>.
    pub fn witness(&mut self, node: &Node) {
        let bucket = node.bucket();
        if self.0.contains_key(bucket) {
            return;
        } else {
            let edges = node.datum().edges();
            let uniform = 1. / edges.len() as Probability;
            for edge in edges {
                self.0
                    .entry(bucket.clone())
                    .or_insert_with(BTreeMap::new)
                    .entry(edge)
                    .or_insert_with(Memory::default)
                    .policy = uniform;
            }
        }
    }

    /// update regret memory
    /// we calculated positive regrets for every Edge
    /// and replace our old regret with the new
    /// new_regret = (old_regret + now_regret) . max(0)
    pub fn update_regret(&mut self, infoset: &Info) {
        assert!(infoset.node().player() == self.walker());
        let bucket = infoset.node().bucket();
        for (ref action, ref regret) in self.regret_vector(infoset) {
            let update = self.update(bucket, action);
            update.regret = *regret;
        }
    }
    /// update strategy vector
    /// lookup our old/running regret vector.
    /// make strategy proportional to this cumulative regret:
    /// p ( action ) = action_regret / sum_actions if sum > 0 ;
    ///              =             1 / num_actions if sum = 0 .
    pub fn update_policy(&mut self, infoset: &Info) {
        assert!(infoset.node().player() == self.walker());
        let epochs = self.epochs();
        let bucket = infoset.node().bucket();
        // self.normalize(bucket);
        for (ref action, ref policy) in self.policy_vector(infoset) {
            let update = self.update(bucket, action);
            update.policy = *policy;
            update.advice *= epochs as Probability;
            update.advice += policy;
            update.advice /= epochs as Probability + 1.;
        }
    }

    /// how many Epochs have we traversed the Tree so far?
    ///
    /// the online nature of the CFR training algorithm
    /// makes this value intrinsic to the learned Profile
    /// weights, hence the tight coupling.
    /// training can be paused, exported, imported, resumed.
    /// division by 2 is used to allow each player
    /// one iteration to walk the Tree in a single Epoch
    pub fn epochs(&self) -> usize {
        self.1 / 2
    }
    /// which player is traversing the Tree on this Epoch?
    /// used extensively in assertions and utility calculations
    pub fn walker(&self) -> Player {
        match self.1 % 2 {
            0 => Player::Choice(Continuation::Decision(0)),
            _ => Player::Choice(Continuation::Decision(1)),
        }
    }
    /// only used for Tree sampling in Monte Carlo Trainer.
    /// assertions remain valid as long as Trainer::children is consistent
    /// with external sampling rules, where this fn is used to
    /// emulate the "opponent" strategy. the opponent is just whoever is not
    /// the traverser
    pub fn policy(&self, node: &Node, edge: &Edge) -> Probability {
        assert!(node.player() != Player::chance().to_owned());
        assert!(node.player() != self.walker());
        self.0
            .get(node.bucket())
            .expect("policy bucket/edge has been visited before")
            .get(edge)
            .expect("policy bucket/edge has been visited before")
            .policy
            .to_owned()
    }

    /// access to regrets, policy, and averaged policy
    /// are tightly coupled.
    /// we use this in Self::update_*
    /// to replace any of the three values
    fn update(&mut self, bucket: &Bucket, edge: &Edge) -> &mut Memory {
        self.0
            .get_mut(bucket)
            .expect("conditional on update, bucket should be visited")
            .get_mut(edge)
            .expect("conditional on update, action should be visited")
    }
    /// if we ever run into floating point issues
    /// from accumulated error in policy calculations,
    /// we can use this to rescale all the values
    /// in a given bucket
    #[allow(dead_code)]
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

    /// memory update calculations
    /// memory update calculations
    /// memory update calculations

    /// using our current strategy Profile,
    /// compute the regret vector
    /// by calculating the marginal Utitlity
    /// missed out on for not having followed
    /// every walkable Edge at this Infoset/Node/Bucket
    fn regret_vector(&self, infoset: &Info) -> BTreeMap<Edge, Utility> {
        assert!(infoset.node().player() == self.walker());
        infoset
            .node()
            .outgoing()
            .into_iter()
            .map(|action| (action.to_owned(), self.accrued_regret(infoset, action)))
            .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
            .collect()
    }
    /// using our current regret Profile,
    /// compute a new strategy vector
    /// by following a given Edge
    /// proportionally to how much regret we felt
    /// for not having followed that Edge in the past.
    fn policy_vector(&self, infoset: &Info) -> BTreeMap<Edge, Probability> {
        assert!(infoset.node().player() == self.walker());
        let regrets = infoset
            .node()
            .outgoing()
            .into_iter()
            .map(|action| (action.to_owned(), self.running_regret(infoset, action)))
            .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
            .collect::<BTreeMap<Edge, Utility>>();
        let sum = regrets.values().sum::<Utility>();
        regrets.into_iter().map(|(a, r)| (a, r / sum)).collect()
    }

    /// regret calculations
    /// regret calculations
    /// regret calculations

    /// on this Profile iteration,
    /// upon visiting this Infoset,
    /// how much regret do we feel
    /// across our strategy vector?
    fn accrued_regret(&self, infoset: &Info, action: &Edge) -> Utility {
        assert!(infoset.node().player() == self.walker());
        let running = self.running_regret(infoset, action);
        let instant = self.instant_regret(infoset, action);
        running + instant
    }
    /// historically,
    /// upon visiting any Node inthis Infoset,
    /// how much cumulative Utility have we missed out on
    /// for not having followed this Edge?
    fn running_regret(&self, infoset: &Info, edge: &Edge) -> Utility {
        assert!(infoset.node().player() == self.walker());
        self.0
            .get(infoset.node().bucket())
            .expect("regret bucket/edge has been visited before")
            .get(edge)
            .expect("regret bucket/edge has been visited before")
            .regret
            .to_owned()
    }
    /// conditional on being in this Infoset,
    /// distributed across all its head Nodes,
    /// with paths weighted according to our Profile:
    /// if we follow this Edge 100% of the time,
    /// what is the expected marginal increase in Utility?
    fn instant_regret(&self, infoset: &Info, edge: &Edge) -> Utility {
        assert!(infoset.node().player() == self.walker());
        infoset
            .heads()
            .iter()
            .map(|head| self.gain(head, edge))
            .sum::<Utility>()
        //? HOIST
        // calculate self.profiled_value(head)
        // in the outer scop
    }

    /// utility calculations
    /// utility calculations
    /// utility calculations

    /// if at this given head Node,
    /// we diverged from our Profile strategy
    /// by "playing toward" this Infoset
    /// and following this Edge 100% of the time,
    /// what is the expected marginal increase in Utility?
    fn gain(&self, head: &Node, edge: &Edge) -> Utility {
        assert!(head.player() == self.walker());
        let expected = self.expected_value(head);
        let cfactual = self.cfactual_value(head, edge);
        cfactual - expected
        //? HOIST
        // could hoist this outside of action/edge loop.
        // label each Node with EV
        // then use that memoized value for CFV
        // memoize via Cell<Option<Utility>>
    }
    /// assuming we start at root Node,
    /// and that we sample the Tree according to Profile,
    /// how much Utility do we expect upon
    /// visiting this Node?
    fn expected_value(&self, head: &Node) -> Utility {
        assert!(head.player() == self.walker());
        self.profiled_reach(head)
            * head
                .leaves()
                .iter()
                .map(|leaf| self.terminal_value(head, leaf))
                .sum::<Utility>()
    }
    /// if,
    /// counterfactually,
    /// we had intended to get ourselves in this infoset,
    /// then what would be the expected Utility of this leaf?
    fn cfactual_value(&self, head: &Node, edge: &Edge) -> Utility {
        assert!(head.player() == self.walker());
        self.external_reach(head)
            * head
                .follow(edge)
                .leaves()
                .iter()
                .map(|leaf| self.terminal_value(head, leaf))
                .sum::<Utility>()
    }
    /// assuming we start at a given head Node,
    /// and that we sample the tree according to Profile,
    /// how much Utility does
    /// this leaf Node backpropagate up to us?
    fn terminal_value(&self, head: &Node, leaf: &Node) -> Utility {
        assert!(head.player() == self.walker());
        assert!(leaf.children().len() == 0);
        let ref player = self.walker();
        leaf.payoff(player)  // Terminal Utility
        / self.external_reach(leaf) // Importance Sampling
        * self.relative_reach(head, leaf) // Path Probability
    }

    /// reach calculations
    /// reach calculations
    /// reach calculations

    /// given a Node on a Tree,
    /// what is the Probability
    /// that flows forward through this given Edge?
    /// note that we assume
    /// - Tree is sampled according to external sampling rules
    /// - we've visited this Infoset at least once, while sampling the Tree
    fn reach(&self, head: &Node, edge: &Edge) -> Probability {
        if head.player() == Player::chance() {
            //. RPS specific
            assert!(head.children().len() == 0);
            unreachable!("early return 1. rather than entering recursive branch")
        } else {
            self.0
                .get(head.bucket())
                .expect("policy bucket/edge has been visited before")
                .get(edge)
                .expect("policy bucket/edge has been visited before")
                .policy
                .to_owned()
        }
    }
    /// if,
    /// counterfactually,
    /// we had intended to get ourselves in this infoset,
    /// then what would be the Probability of us being
    /// in this infoset? that is, assuming our opponents
    /// played according to distributions from Profile,
    /// but we did not.
    ///
    /// this function also serves as a form of importance sampling.
    /// MCCFR requires we adjust our reach in counterfactual
    /// regret calculation to account for the under- and over-sampling
    /// of regret across different Infosets.
    fn external_reach(&self, node: &Node) -> Probability {
        if let (Some(head), Some(edge)) = (node.parent(), node.incoming()) {
            if head.player() == self.walker() {
                self.external_reach(head)
            } else {
                self.external_reach(head) * self.reach(head, edge)
            }
        } else {
            1.
        }
    }
    /// if we were to play by the Profile,
    /// up to this Node in the Tree,
    /// then what is the probability of visiting this Node?
    fn profiled_reach(&self, head: &Node) -> Probability {
        if let (Some(head), Some(edge)) = (head.parent(), head.incoming()) {
            self.profiled_reach(head) * self.reach(head, edge)
        } else {
            1.
        }
    }
    /// conditional on being in a given Infoset,
    /// what is the Probability of
    /// visiting this particular leaf Node,
    /// given the distribution offered by Profile?
    fn relative_reach(&self, root: &Node, leaf: &Node) -> Probability {
        if root.bucket() == leaf.bucket() {
            1.
        } else {
            if let (Some(head), Some(edge)) = (leaf.parent(), leaf.incoming()) {
                self.relative_reach(root, head) * self.reach(head, edge)
            } else {
                unreachable!("tail must have parent")
            }
        }
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
