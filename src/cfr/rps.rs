#![allow(dead_code)]

use super::cfr;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub(crate) enum Move {
    R,
    P,
    S,
}

/// Player 1 and Player 2
#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum RPSPlayer {
    P1,
    P2,
}

/// Rock, Paper, Scissors
#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) struct RPSEdge {
    player: RPSPlayer,
    action: Move,
}

/// Shared-lifetime game tree nodes
#[derive(PartialEq, Eq)]
pub(crate) struct RPSNode<'t> {
    chooser: &'t RPSPlayer,
    parent: Option<&'t RPSNode<'t>>,
    precedent: Option<&'t RPSEdge>,
    children: Vec<&'t RPSNode<'t>>,
    available: Vec<&'t RPSEdge>,
}

/// Indistinguishable states belonging to same InfoSets. Effectively, distribution of possile opponent actions.
#[derive(PartialEq, Eq)]
pub(crate) struct RPSInfo<'t> {
    roots: Vec<&'t RPSNode<'t>>,
}

/// Game tree
pub(crate) struct RPSTree<'t> {
    edges: Vec<RPSEdge>,
    nodes: Vec<RPSNode<'t>>,
    infos: HashSet<RPSInfo<'t>>,
}

/// tabular Action > Probability
pub(crate) struct RPSPolicy {
    weights: HashMap<RPSEdge, cfr::Probability>,
}

/// tabular Node > Policy
pub(crate) struct RPSStrategy<'t> {
    policies: HashMap<RPSNode<'t>, RPSPolicy>,
}

/// constant Player > Strategy
pub(crate) struct RPSProfile<'t> {
    strategy: RPSStrategy<'t>,
}

/// self-contained training algorithm
pub(crate) struct RPSTrainer {
    regrets: HashMap<(&'static RPSInfo<'static>, &'static RPSEdge), cfr::Utility>,
    profile: RPSProfile<'static>,
    tree: RPSTree<'static>,
    t: usize,
}

impl cfr::Player for RPSPlayer {}

impl Hash for RPSEdge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.action.hash(state);
    }
}
impl cfr::Action for RPSEdge {
    type APlayer = RPSPlayer;
    fn player(&self) -> &Self::APlayer {
        todo!("use owned values for Player and Action, assume cheap clone")
    }
}

impl Hash for RPSNode<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        0.hash(state)
    }
}
impl cfr::Node for RPSNode<'_> {
    type NPlayer = RPSPlayer;
    type NAction = RPSEdge;
    fn chooser(&self) -> &Self::NPlayer {
        self.chooser
    }
    fn available(&self) -> &Vec<&Self::NAction> {
        &self.available
    }
    fn children(&self) -> &Vec<&Self> {
        &self.children
    }
    fn parent(&self) -> &Option<&Self> {
        &self.parent
    }
    fn precedent(&self) -> &Option<&Self::NAction> {
        &self.precedent
    }
    fn utility(&self, _: &Self::NPlayer) -> cfr::Utility {
        todo!("utility function")
    }
}

impl Hash for RPSInfo<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        0.hash(state)
    }
}
impl<'t> cfr::Info for RPSInfo<'t> {
    type IPlayer = RPSPlayer;
    type IAction = RPSEdge;
    type INode = RPSNode<'t>;
    fn roots(&self) -> &Vec<&Self::INode> {
        &self.roots
    }
}

impl<'t> RPSTree<'t> {
    fn new() -> Self {
        todo!("initialize game tree")
    }
}
impl<'t> cfr::Tree for RPSTree<'t> {
    type TPlayer = RPSPlayer;
    type TEdge = RPSEdge;
    type TNode = RPSNode<'t>;
    type TInfo = RPSInfo<'t>;
    fn infos(&self) -> Vec<&Self::TInfo> {
        self.infos.iter().collect()
    }
}

impl RPSPolicy {
    fn new() -> Self {
        todo!("initialize policy")
    }
}
impl cfr::Policy for RPSPolicy {
    type PAction = RPSEdge;
    fn weights(&self, action: &Self::PAction) -> cfr::Probability {
        match self.weights.get(action) {
            None => 0.0,
            Some(utility) => *utility,
        }
    }
}

impl RPSStrategy<'_> {
    fn new() -> Self {
        todo!("initialize strategy")
    }
}
impl<'t> cfr::Strategy for RPSStrategy<'t> {
    type SPlayer = RPSPlayer;
    type SAction = RPSEdge;
    type SPolicy = RPSPolicy;
    type SNode = RPSNode<'t>;
    fn policy(&self, node: &Self::SNode) -> &Self::SPolicy {
        match self.policies.get(node) {
            None => todo!("set default policy, uniform over node.available()"),
            Some(policy) => policy,
        }
    }
}

impl RPSProfile<'_> {
    fn new() -> Self {
        todo!("initialize profile")
    }
}
impl<'t> cfr::Profile for RPSProfile<'t> {
    type PPlayer = RPSPlayer;
    type PAction = RPSEdge;
    type PPolicy = RPSPolicy;
    type PNode = RPSNode<'t>;
    type PInfo = RPSInfo<'t>;
    type PStrategy = RPSStrategy<'t>;
    fn strategy(&self, _: &Self::PPlayer) -> &Self::PStrategy {
        &self.strategy
    }
}

impl RPSTrainer {
    pub fn new() -> Self {
        todo!("initialize trainer")
    }
}
impl cfr::Trainer for RPSTrainer {
    type TPlayer = RPSPlayer;
    type TAction = RPSEdge;
    type TNode = RPSNode<'static>;
    type TInfo = RPSInfo<'static>;
    type TTree = RPSTree<'static>;
    type TPolicy = RPSPolicy;
    type TProfile = RPSProfile<'static>;
    type TStrategy = RPSStrategy<'static>;

    fn save(&self) {
        todo!()
    }
    fn update_regrets(&mut self) {
        todo!()
    }
    fn update_profile(&mut self) {
        todo!()
    }

    fn profile(&self) -> &Self::TProfile {
        &self.profile
    }
    fn tree(&self) -> &Self::TTree {
        &self.tree
    }
    fn prev_regret(&self, info: &Self::TInfo, action: &Self::TAction) -> cfr::Utility {
        match self.regrets.get(&(info, action)) {
            None => 0.0,
            Some(regret) => *regret,
        }
    }
}
