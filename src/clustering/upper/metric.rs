use super::histogram::Histogram;
use super::xor::Pair;
use crate::clustering::abstraction::Abstraction;
use std::collections::HashMap;

/// Trait for defining distance metrics between abstractions and histograms.
///
/// Calculating similarity between abstractions
/// and Earth Mover's Distance (EMD) between histograms. These metrics are
/// essential for clustering algorithms and comparing distributions.
pub trait Metric {
    fn emd(&self, x: &Histogram, y: &Histogram) -> f32;
    fn similarity(&self, x: &Abstraction, y: &Abstraction) -> f32;
}
impl Metric for HashMap<Pair, f32> {
    fn similarity(&self, x: &Abstraction, y: &Abstraction) -> f32 {
        let ref xor = Pair::from((x, y));
        self.get(xor).expect("precalculated distance").clone()
    }
    fn emd(&self, x: &Histogram, y: &Histogram) -> f32 {
        let mut journey = 0.0;
        let mut remains = 1.0;
        for absx in x.domain() {
            let massx = x.weight(absx);
            let mut removed = 0.0;
            for absy in y.domain() {
                let massy = y.weight(absy);
                let distance = self.similarity(absx, absy);
                let delta = massx.min(massy - removed).min(remains);
                journey += delta * distance;
                removed += delta;
                remains -= delta;
                if remains <= 0.0 {
                    break;
                }
            }
            if remains <= 0.0 {
                break;
            }
        }
        journey
    }
}
