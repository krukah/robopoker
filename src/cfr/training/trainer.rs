use super::minimizer::Minimizer;
use crate::cfr::tree::rps::tree::Tree;

pub(crate) struct Trainer {
    tree: Tree,
    solver: Minimizer,
}
impl Trainer {
    pub fn train(epochs: usize) {
        let tree = Tree::new();
        let solver = Minimizer::new(&tree);
        let mut trainer = Self { solver, tree };
        let infos = trainer.tree.infosets();
        for t in 0..epochs {
            //? don't love how epoch is contagious across Trainer < Minimizer < Profile > >
            trainer.solver.update_epoch(t);
            for info in infos.iter() {
                trainer.solver.update_regret(info);
            }
            for info in infos.iter() {
                trainer.solver.update_policy(info);
            }
            trainer.solver.report();
        }
    }
}
