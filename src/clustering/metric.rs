use super::equity::Equity;
use super::kontorovich::Kontorovich;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::xor::Pair;
use crate::transport::coupling::Coupling;
use crate::transport::density::Density;
use crate::transport::measure::Measure;
use crate::Arbitrary;
use crate::Probability;
use crate::Utility;
use rand::Rng;
use std::collections::BTreeMap;

type Distance = f32;

/// Distance metric for kmeans clustering.
/// encapsulates distance between `Abstraction`s of the "previous" hierarchy,
/// as well as: distance between `Histogram`s of the "current" hierarchy.
#[derive(Default)]
pub struct Metric(BTreeMap<Pair, Distance>);

impl From<BTreeMap<Pair, Distance>> for Metric {
    fn from(metric: BTreeMap<Pair, Distance>) -> Self {
        Self(metric)
    }
}

impl Measure for Metric {
    type X = Abstraction;
    type Y = Abstraction;
    fn distance(&self, x: &Self::X, y: &Self::Y) -> Distance {
        match (x, y) {
            (Self::X::Learned(_), Self::Y::Learned(_)) => self.lookup(x, y),
            (Self::X::Percent(_), Self::Y::Percent(_)) => Equity.distance(x, y),
            (Self::X::PreFlop(_), Self::Y::PreFlop(_)) => unreachable!("no preflop distance"),
            _ => unreachable!(),
        }
    }
}

impl Coupling for Metric {
    type M = Self;
    type X = Abstraction;
    type Y = Abstraction;
    type P = Histogram;
    type Q = Histogram;
    fn flow(&self, _: &Self::X, _: &Self::Y) -> f32 {
        todo!("implementation would require us to eagerly calculate and store P_ij in Metric constructor(s). e.g. if we use a LP solver to get an exact solution, then Metric would store matrix elements of the optimal P_ij transport plan.")
    }
    fn cost(&self, x: &Self::P, y: &Self::Q, _: &Self::M) -> Distance {
        self.emd(x, y)
    }
}

impl Metric {
    pub fn transportation() -> (Metric, Histogram, Histogram) {
        use rand::thread_rng;
        // construct random metric satisfying symmetric semipositivity
        const MAX_DISTANCE: f32 = 1.0;
        const CORRELATIONS: f64 = 0.1;
        const OVERLAPPINGS: usize = 16;
        let mut rng = thread_rng();
        let mut metric = BTreeMap::new();
        let p = Histogram::random();
        let q = Histogram::random();
        // introduce some overlap in support
        let q = p
            .support()
            .collect::<Vec<_>>()
            .into_iter()
            .cycle()
            .take(OVERLAPPINGS)
            .filter(|_| rng.gen_bool(CORRELATIONS))
            .fold(q, |h, x| h.increment(*x));
        println!("P {:?}", p);
        println!("Q {:?}", q);
        for x in p.support() {
            for y in q.support() {
                let dist = rng.gen_range(0.0..MAX_DISTANCE);
                let pair = Pair::from((x, y));
                metric.insert(pair, dist);
            }
        }
        let m = Metric(metric);
        (m, p, q)
    }

    pub fn emd(&self, source: &Histogram, target: &Histogram) -> Distance {
        match source.peek() {
            Abstraction::Learned(_) => self.sinkhorn(source, target),
            Abstraction::Percent(_) => Equity::variation(source, target),
            Abstraction::PreFlop(_) => unreachable!("no preflop emd"),
        }
    }
    fn lookup(&self, x: &Abstraction, y: &Abstraction) -> Distance {
        if x == y {
            0.
        } else {
            self.0
                .get(&Pair::from((x, y)))
                .copied()
                .expect("missing abstraction pair")
        }
    }

