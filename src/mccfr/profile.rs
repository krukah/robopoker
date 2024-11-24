use super::counterfactual::Counterfactual;
use super::discount::Discount;
use super::memory::Memory;
use super::phase::Phase;
use super::policy::Policy;
use super::regret::Regret;
use super::strategy::Strategy;
use super::tree::Branch;
use crate::mccfr::bucket::Bucket;
use crate::mccfr::edge::Edge;
use crate::mccfr::info::Info;
use crate::mccfr::node::Node;
use crate::mccfr::player::Player;
use crate::play::ply::Ply;
use crate::Probability;
use crate::Utility;
use rand::prelude::Distribution;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::usize;

/// this is the meat of our solution.
/// we keep a (Regret, AveragePolicy, CurrentPolicy)
/// for each distinct Bucket(Path, Abstraction) that we visit.
/// we also count how many training epochs we've run so far.
/// i feel like this can be broken up into
/// - Minimizer: handles policy and regret updates by implementing some regret-minimzation subroutine
/// - Profile: stores policy & regret values. used by reference for a lot of calculations,
/// such as Reach, Utility, MinimizerRegretVector, MinimizerPolicyVector, SampleTree, etc.
#[derive(Default)]
pub struct Profile {
    iterations: usize,
    strategies: BTreeMap<Bucket, Strategy>,
}

impl Profile {
    const PREFIX: &'static str = "blueprint";
    /// check (by filename) if a profile has been saved to disk.
    pub fn done() -> bool {
        std::fs::metadata(format!("{}.profile.pgcopy", Self::PREFIX)).is_ok()
    }
    /// load existing profile from disk. implicit assumption of Self::done() having been checked beforehand.
    pub fn load() -> Self {
        if Self::done() {
            Self::from(Self::PREFIX)
        } else {
            Self::default()
        }
    }
    /// increment Epoch counter
    /// and return current count
    pub fn next(&mut self) -> usize {
        self.iterations += 1;
        self.iterations
    }
    /// idempotent initialization of Profile
    /// at a given Node.
    ///
    /// if we've already visited this Bucket,
    /// then we just want to make sure that
    /// the available outgoing Edges are consistent.
    ///
    /// otherwise, we initialize the strategy
    /// at this Node with uniform distribution
    /// over its outgoing Edges .
    pub fn witness(&mut self, node: &Node, children: &Vec<Branch>) {
        let bucket = node.bucket();
        match self.strategies.get(bucket) {
            None => {
                let n = children.len();
                let uniform = 1. / n as Probability;
                for edge in children.iter().map(|b| b.edge()) {
                    let mut memory = Memory::default();
                    memory.set_policy(uniform);
                    self.strategies
                        .entry(bucket.clone())
                        .or_insert_with(Strategy::new)
                        .entry(edge.clone())
                        .or_insert(memory);
                }
            }
            Some(_) => {
                // asssertion needs to relax once i reintroduce pruning\
                // some (incoming, children) branches will be permanently
                // pruned, both in the Profile and when sampling children
                // in this case we have to reasses "who" is expected to
                // have "what" edges on "which when" epochs
                // let existing = strategy.keys().collect::<BTreeSet<_>>();
                // let observed = children.iter().map(|b| b.edge()).collect::<BTreeSet<_>>();
                // assert!(observed == existing);
            }
        }
    }
    /// using our current strategy Profile,
    /// compute the regret vector
    /// by calculating the marginal Utitlity
    /// missed out on for not having followed
    /// every walkable Edge at this Infoset/Node/Bucket
    pub fn regret_vector(&self, infoset: &Info) -> BTreeMap<Edge, Utility> {
        assert!(infoset.node().player() == self.walker());
        log::trace!("regret vector @ {}", infoset.node().bucket());
        infoset
            .node()
            .outgoing()
            .into_iter()
            .map(|a| (a.clone(), self.immediate_regret(infoset, a)))
            .map(|(a, r)| (a, r.max(crate::REGRET_MIN)))
            .map(|(a, r)| (a, r.min(crate::REGRET_MAX)))
            .inspect(|(a, r)| log::trace!("{:16} ! {:>10 }", format!("{:?}", a), r))
            .inspect(|(_, r)| assert!(!r.is_nan()))
            .inspect(|(_, r)| assert!(!r.is_infinite()))
            .collect::<BTreeMap<Edge, Utility>>()
    }
    /// using our current regret Profile,
    /// compute a new strategy vector
    /// by following a given Edge
    /// proportionally to how much regret we felt
    /// for not having followed that Edge in the past.
    pub fn policy_vector(&self, infoset: &Info) -> BTreeMap<Edge, Probability> {
        assert!(infoset.node().player() == self.walker());
        log::trace!("policy vector @ {}", infoset.node().bucket());
        let regrets = infoset
            .node()
            .outgoing()
            .into_iter()
            .map(|action| (action.clone(), self.cumulated_regret(infoset, action)))
            .map(|(a, r)| (a, r.max(crate::POLICY_MIN)))
            .collect::<BTreeMap<Edge, Utility>>();
        let sum = regrets.values().sum::<Utility>();
        let policy = regrets
            .into_iter()
            .map(|(a, r)| (a, r / sum))
            .inspect(|(a, p)| log::trace!("{:16} ~ {:>5.03}", format!("{:?}", a), p))
            .inspect(|(_, p)| assert!(*p >= 0.))
            .inspect(|(_, p)| assert!(*p <= 1.))
            .collect::<BTreeMap<Edge, Probability>>();
        policy
    }

