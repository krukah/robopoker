use rand::rngs::SmallRng;
use rand::SeedableRng;

use crate::cfr::profile::profile::Profile;
use crate::cfr::tree::rps::action::Edge;
use crate::cfr::tree::rps::info::Info;
use crate::cfr::tree::rps::node::Node;
use crate::cfr::tree::rps::player::Player;
use crate::cfr::tree::rps::tree::Tree;
use crate::Probability;
use crate::Utility;

type Epoch = usize;
type Regrets = Profile;

pub struct Solver {
    tree: Tree,
    epoch: Epoch,
    regrets: Regrets,
    current: Profile,
    average: Profile,
}
impl Solver {
    pub fn new() -> Self {
        Self {
            tree: Tree::new(),
            epoch: 0,
            average: Profile::new(),
            current: Regrets::new(),
            regrets: Profile::new(),
        }
    }
    pub fn report(&self) {
        const CHECKPOINT: Epoch = 1_000;
        if self.epoch % CHECKPOINT == 0 {
            println!("T{}", self.epoch);
            for (bucket, strategy) in self.average().0.iter() {
                for (action, weight) in strategy.0.iter() {
                    println!("Bucket {:?}  {:?}: {:.4?}", bucket, action, weight);
                }
                break;
            }
        }
    }
    pub fn average(&self) -> &Profile {
        &self.average
    }

    pub fn solve(&mut self, epochs: usize) {
        self.initialize();
        while self.epoch < epochs {
            self.step();
            self.report();
            self.epoch += 1;
        }
    }
    pub fn step(&mut self) {
        for block in self.tree.blocks() {
            if self.walker() != block.sample().player() {
                continue;
            }
            self.update_regret(block);
            self.update_policy(block);
        }
    }
    pub fn initialize(&mut self) {
        for info in self.tree.blocks() {
            let actions = info.sample().outgoing();
            let bucket = info.sample().bucket();
            let weight = 1.0 / actions.len() as Probability;
            let regret = 0.0;
            for action in actions {
                self.regrets.set_val(*bucket, *action, regret);
                self.average.set_val(*bucket, *action, weight);
                self.current.set_val(*bucket, *action, weight);
            }
        }
    }

    // TODO
    /*
    mutable recursive update_regret , update_policy methods
    take Node as argument rather than Info, since regret calcs are implicitly 1-node-infosets in external sampling
    */

    // external sampling helper methods derived from epoch
    fn walker(&self) -> &Player {
        match self.epoch % 2 {
            0 => &Player::P1,
            _ => &Player::P2,
        }
    }

    fn update_regret(&mut self, info: &Info) {
        for (ref action, regret) in self.regret_vector(info) {
            let bucket = info.sample().bucket();
            let running = self.regrets.get_mut(bucket, action);
            *running = regret;
        }
    }
    fn update_policy(&mut self, info: &Info) {
        for (ref action, weight) in self.policy_vector(info) {
            let bucket = info.sample().bucket();
            let current = self.current.get_mut(bucket, action);
            let average = self.average.get_mut(bucket, action);
            *current = weight;
            *average *= self.epoch as Probability;
            *average += weight;
            *average /= self.epoch as Probability + 1.;
        }
    }

    // policy calculation via cumulative regrets
    // regret calculation via regret matching +
    fn policy_vector(&self, infonode: &Info) -> Vec<(Edge, Probability)> {
        let regrets = infonode
            .sample()
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
            .sample()
            .outgoing()
            .into_iter()
            .map(|action| (*action, self.matched_regret(infonode, action)))
            .collect()
    }

    // regret storge and calculation
    fn matched_regret(&self, infonode: &Info, action: &Edge) -> Utility {
        let running = self.running_regret(infonode, action);
        let instant = self.instant_regret(infonode, action);
        (running + instant).max(Utility::MIN_POSITIVE)
    }
    fn running_regret(&self, infonode: &Info, action: &Edge) -> Utility {
        let bucket = infonode.sample().bucket();
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
        self.sample_terminal_nodes(root)
            .iter()
            .map(|leaf| self.relative_value(root, leaf))
            .sum::<Utility>()
    }
    fn relative_value(&self, root: &Node, leaf: &Node) -> Utility {
        Node::payoff(root, leaf)
            * self.current.relative_reach(root, leaf)
            * self.current.sampling_reach(root, leaf)
    }

    // recursive sampling methods
    fn select_terminal_nodes<'a>(&self, node: &'a Node) -> Terminals<'a> {
        match node.children().len() {
            0 => vec![&node],
            _ => node
                .children()
                .iter()
                .map(|child| self.select_terminal_nodes(child))
                .flatten()
                .collect(),
        }
    }
    fn sample_terminal_nodes<'a>(&self, node: &'a Node) -> Terminals<'a> {
        if 0 == node.children().len() {
            vec![&node]
        } else if self.walker() == node.player() {
            self.sample_terminal_nodes_all(node)
        } else {
            self.sample_terminal_nodes_one(node)
        }
    }
    fn sample_terminal_nodes_all<'a>(&self, node: &'a Node) -> Terminals<'a> {
        node.children()
            .iter()
            .map(|child| self.sample_terminal_nodes(child)) // mut self.regrets ( child.bucket(), child.incoming() ) = self.walk(child)
            .flatten()
            .collect()
    }
    fn sample_terminal_nodes_one<'a>(&self, node: &'a Node) -> Terminals<'a> {
        use rand::distributions::Distribution;
        use rand::distributions::WeightedIndex;

        let seed = [(self.epoch + node.index.index()) as u8; 32];
        let ref mut rng = SmallRng::from_seed(seed);
        let ref weights = node
            .outgoing()
            .iter()
            .map(|edge| self.current.weight(node, edge))
            .collect::<Vec<Probability>>();
        let distribution = WeightedIndex::new(weights).expect("same length");
        let child = distribution.sample(rng);
        let child = node.children().remove(child); // kidnapped!
        self.sample_terminal_nodes(child)
    }
}

type Terminals<'a> = Vec<&'a Node>;