    /// greedy algorithm for optimimal transport.
    /// my favorite interpretation of this is in the formalization
    /// of bipartite matching. we have a Left set of sources and a
    /// Right set of targets. we want to find a way to pair each source
    /// to a target under the constraint of conserving probability mass.
    ///
    /// for each step of the algorithm, we pair the next source to its nearest target.
    /// we move as much mass as we can. we then continue to an arbitrary next source, and repeat.
    ///
    /// this is O(N * M) in (sources, targets) size. for us these are both
    /// the K number of clusters at different layers of the hierarchy.
    ///
    /// if anything, the most expensive part about this is the double BTreeMap
    /// allocation. considering that we do this a few billion times it is
    /// probably worth optimizing into a 0-alloc implementation.
    ///
    /// also, it turns out this algorithm sucks in worst case. like it's just not at all
    /// a reasonable heuristic, even in pathological 1D trivial cases.
    fn greedy(&self, x: &Histogram, y: &Histogram) -> Distance {
        let mut cost = 0.;
        let mut pile = x.normalize();
        let mut sink = y.normalize();
        'cost: while pile.values().any(|&dx| dx > 0.) {
            'pile: for (x, dx) in pile
                .iter_mut()
                .filter(|(_, dx)| **dx > 0.)
                .map(|(&x, dx)| (x, dx))
                .collect::<Vec<_>>()
            {
                match sink
                    .iter_mut()
                    .filter(|(_, dy)| **dy > 0.)
                    .map(|(&y, dy)| ((y, dy), self.distance(&x, &y)))
                    .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap())
                {
                    None => break 'cost,
                    Some(((_, dy), distance)) => {
                        let flow = f32::min(*dx, *dy);
                        *dx -= flow;
                        *dy -= flow;
                        cost += flow * distance;
                        continue 'pile;
                    }
                }
            }
        }
        cost
    }
}

impl Density for BTreeMap<Abstraction, Distance> {
    type S = Abstraction;
    fn density(&self, x: &Self::S) -> Distance {
        self.get(x).copied().unwrap_or(0.)
    }
    fn support(&self) -> impl Iterator<Item = &Self::S> {
        self.keys()
    }
}
type A = Abstraction;
type M = BTreeMap<A, f32>;