    /// update regret vector for a given Bucket
    pub fn add_regret(&mut self, bucket: &Bucket, regrets: &Regret) {
        log::trace!("update regret @ {}", bucket);
        let t = self.epochs();
        let phase = self.phase();
        let discount = Discount::default();
        let strategy = self
            .strategies
            .get_mut(bucket)
            .expect("bucket been witnessed");
        for (action, &regret) in regrets.inner() {
            let decision = strategy.get_mut(action).expect("action been witnessed");
            let discount = match phase {
                Phase::Discount => discount.regret(t, regret),
                Phase::Explore => 1.,
                Phase::Prune => 1.,
            };
            decision.add_regret(discount, regret);
            log::trace!("{} : {}", action, decision.regret());
        }
    }
    /// update policy vector for a given Bucket
    pub fn add_policy(&mut self, bucket: &Bucket, policys: &Policy) {
        log::trace!("update policy @ {}", bucket);
        let t = self.epochs();
        let discount = Discount::default();
        let strategy = self
            .strategies
            .get_mut(bucket)
            .expect("bucket been witnessed");
        for (action, &policy) in policys.inner() {
            let discount = discount.policy(t);
            let decision = strategy.get_mut(action).expect("action been witnessed");
            decision.add_policy(discount, policy);
            log::trace!("{} : {}", action, decision.policy());
        }
    }

    /// public metadata

    /// how many Epochs have we traversed the Tree so far?
    ///
    /// the online nature of the CFR training algorithm
    /// makes this value intrinsic to the learned Profile
    /// weights, hence the tight coupling.
    /// training can be paused, exported, imported, resumed.
    /// division by 2 is used to allow each player
    /// one iteration to walk the Tree in a single Epoch
    pub fn epochs(&self) -> usize {
        self.iterations
    }
    /// derive current phase from Epoch count
    pub fn phase(&self) -> Phase {
        Phase::from(self.epochs())
    }
    /// which player is traversing the Tree on this Epoch?
    /// used extensively in assertions and utility calculations
    pub fn walker(&self) -> Player {
        match self.iterations % 2 {
            0 => Player(Ply::Choice(0)),
            _ => Player(Ply::Choice(1)),
        }
    }
    /// full set of available actions and their weights (not Probabilities)
    pub fn policy(&self, bucket: &Bucket) -> Policy {
        self.strategies
            .get(bucket)
            .expect("bucket must exist")
            .policy()
    }
    /// absolute Probability. only used for Tree sampling in Monte Carlo Trainer.
    pub fn weight(&self, bucket: &Bucket, edge: &Edge) -> Probability {
        self.strategies
            .get(bucket)
            .expect("bucket must exist")
            .weight(edge)
    }
    /// generate seed for PRNG. using hashing yields for deterministic, reproducable sampling
    /// for our Monte Carlo sampling.
    pub fn rng(&self, node: &Node) -> SmallRng {
        let ref mut hasher = DefaultHasher::new();
        self.epochs().hash(hasher);
        node.bucket().hash(hasher);
        SmallRng::seed_from_u64(hasher.finish())
    }

