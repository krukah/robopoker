use crate::cfr::rps::action::RpsAction;
use crate::cfr::rps::bucket::RpsBucket;
use crate::cfr::rps::info::RpsInfo;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::optimizer::RpsOptimizer;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::rps::tree::RpsTree;
use crate::cfr::traits::training::optimizer::Optimizer;
use crate::cfr::traits::training::trainer::Trainer;
use crate::cfr::traits::tree::tree::Tree;
use crate::cfr::traits::Probability;
use std::collections::HashMap;

/// self-contained training algorithm. owns the changing state of the training process, regrets and profiles. maybe could be consolidated? don't think so, they work at different levels of abstraction... profile: (node -> action -> probability) regrets: (info -> action -> utility)
pub(crate) struct RpsTrainer {
    tree: RpsTree<'static>,
    optimizer: RpsOptimizer,
}

impl RpsTrainer {
    pub fn new() -> Self {
        let mut tree = RpsTree::new();
        tree.expand();
        let mut optimizer = RpsOptimizer::new();
        optimizer.scan(&tree);
        Self { optimizer, tree }
    }
}

impl Trainer for RpsTrainer {
    type TMinimizer = RpsOptimizer;
    type TPolicy = HashMap<RpsAction, Probability>;
    type TProfile = HashMap<RpsBucket, HashMap<RpsAction, Probability>>;
    type TStrategy = HashMap<RpsBucket, HashMap<RpsAction, Probability>>;
    type TNode = RpsNode;
    type TInfo = RpsInfo<'static>;
    type TTree = RpsTree<'static>;
    type TAction = RpsAction;
    type TPlayer = RpsPlayer;

    fn save(&self) {
        todo!("write to stdout, file, or database")
    }
    fn train(&mut self, n: usize) {
        for _ in 0..n {
            for info in self.tree.infos() {
                self.optimizer.update_regret(info);
                self.optimizer.update_policy(info);
            }
        }
        self.save();
    }
}