impl Metric {
    ///
    /// Sinkhorn algorithm for optimal transport.
    fn sinkhorn(&self, x: &Histogram, y: &Histogram) -> f32 {
        let (u, v) = self.transport(x, y);
        u.support()
            .map(|i| {
                v.support()
                    // .inspect(|j| println!("ELEME {}{}", i, j))
                    .map(|j| {
                        self.kernel(i, j) // .
                        * u.density(i) // .
                        * v.density(j) // .
                    })
                    // .inspect(|x| println!("INNER {:16e}", x))
                    .sum::<Utility>()
            })
            // .inspect(|x| println!("OUTER {:16e}", x))
            .sum::<Utility>()
            .ln()
    }
    fn kernel(&self, i: &A, j: &A) -> Probability {
        (-self.distance(&i, &j) / self.epsilon()).exp()
    }
    #[rustfmt::skip]
    fn delta(&self, x: &M, u: &M, y: &M, v: &M) -> Utility {
        println!("PREV_U {}", x.iter().map(|(k,v)| format!("{} {:.2e}", k.to_string()[k.to_string().len()-2..].to_owned(), v)).collect::<Vec<_>>().join(", "));
        println!("PREV_V {}", y.iter().map(|(k,v)| format!("{} {:.2e}", k.to_string()[k.to_string().len()-2..].to_owned(), v)).collect::<Vec<_>>().join(", "));
        println!("NEXT_U {}", u.iter().map(|(k,v)| format!("{} {:.2e}", k.to_string()[k.to_string().len()-2..].to_owned(), v)).collect::<Vec<_>>().join(", "));
        println!("NEXT_V {}", v.iter().map(|(k,v)| format!("{} {:.2e}", k.to_string()[k.to_string().len()-2..].to_owned(), v)).collect::<Vec<_>>().join(", "));
        0. // .
            + x.iter()
                .map(|(i, _)| (x.density(i) - u.density(i)).abs())
                .sum::<Utility>() / x.len() as Utility
            + y.iter()
                .map(|(j, _)| (y.density(j) - v.density(j)).abs())
                .sum::<Utility>() / y.len() as Utility
    }
    #[rustfmt::skip]
    fn transport(&self, x: &Histogram, y: &Histogram) -> (M, M) {
        println!("TRANSPORT");
        let mut u = self.uniform(x);
        let mut v = self.uniform(y);
        let x = x.normalize();
        let y = y.normalize();
        for _ in 0..10 {
            let prev_u = u.clone();
            let prev_v = v.clone();
            v = self.scale(&y, &u);
            u = self.scale(&x, &v);
            let delta = self.delta(&prev_u, &u, &prev_v, &v);
            println!("DELTA {:8e}", delta);
            if delta < self.tolerance() {
                break;
            }
        }
        (u, v)
    }
    /// O(N * M) in (sources, targets), but embarrasingly parallel.
    fn scale(&self, x: &M, y: &M) -> M {
        println!(
            "X {}",
            x.iter()
                .map(|(k, v)| format!(
                    "{} {:.2e}",
                    k.to_string()[k.to_string().len() - 2..].to_owned(),
                    v
                ))
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!(
            "Y {}",
            y.iter()
                .map(|(k, v)| format!(
                    "{} {:.2e}",
                    k.to_string()[k.to_string().len() - 2..].to_owned(),
                    v
                ))
                .collect::<Vec<_>>()
                .join(", ")
        );
        for (p, d) in self.0.iter() {
            println!("METR {:?} -> {d}", p);
        }
        x.support()
            .map(|i| {
                // element-wise x axis
                x.density(i)
                    / y.support()
                        .map(|j| {
                            let pair = Pair::from((i, j));
                            println!("PAIR {:?} i={:?} j={:?}", pair, i, j);
                            // sum over & marginalize y axis
                            let dy = y.density(j);
                            let k = self.kernel(i, j);
                            dy * k
                        })
                        .sum::<Probability>()
            })
            // .inspect(|x| println!("SCALE {:2e}", x))
            .zip(x.support().copied())
            .map(|(dx, i)| (i, dx))
            .collect::<BTreeMap<_, _>>()
    }
    fn uniform(&self, x: &Histogram) -> M {
        Histogram::from(x.support().copied().collect::<Vec<A>>()).normalize()
    }
    fn epsilon(&self) -> Utility {
        1.
    }
    fn tolerance(&self) -> Utility {
        1e-12
    }
}

impl Metric {
    /// save profile to disk in a PGCOPY compatible format
    pub fn save(&self, street: Street) {
        println!("{:<32}{:<32}", "saving metric", street);
        use byteorder::WriteBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::Write;
        let ref mut file = File::create(format!("{}.metric.pgcopy", street)).expect("touch");
        file.write_all(b"PGCOPY\n\xFF\r\n\0").expect("header");
        file.write_u32::<BE>(0).expect("flags");
        file.write_u32::<BE>(0).expect("extension");
        for (pair, distance) in self.0.iter() {
            const N_FIELDS: u16 = 2;
            file.write_u16::<BE>(N_FIELDS).unwrap();
            file.write_u32::<BE>(size_of::<i64>() as u32).unwrap();
            file.write_i64::<BE>(i64::from(*pair)).unwrap();
            file.write_u32::<BE>(size_of::<f32>() as u32).unwrap();
            file.write_f32::<BE>(*distance).unwrap();
        }
        file.write_u16::<BE>(0xFFFF).expect("trailer");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::observation::Observation;
    use crate::cards::street::Street;
    use crate::clustering::histogram::Histogram;

    #[test]
    fn is_turn_emd_zero() {
        let metric = Metric::default();
        let obs = Observation::from(Street::Turn);
        let ref h1 = Histogram::from(obs.clone());
        let ref h2 = Histogram::from(obs.clone());
        assert!(metric.emd(h1, h2) == 0.);
        assert!(metric.emd(h2, h1) == 0.);
    }

    #[test]
    fn is_turn_emd_positive() {
        let metric = Metric::default();
        let ref h1 = Histogram::from(Observation::from(Street::Turn));
        let ref h2 = Histogram::from(Observation::from(Street::Turn));
        assert!(metric.emd(h1, h2) > 0.);
        assert!(metric.emd(h2, h1) > 0.);
    }

    #[test]
    fn is_turn_emd_symmetric() {
        let metric = Metric::default();
        let ref h1 = Histogram::from(Observation::from(Street::Turn));
        let ref h2 = Histogram::from(Observation::from(Street::Turn));
        assert!(metric.emd(h1, h2) == metric.emd(h2, h1));
    }

    #[test]
    fn is_transport_emd_zero() {
        let (metric, h1, h2) = Metric::transportation();
        assert!(metric.emd(&h1, &h1) == 0.);
        assert!(metric.emd(&h2, &h2) == 0.);
    }

    #[test]
    fn is_transport_emd_positive() {
        let (metric, h1, h2) = Metric::transportation();
        assert!(metric.emd(&h1, &h2) > 0.);
        assert!(metric.emd(&h2, &h1) > 0.);
    }

    #[test]
    fn is_transport_emd_symmetric() {
        let (metric, h1, h2) = Metric::transportation();
        assert!(metric.emd(&h1, &h2) == metric.emd(&h2, &h1));
    }
}