    /// full exploration of my decision space Edges
    pub fn explore_all(&self, choices: Vec<Branch>, _: &Node) -> Vec<Branch> {
        choices
            .into_iter()
            .inspect(|Branch(_, edge, _)| assert!(edge.is_choice()))
            .collect()
    }
    /// uniform sampling of chance Edge
    pub fn explore_any(&self, choices: Vec<Branch>, head: &Node) -> Vec<Branch> {
        let n = choices.len();
        let mut choices = choices;
        let ref mut rng = self.rng(head);
        let choice = rng.gen_range(0..n);
        let chosen = choices.remove(choice);
        assert!(chosen.1.is_chance());
        vec![chosen]
    }
    /// Profile-weighted sampling of opponent Edge
    pub fn explore_one(&self, choices: Vec<Branch>, head: &Node) -> Vec<Branch> {
        use rand::distributions::WeightedIndex;
        let ref mut rng = self.rng(head);
        let ref bucket = head.bucket();
        let mut choices = choices;
        let policy = choices
            .iter()
            .map(|Branch(_, edge, _)| self.weight(bucket, edge))
            .collect::<Vec<Probability>>();
        let choice = WeightedIndex::new(policy)
            .expect("at least one policy > 0")
            .sample(rng);
        let chosen = choices.remove(choice);
        assert!(chosen.1.is_choice());
        vec![chosen]
    }

    /// counterfactual regret calculations

    /// compute regret and policy vectors for a given infoset
    pub fn counterfactual(&self, info: Info) -> Counterfactual {
        let regret = Regret::from(self.regret_vector(&info));
        let policy = Policy::from(self.policy_vector(&info));
        Counterfactual::from((info, regret, policy))
    }

    /// historically,
    /// upon visiting any Node inthis Infoset,
    /// how much cumulative Utility have we missed out on
    /// for not having followed this Edge?
    fn cumulated_regret(&self, infoset: &Info, edge: &Edge) -> Utility {
        assert!(infoset.node().player() == self.walker());
        let node = infoset.node();
        let bucket = node.bucket();
        self.strategies
            .get(bucket)
            .expect("bucket has been witnessed")
            .0
            .get(edge)
            .expect("action has been witnessed")
            .regret()
            / self.epochs() as Utility
    }
    /// conditional on being in this Infoset,
    /// distributed across all its head Nodes,
    /// with paths weighted according to our Profile:
    /// if we follow this Edge 100% of the time,
    /// what is the expected marginal increase in Utility?
    fn immediate_regret(&self, infoset: &Info, edge: &Edge) -> Utility {
        assert!(infoset.node().player() == self.walker());
        infoset
            .roots()
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
                .expect("valid edge to follow")
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
        let probability = self.relative_reach(head, leaf);
        let conditional = self.external_reach(leaf);
        let walker = self.walker();
        let reward = leaf.payoff(&walker);
        log::trace!("R{:<9} I{:<9} P{:<9}", reward, conditional, probability);
        reward * probability / conditional
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
        if Player::chance() == head.player() {
            1.
        } else {
            let ref bucket = head.bucket();
            let policy = self.weight(bucket, edge);
            policy
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
        if let (Some(parent), Some(incoming)) = (node.parent(), node.incoming()) {
            if parent.player() == self.walker() {
                self.external_reach(&parent)
            } else {
                self.external_reach(&parent) * self.reach(&parent, incoming)
            }
        } else {
            1.
        }
    }
    /// if we were to play by the Profile,
    /// up to this Node in the Tree,
    /// then what is the probability of visiting this Node?
    fn profiled_reach(&self, node: &Node) -> Probability {
        if let (Some(parent), Some(incoming)) = (node.parent(), node.incoming()) {
            self.profiled_reach(&parent) * self.reach(&parent, incoming)
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
        } else if let (Some(parent), Some(incoming)) = (leaf.parent(), leaf.incoming()) {
            self.relative_reach(root, &parent) * self.reach(&parent, incoming)
        } else {
            unreachable!("tail must have parent")
        }
    }
}
impl From<&str> for Profile {
    fn from(name: &str) -> Self {
        use crate::clustering::abstraction::Abstraction;
        use crate::mccfr::path::Path;
        use byteorder::ReadBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::BufReader;
        use std::io::Read;
        use std::io::Seek;
        use std::io::SeekFrom;
        log::info!("loading profile from disk");
        let file = File::open(format!("{}.profile.pgcopy", name)).expect("open file");
        let mut buffer = [0u8; 2];
        let mut strategies = BTreeMap::new();
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(19)).expect("seek past header");
        while reader.read_exact(&mut buffer).is_ok() {
            if u16::from_be_bytes(buffer) == 6 {
                // We expect 6 fields per record
                reader.read_u32::<BE>().expect("past path length");
                let past = Path::from(reader.read_u64::<BE>().expect("read past path"));
                reader.read_u32::<BE>().expect("abstraction length");
                let abs = Abstraction::from(reader.read_u64::<BE>().expect("read abstraction"));
                reader.read_u32::<BE>().expect("future path length");
                let future = Path::from(reader.read_u64::<BE>().expect("read future path"));
                reader.read_u32::<BE>().expect("edge length");
                let edge = Edge::from(reader.read_u64::<BE>().expect("read edge"));
                reader.read_u32::<BE>().expect("regret length");
                let regret = reader.read_f32::<BE>().expect("read regret");
                reader.read_u32::<BE>().expect("policy length");
                let policy = reader.read_f32::<BE>().expect("read policy");
                // idempotent insert
                let bucket = Bucket::from((past, abs, future));
                let memory = strategies
                    .entry(bucket)
                    .or_insert_with(Strategy::new)
                    .entry(edge)
                    .or_insert_with(Memory::default);
                memory.set_regret(regret);
                memory.set_policy(policy);
                continue;
            } else {
                break;
            }
        }
        Self {
            iterations: 0,
            strategies,
        }
    }
}

