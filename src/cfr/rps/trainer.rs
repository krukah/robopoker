use super::{
    action::RPSEdge, info::RPSInfo, node::RPSNode, player::RPSPlayer, policy::RPSPolicy,
    profile::RPSProfile, strategy::RPSStrategy, tree::RPSTree,
};
use crate::cfr::training::{profile::Profile, trainer::Trainer, tree::Tree};

/// self-contained training algorithm. owns the changing state of the training process, regrets and profiles. maybe could be consolidated? don't think so, they work at different levels of abstraction... profile: (node -> action -> probability) regrets: (info -> action -> utility)
pub(crate) struct RPSTrainer<'tree> {
    tree: RPSTree<'tree>,
    profile: RPSProfile<'tree>,
}

impl RPSTrainer<'_> {
    pub fn new() -> Self {
        let tree = RPSTree::new();
        let mut profile = RPSProfile::new();
        profile.walk(&tree);
        Self { profile, tree }
    }
}

impl<'t> Trainer for RPSTrainer<'t> {
    type TPlayer = RPSPlayer;
    type TAction = RPSEdge;
    type TNode = RPSNode<'t>;
    type TInfo = RPSInfo<'t>;
    type TTree = RPSTree<'t>;
    type TPolicy = RPSPolicy;
    type TProfile = RPSProfile<'t>;
    type TStrategy = RPSStrategy<'t>;

    fn save(&self) {
        todo!("write to stdout, file, or database")
    }
    fn train(&mut self, n: usize) {
        for _ in 0..n {
            for info in self.tree.infos() {
                self.profile.update_regret(info);
                self.profile.update_policy(info);
            }
        }
        self.save();
    }
}
