use crate::cfr::profile::profile::Profile;
use crate::cfr::tree::rps::action::Edge;
use crate::cfr::tree::rps::info::Info;
use crate::cfr::tree::rps::node::Node;
use crate::cfr::tree::rps::player::Player;
use crate::cfr::tree::rps::tree::Tree;
use crate::Probability;
use crate::Utility;

type Epoch = usize;

pub struct Minimizer {
    epoch: Epoch,
    regrets: Profile,
    current: Profile,
    average: Profile,
    traverser: Player,
}
impl Minimizer {
    pub fn report(&self) {
        if self.epoch % 1_000 == 0 {
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
    #[allow(dead_code)]
    pub fn current(&self) -> &Profile {
        &self.current
    }
    pub fn new(tree: &Tree) -> Self {
        let mut regrets = Profile::new();
        let mut average = Profile::new();
        let mut current = Profile::new();
        for info in tree.infosets() {
            let actions = info.sample().outgoing();
            let bucket = info.sample().bucket();
            let weight = 1.0 / actions.len() as Probability;
            let regret = 0.0;
            for action in actions {
                regrets.set_val(*bucket, *action, regret);
                average.set_val(*bucket, *action, weight);
                current.set_val(*bucket, *action, weight);
            }
        }
        Self {
            epoch: 0,
            average,
            current,
            regrets,
            traverser: Player::P1,
        }
    }

    // mutating update methods at each infoset
    pub fn update_epoch(&mut self, t: Epoch) {
        self.epoch = t;
        match self.epoch % 2 {
            0 => self.traverser = Player::P1,
            _ => self.traverser = Player::P2,
        }
    }
    pub fn update_regret(&mut self, info: &Info) {
        // bail if the traverser is not the player at the infoset
        if &self.traverser != info.sample().player() {
            return;
            //? TODO weird duplication of traverser check
            //? TODO weird duplication of traverser check
        }
        //? TODO weird duplication of traverser check
        for (ref action, regret) in self.regret_vector(info) {
            let bucket = info.sample().bucket();
            let running = self.regrets.get_mut(bucket, action);
            *running = regret;
        }
    }
    pub fn update_policy(&mut self, info: &Info) {
        // bail if the traverser is not the player at the infoset
        if &self.traverser != info.sample().player() {
            return;
            //? TODO weird duplication of traverser check
            //? TODO weird duplication of traverser check
        }
        //? TODO weird duplication of traverser check
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
    fn policy_vector(&self, info: &Info) -> Vec<(Edge, Probability)> {
        let regrets = info
            .sample()
            .outgoing()
            .iter()
            .map(|action| (**action, self.running_regret(info, action)))
            .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
            .collect::<Vec<(Edge, Probability)>>();
        let sum = regrets.iter().map(|(_, r)| r).sum::<Utility>();
        let policy = regrets.into_iter().map(|(a, r)| (a, r / sum)).collect();
        policy
    }
    fn regret_vector(&self, info: &Info) -> Vec<(Edge, Utility)> {
        info.sample()
            .outgoing()
            .iter()
            .map(|action| (**action, self.matched_regret(info, action)))
            .collect()
    }

    // regret storge and calculation
    fn matched_regret(&self, info: &Info, action: &Edge) -> Utility {
        let running = self.running_regret(info, action);
        let instant = self.instant_regret(info, action);
        (running + instant).max(Utility::MIN_POSITIVE)
    }
    fn running_regret(&self, info: &Info, action: &Edge) -> Utility {
        let bucket = info.sample().bucket();
        let regret = self.regrets.get_ref(bucket, action);
        *regret
    }
    fn instant_regret(&self, info: &Info, action: &Edge) -> Utility {
        info.roots()
            .iter()
            .map(|root| self.gain(root, action))
            .sum::<Utility>()
    }

    // marginal counterfactual gain over strategy EV
    fn gain(&self, root: &Node, edge: &Edge) -> Utility {
        let cfactual = self.cfactual_value(root, edge);
        let expected = self.expected_value(root);
        cfactual - expected
    }
    fn cfactual_value(&self, root: &Node, edge: &Edge) -> Utility {
        1.0 * self.current.cfactual_reach(root)
            * self
                .leaves(root.follow(edge))
                .iter() //                                  duplicated calculation
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn expected_value(&self, root: &Node) -> Utility {
        1.0 * self.current.expected_reach(root)
            * self
                .leaves(root)
                .iter() //                                  duplicated calculation
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn relative_value(&self, root: &Node, leaf: &Node) -> Utility {
        1.0 * self.current.relative_reach(root, leaf)
            * self.current.sampling_reach(root, leaf)
            * self.terminal_value(root, leaf)
    }
    fn terminal_value(&self, root: &Node, leaf: &Node) -> Utility {
        Node::payoff(root, leaf)
    }

    // recursive sampling methods
    fn leaves<'a>(&self, node: &'a Node) -> Vec<&'a Node> {
        match node.children().len() {
            0 => vec![&node],
            _ => node
                .children()
                .iter()
                .map(|child| self.leaves(child))
                .flatten()
                .collect(),
        }
    }
    #[allow(dead_code)]
    fn sample<'a>(&self, node: &'a Node) -> Vec<&'a Node> {
        // external sampling explores ALL possible actions for the traverser and ONE possible action for chance & opps
        let ref traverser = self.traverser;
        if 0 == node.children().len() {
            vec![&node]
        } else if traverser == node.player() {
            self.explore_all(node)
        } else {
            self.explore_one(node) // external sampling is weird
        }
    }
    #[allow(dead_code)]
    fn explore_all<'a>(&self, node: &'a Node) -> Vec<&'a Node> {
        // implicitly we're at a node belonging to the traverser
        node.children()
            .iter()
            .map(|child| self.sample(child))
            .flatten()
            .collect()
    }
    #[allow(dead_code)]
    fn explore_one<'a>(&self, node: &'a Node) -> Vec<&'a Node> {
        // now wer'e at an opp or chance node
        use rand::distributions::Distribution;
        use rand::distributions::WeightedIndex;

        let ref mut rng = rand::thread_rng();
        let ref weights = node
            .outgoing()
            .iter()
            .map(|edge| self.current.weight(node, edge))
            .collect::<Vec<Probability>>();
        let distribution = WeightedIndex::new(weights).expect("same length");
        let index = distribution.sample(rng);
        let child = node.children().remove(index); // kidnapped!
        self.sample(child)
    }
}