impl Profile {
    /// persist the Profile to disk
    pub fn save(&self, name: &str) {
        log::info!("saving blueprint");
        use byteorder::WriteBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::Write;
        let ref mut file = File::create(format!("{name}.profile.pgcopy")).expect("touch");
        file.write_all(b"PGCOPY\n\xFF\r\n\0").expect("header");
        file.write_u32::<BE>(0).expect("flags");
        file.write_u32::<BE>(0).expect("extension");
        for (bucket, strategy) in self.strategies.iter() {
            for (edge, memory) in strategy.iter() {
                const N_FIELDS: u16 = 6;
                file.write_u16::<BE>(N_FIELDS).unwrap();
                file.write_u32::<BE>(size_of::<u64>() as u32).unwrap();
                file.write_u64::<BE>(u64::from(bucket.0)).unwrap();
                file.write_u32::<BE>(size_of::<u64>() as u32).unwrap();
                file.write_u64::<BE>(u64::from(bucket.1)).unwrap();
                file.write_u32::<BE>(size_of::<u64>() as u32).unwrap();
                file.write_u64::<BE>(u64::from(bucket.2)).unwrap();
                file.write_u32::<BE>(size_of::<u64>() as u32).unwrap();
                file.write_u64::<BE>(u64::from(*edge)).unwrap();
                file.write_u32::<BE>(size_of::<f32>() as u32).unwrap();
                file.write_f32::<BE>(memory.regret()).unwrap();
                file.write_u32::<BE>(size_of::<f32>() as u32).unwrap();
                file.write_f32::<BE>(memory.policy()).unwrap();
            }
        }
        file.write_u16::<BE>(0xFFFF).expect("trailer");
    }
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.strategies
                .iter()
                .map(|(bucket, strategies)| {
                    format!(
                        "{}\n{}",
                        bucket,
                        strategies
                            .iter()
                            .map(|(edge, _)| format!(
                                " ├─{}: {:.2}",
                                edge,
                                self.weight(bucket, edge)
                            ))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl crate::Arbitrary for Profile {
    fn random() -> Self {
        Self {
            iterations: 0,
            strategies: (0..100)
                .map(|_| (Bucket::random(), Strategy::random()))
                .collect(),
        }
    }
}

// pruning stuff
// const P_PRUNE: Probability = 0.95;
// enum Expansion {
//     Explore,
//     Pruning,
// }
// impl From<Phase> for Expansion {
//     fn from(phase: Phase) -> Self {
//         match phase {
//             Phase::Prune if crate::P_PRUNE > rand::thread_rng().gen::<f32>() => Expansion::Pruning,
//             _ => Expansion::Explore,
//         }
//     }
// }
// fn expansion(&self) -> Expansion {
//     Expansion::from(self.phase())
// }
// fn keep(&self, bucket: &Bucket, edge: &Edge) -> bool {
//     match self.expansion() {
//         Expansion::Explore => true,
//         Expansion::Focused => self.regret(bucket, edge) > REGRET_PRUNE,
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Arbitrary;

    #[test]
    fn persistence() {
        let name = "test";
        let file = format!("{}.profile.pgcopy", name);
        let save = Profile::random();
        save.save(name);
        let load = Profile::from(name);
        std::fs::remove_file(file).unwrap();
        assert!(std::iter::empty()
            .chain(save.strategies.iter().zip(load.strategies.iter()))
            .chain(load.strategies.iter().zip(save.strategies.iter()))
            .all(|((s1, l1), (s2, l2))| s1 == s2 && l1 == l2));
    }
}
