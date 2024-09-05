use std::collections::HashMap;

use super::bucket::Bucket;
use crate::cfr::edge::Edge;
use crate::cfr::info::Info;
use crate::cfr::node::Node;
use crate::cfr::player::Player;
use crate::cfr::profile::Profile;
use crate::cfr::tree::Tree;
use crate::Probability;
use crate::Utility;

struct Solution(HashMap<Bucket, HashMap<Edge, Memory>>);
struct Memory {
    profile: Probability,
    regrets: Probability,
    average: Probability,
}

// ==

pub struct Solver {
    t: usize,
    regrets: Profile,
    current: Profile,
    average: Profile,
}

impl Solver {
    pub fn new() -> Self {
        Self {
            t: 0,
            average: Profile::new(),
            current: Profile::new(),
            regrets: Profile::new(),
        }
    }
    pub fn report(&self) {
        const CHECKPOINT: usize = 1_000;
        if self.t % CHECKPOINT == 0 {
            println!("T{}", self.t);
            for (bucket, strategy) in self.mean().strategies() {
                for (action, weight) in strategy.0.iter() {
                    println!("Bucket {:?}  {:?}: {:.4?}", bucket, action, weight);
                }
                break;
            }
        }
    }
    pub fn mean(&self) -> &Profile {
        &self.average
    }
    pub fn solve(&mut self, epochs: usize) {
        while self.t < epochs {
            for block in self.sample().blocks() {
                if self.walker() == block.node().player() {
                    self.update_regret(block);
                    self.update_policy(block);
                } else {
                    continue;
                }
            }
            self.report();
            self.t += 1;
        }
    }

    /// generate a new MC tree using the current profile as a sampling distribution
    /// - use DFS to expand
    /// - use the current profile as a sampling distribution
    /// - initialize unreached Buckets with normalized probabilities
    fn sample(&self) -> Tree {
        Tree::new()
    }

    /// update regrets via regret matching
    fn update_regret(&mut self, info: &Info) {
        for (ref action, ref mut regret) in self.regret_vector(info) {
            let bucket = info.node().bucket();
            let running = self.regrets.get_mut(bucket, action);
            std::mem::swap(running, regret);
        }
    }
    fn update_policy(&mut self, info: &Info) {
        for (ref action, weight) in self.policy_vector(info) {
            let bucket = info.node().bucket();
            let current = self.current.get_mut(bucket, action);
            let average = self.average.get_mut(bucket, action);
            *current = weight;
            *average *= self.t as Probability;
            *average += weight;
            *average /= self.t as Probability + 1.;
        }
    }

    /// policy calculation via cumulative regrets
    fn policy_vector(&self, infonode: &Info) -> Vec<(Edge, Probability)> {
        let regrets = infonode
            .node()
            .outgoing()
            .iter()
            .map(|action| (**action, self.running_regret(infonode, action)))
            .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
            .collect::<Vec<(Edge, Probability)>>();
        let sum = regrets.iter().map(|(_, r)| r).sum::<Utility>();
        let policy = regrets.into_iter().map(|(a, r)| (a, r / sum)).collect();
        policy
    }
    fn regret_vector(&self, infonode: &Info) -> Vec<(Edge, Utility)> {
        infonode
            .node()
            .outgoing()
            .into_iter()
            .map(|action| (*action, self.matched_regret(infonode, action)))
            .collect()
    }

    /// regret storage and calculation
    fn matched_regret(&self, infonode: &Info, action: &Edge) -> Utility {
        let running = self.running_regret(infonode, action);
        let instant = self.instant_regret(infonode, action);
        (running + instant).max(Utility::MIN_POSITIVE)
    }
    fn running_regret(&self, infonode: &Info, action: &Edge) -> Utility {
        let bucket = infonode.node().bucket();
        let regret = self.regrets.get_ref(bucket, action);
        *regret
    }
    fn instant_regret(&self, infonode: &Info, action: &Edge) -> Utility {
        infonode
            .roots()
            .iter()
            .map(|root| self.gain(root, action))
            .sum::<Utility>()
    }

    // marginal counterfactual gain over strategy EV
    fn gain(&self, root: &Node, edge: &Edge) -> Utility {
        let cfactual = self.cfactual_value(root, edge);
        let expected = self.expected_value(root); // should hoist outside of action/edge loop
        cfactual - expected
    }
    fn cfactual_value(&self, root: &Node, edge: &Edge) -> Utility {
        self.current.cfactual_reach(root) * self.visiting_value(root.follow(edge))
    }
    fn expected_value(&self, root: &Node) -> Utility {
        self.current.expected_reach(root) * self.visiting_value(root)
    }
    fn visiting_value(&self, root: &Node) -> Utility {
        self.explore(root)
            .iter()
            .map(|leaf| self.relative_value(root, leaf))
            .sum::<Utility>()
    }
    fn relative_value(&self, root: &Node, leaf: &Node) -> Utility {
        Node::payoff(root, leaf)
            * self.current.relative_reach(root, leaf)
            * self.current.sampling_reach(root, leaf)
    }

    /// external sampling helper method derived from epoch
    /// basically just used to alternate between P1 and P2
    fn walker(&self) -> &Player {
        match self.t % 2 {
            0 => &Player::P1,
            _ => &Player::P2,
        }
    }
}
