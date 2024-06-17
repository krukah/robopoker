use super::minimizer::Minimizer;
use crate::cfr::tree::rps::tree::Tree;

pub(crate) struct Trainer {
    t: usize,
    tree: Tree,
    minimizer: Minimizer,
}
impl Trainer {
    pub fn train(t: usize) {
        let tree = Tree::new();
        let minimizer = Minimizer::new(&tree);
        let mut this = Self { minimizer, tree, t };
        let infos = this.tree.infosets();
        // silly way to use training time, unclear if should be minimzer or trainer owned.
        let t = this.t;
        this.t = 0;
        for _ in 0..t {
            for info in infos.iter() {
                this.minimizer.update_regret(info);
                this.minimizer.update_policy(info);
            }
            this.report();
            this.t += 1;
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
