use crate::cfr::rps::action::RpsEdge;
use crate::cfr::rps::info::RpsInfo;
use crate::cfr::rps::minimizer::RpsMinimizer;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::rps::signal::RpsSignal;
use crate::cfr::rps::tree::RpsTree;
use crate::cfr::training::learning::minimizer::Minimizer;
use crate::cfr::training::learning::trainer::Trainer;
use crate::cfr::training::tree::tree::Tree;
use crate::cfr::training::Probability;
use std::collections::HashMap;

/// self-contained training algorithm. owns the changing state of the training process, regrets and profiles. maybe could be consolidated? don't think so, they work at different levels of abstraction... profile: (node -> action -> probability) regrets: (info -> action -> utility)
pub(crate) struct RpsTrainer {
    tree: RpsTree<'static>,
    minimizer: RpsMinimizer,
}

impl RpsTrainer {
    pub fn new() -> Self {
        let tree = RpsTree::new();
        let mut minimizer = RpsMinimizer::new();
        minimizer.scan(&tree);
        Self { minimizer, tree }
    }
}

impl Trainer for RpsTrainer {
    type TMinimizer = RpsMinimizer;
    type TPolicy = HashMap<RpsEdge, Probability>;
    type TProfile = HashMap<RpsSignal, HashMap<RpsEdge, Probability>>;
    type TStrategy = HashMap<RpsSignal, HashMap<RpsEdge, Probability>>;
    type TNode = RpsNode<'static>;
    type TInfo = RpsInfo<'static>;
    type TTree = RpsTree<'static>;
    type TAction = RpsEdge;
    type TPlayer = RpsPlayer;

    fn save(&self) {
        todo!("write to stdout, file, or database")
    }
    fn train(&mut self, n: usize) {
        for _ in 0..n {
            for info in self.tree.infos() {
                self.minimizer.update_regret(info);
                self.minimizer.update_policy(info);
            }
        }
        self.save();
    }
}
