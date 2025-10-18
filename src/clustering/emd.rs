use super::heuristic::Heuristic;
use super::histogram::Histogram;
use super::metric::Metric;
use super::pair::Pair;
use super::sinkhorn::Sinkhorn;
use crate::transport::coupling::Coupling;
use crate::Arbitrary;
use std::collections::BTreeMap;

/// this guy is used just to construct arbitrary metric, histogram, histogram tuples
/// to test transport mechanisms
pub struct EMD(Metric, Histogram, Histogram, Histogram);

impl EMD {
    pub fn metric(&self) -> &Metric {
        &self.0
    }
    pub fn sinkhorn(&self) -> Sinkhorn<'_> {
        Sinkhorn::from((&self.1, &self.2, &self.0)).minimize()
    }
    pub fn heuristic(&self) -> Heuristic<'_> {
        Heuristic::from((&self.1, &self.2, &self.0)).minimize()
    }
    pub fn inner(self) -> (Metric, Histogram, Histogram, Histogram) {
        (self.0, self.1, self.2, self.3)
    }
}

impl Arbitrary for EMD {
    fn random() -> Self {
        // construct random metric satisfying symmetric semipositivity
        let p = Histogram::random();
        let q = Histogram::random();
        let r = Histogram::random();
        let m = Metric::from(
            std::iter::empty()
                .chain(p.support())
                .chain(q.support())
                .chain(r.support())
                .flat_map(|x| {
                    std::iter::empty()
                        .chain(p.support())
                        .chain(q.support())
                        .chain(r.support())
                        .map(move |y| (x, y))
                })
                .filter(|(x, y)| x > y)
                .map(|(x, y)| Pair::from((x, y)))
                .map(|paired| (paired, rand::random::<f32>()))
                .collect::<BTreeMap<_, _>>(),
        );
        Self(m, p, q, r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::observation::Observation;
    use crate::cards::street::Street;
    use crate::clustering::histogram::Histogram;

    /// equity implementation should be
    /// 1. symmetric
    /// 2. positive semidefinite
    /// 3. self-annihilating

    #[test]
    fn is_equity_emd_symmetric() {
        let metric = Metric::default();
        let ref h1 = Histogram::from(Observation::from(Street::Turn));
        let ref h2 = Histogram::from(Observation::from(Street::Turn));
        let d12 = metric.emd(h1, h2);
        let d21 = metric.emd(h2, h1);
        assert!(d12 == d21);
    }
    #[test]
    fn is_equity_emd_positive() {
        let metric = Metric::default();
        let ref h1 = Histogram::from(Observation::from(Street::Turn));
        let ref h2 = Histogram::from(Observation::from(Street::Turn));
        let d12 = metric.emd(h1, h2);
        let d21 = metric.emd(h2, h1);
        assert!(d12 > 0.);
        assert!(d21 > 0.);
    }
    #[test]
    fn is_equity_emd_zero() {
        let metric = Metric::default();
        let h = Histogram::from(Observation::from(Street::Turn));
        let d = metric.emd(&h, &h);
        assert!(d == 0.);
    }

    /// sinkhorn implementation should be
    /// 1. positive semidefinite
    /// 2. approximately symmetric (untested)
    /// 3. approximately self-annihilating
    /// 4. satisfies triangle inequality

    #[test]
    fn is_sinkhorn_emd_triangle() {
        let EMD(metric, h1, h2, h3) = EMD::random();
        let d12 = Sinkhorn::from((&h1, &h2, &metric)).minimize().cost();
        let d23 = Sinkhorn::from((&h2, &h3, &metric)).minimize().cost();
        let d13 = Sinkhorn::from((&h1, &h3, &metric)).minimize().cost();
        assert!(d12 + d23 >= d13, "{} + {} > {}", d12, d23, d13);
        assert!(d12 + d13 >= d23, "{} + {} > {}", d12, d13, d23);
        assert!(d23 + d13 >= d12, "{} + {} > {}", d23, d13, d12);
    }
    #[test]
    fn is_sinkhorn_emd_positive() {
        let EMD(metric, h1, h2, _) = EMD::random();
        let d12 = Sinkhorn::from((&h1, &h2, &metric)).minimize().cost();
        let d21 = Sinkhorn::from((&h2, &h1, &metric)).minimize().cost();
        assert!(d12 > 0., "{}", d12);
        assert!(d21 > 0., "{}", d21);
    }
    #[test]
    fn is_sinkhorn_emd_zero() {
        const TOLERANCE: f32 = 0.01;
        let EMD(metric, h1, h2, _) = EMD::random();
        let d11 = Sinkhorn::from((&h1, &h1, &metric)).minimize().cost();
        let d22 = Sinkhorn::from((&h2, &h2, &metric)).minimize().cost();
        assert!(
            d11 <= TOLERANCE,
            "consider decreasing temp or tolerance\n{d11} {TOLERANCE}",
        );
        assert!(
            d22 <= TOLERANCE,
            "consider decreasing temp or tolerance\n{d22} {TOLERANCE}",
        );
    }

    /// heuristic implementation should be
    /// 1. positive semidefinite
    /// 2. approximately symmetric
    /// 3. exactly self-annihilating
    /// 4. satisfies triangle inequality

    #[test]
    fn is_heuristic_emd_triangle() {
        const TOLERANCE: f32 = 1.25;
        let EMD(metric, h1, h2, h3) = EMD::random();
        let d12 = Heuristic::from((&h1, &h2, &metric)).minimize().cost();
        let d23 = Heuristic::from((&h2, &h3, &metric)).minimize().cost();
        let d13 = Heuristic::from((&h1, &h3, &metric)).minimize().cost();
        assert!(d12 + d23 >= d13 / TOLERANCE, "{} + {} > {}", d12, d23, d13);
        assert!(d12 + d13 >= d23 / TOLERANCE, "{} + {} > {}", d12, d13, d23);
        assert!(d23 + d13 >= d12 / TOLERANCE, "{} + {} > {}", d23, d13, d12);
    }
    #[test]
    fn is_heuristic_emd_positive() {
        let EMD(metric, h1, h2, _) = EMD::random();
        let d12 = Heuristic::from((&h1, &h2, &metric)).minimize().cost();
        let d21 = Heuristic::from((&h2, &h1, &metric)).minimize().cost();
        assert!(d12 > 0.);
        assert!(d21 > 0.);
    }
    #[test]
    fn is_heuristic_emd_zero() {
        let EMD(metric, h1, h2, _) = EMD::random();
        let d11 = Heuristic::from((&h1, &h1, &metric)).minimize().cost();
        let d22 = Heuristic::from((&h2, &h2, &metric)).minimize().cost();
        assert!(d11 == 0.);
        assert!(d22 == 0.);
    }
}
