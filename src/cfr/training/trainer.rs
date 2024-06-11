use super::minimizer::Minimizer;
use crate::cfr::tree::tree::Tree;

pub(crate) struct Trainer {
    t: usize,
    tree: Tree,
    minimizer: Minimizer,
}
impl Trainer {
    pub fn new(t: usize) -> Self {
        let tree = Tree::new();
        let minimizer = Minimizer::new(&tree);
        Self { minimizer, tree, t }
    }
    pub fn train(&mut self) {
        // silly way to use training time, unclear if should be minimzer or trainer owned.
        let t = self.t;
        self.t = 0;
        for _ in 0..t {
            for info in self.tree.infosets() {
                self.minimizer.update_regret(info);
                self.minimizer.update_policy(info);
            }
            self.report();
            self.t += 1;
        }
    }
    fn report(&self) {
        if self.t % 10_000 == 100 {
            println!("T{}", self.t);
            for (bucket, strategy) in self.minimizer.average().0.iter() {
                for (action, weight) in strategy.0.iter() {
                    println!("Bucket {:?}  {:?}: {:.4?}", bucket, action, weight);
                }
                break;
            }
        }
    }
}
