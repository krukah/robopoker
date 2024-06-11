use super::minimizer::M;
use crate::cfr::tree::tree::T;

pub(crate) struct X {
    t: usize,
    tree: T,
    minimizer: M,
}
impl X {
    pub fn new(t: usize) -> Self {
        let tree = T::new();
        let minimizer = M::new(&tree);
        Self { minimizer, tree, t }
    }
    pub fn train(&mut self) {
        for _ in 0..self.t {
            for info in self.tree.infosets() {
                self.minimizer.update_regret(info);
                self.minimizer.update_policy(info);
            }
            self.report();
            self.t += 1;
        }
    }
    fn report(&self) {
        if self.t % 1_000 == 0 {
            println!("T{}", self.t);
            for (bucket, strategy) in self.minimizer.average().0.iter() {
                for (action, weight) in strategy.0.iter() {
                    println!("Bucket {:?}  {:?}: {:.3?}", bucket, action, weight);
                }
                break;
            }
        }
    }
}
