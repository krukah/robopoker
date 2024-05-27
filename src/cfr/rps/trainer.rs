use super::{
    action::RPSEdge, info::RPSInfo, node::RPSNode, optimizer::RPSOptimzer, player::RPSPlayer,
    policy::RPSPolicy, profile::RPSProfile, strategy::RPSStrategy, tree::RPSTree,
};
use crate::cfr::training::{optimizer::Optimizer, trainer::Trainer, tree::Tree};

/// self-contained training algorithm. owns the changing state of the training process, regrets and profiles. maybe could be consolidated? don't think so, they work at different levels of abstraction... profile: (node -> action -> probability) regrets: (info -> action -> utility)
pub(crate) struct RPSTrainer<'tree> {
    tree: RPSTree<'tree>,
    optimizer: RPSOptimzer<'tree>,
}

impl RPSTrainer<'_> {
    pub fn new() -> Self {
        let tree = RPSTree::new();
        let optimizer = RPSOptimzer::new(&tree);
        Self { optimizer, tree }
    }
}

impl<'t> Trainer for RPSTrainer<'t> {
    type TAction = RPSEdge;
    type TPlayer = RPSPlayer;
    type TPolicy = RPSPolicy;
    type TNode = RPSNode<'t>;
    type TInfo = RPSInfo<'t>;
    type TTree = RPSTree<'t>;
    type TProfile = RPSProfile<'t>;
    type TStrategy = RPSStrategy<'t>;
    type TOptimizer = RPSOptimzer<'t>;

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
