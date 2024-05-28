use crate::cfr::rps::action::RPSEdge;
use crate::cfr::rps::info::RPSInfo;
use crate::cfr::rps::minimizer::RPSMinimizer;
use crate::cfr::rps::node::RPSNode;
use crate::cfr::rps::player::RPSPlayer;
use crate::cfr::rps::policy::RPSPolicy;
use crate::cfr::rps::profile::RPSProfile;
use crate::cfr::rps::strategy::RPSStrategy;
use crate::cfr::rps::tree::RPSTree;
use crate::cfr::training::learning::minimizer::Minimizer;
use crate::cfr::training::learning::trainer::Trainer;
use crate::cfr::training::tree::tree::Tree;

/// self-contained training algorithm. owns the changing state of the training process, regrets and profiles. maybe could be consolidated? don't think so, they work at different levels of abstraction... profile: (node -> action -> probability) regrets: (info -> action -> utility)
pub(crate) struct RPSTrainer {
    tree: RPSTree<'static>,
    minimizer: RPSMinimizer,
}

impl RPSTrainer {
    pub fn new() -> Self {
        let tree = RPSTree::new();
        let mut minimizer = RPSMinimizer::new();
        minimizer.initialize(&tree);
        Self { minimizer, tree }
    }
}

impl Trainer for RPSTrainer {
    type TAction = RPSEdge;
    type TPlayer = RPSPlayer;
    type TPolicy = RPSPolicy;
    type TNode = RPSNode<'static>;
    type TInfo = RPSInfo<'static>;
    type TTree = RPSTree<'static>;
    type TProfile = RPSProfile<'static>;
    type TStrategy = RPSStrategy<'static>;
    type TMinimizer = RPSMinimizer;

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
