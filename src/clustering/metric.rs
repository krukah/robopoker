#![allow(unused)]

use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::xor::Pair;
use std::collections::BTreeMap;

struct River;
impl Transport for River {
    type M = Metric;
    type X = Abstraction; // ::Equity(i8)
    type Y = Abstraction; // ::Equity(i8)
    type P = Histogram;
    type Q = Histogram;
    fn flow(&self, x: &Self::X, y: &Self::Y) -> f32 {
        unreachable!()
    }
    fn cost(&self, x: &Self::P, y: &Self::Q, _: &Self::M) -> f32 {
        // Self::variation(x, y)
        // Self::euclidean(x, y)
        // Self::chisq(x, y)
        Self::variation(x, y)
    }
}
/// different distance metrics over Equity Histograms
/// conveniently have properties of distributions over the [0, 1] interval.
impl River {
    fn variation(x: &Histogram, y: &Histogram) -> f32 {
        // assert!(matches!(x.peek(), Abstraction::Equity(_)));
        // assert!(matches!(y.peek(), Abstraction::Equity(_)));
        let mut total = 0.;
        let mut cdf_x = 0.;
        let mut cdf_y = 0.;
        for abstraction in Abstraction::range() {
            cdf_x += x.weight(abstraction);
            cdf_y += y.weight(abstraction);
            total += (cdf_x - cdf_y).abs();
        }
        total / 2.
    }
    #[allow(unused)]
    fn euclidean(x: &Histogram, y: &Histogram) -> f32 {
        // assert!(matches!(x.peek(), Abstraction::Equity(_)));
        // assert!(matches!(y.peek(), Abstraction::Equity(_)));
        let mut total = 0.;
        for abstraction in Abstraction::range() {
            let x_density = x.weight(abstraction);
            let y_density = y.weight(abstraction);
            total += (x_density - y_density).powi(2);
        }
        total.sqrt()
    }
    #[allow(unused)]
    fn chisq(x: &Histogram, y: &Histogram) -> f32 {
        // assert!(matches!(x.peek(), Abstraction::Equity(_)));
        // assert!(matches!(y.peek(), Abstraction::Equity(_)));
        let mut total = 0.;
        for abstraction in Abstraction::range() {
            let x_density = x.weight(abstraction);
            let y_density = y.weight(abstraction);
            let delta = x_density - y_density;
            total += delta * delta / (x_density + y_density);
        }
        total
    }
}

/// Distance metric for kmeans clustering.
/// encapsulates distance between `Abstraction`s of the "previous" hierarchy,
/// as well as: distance between `Histogram`s of the "current" hierarchy.
#[derive(Default)]
pub struct Metric(pub BTreeMap<Pair, f32>);

impl Metric {
    pub fn emd(&self, source: &Histogram, target: &Histogram) -> f32 {
        match source.peek() {
            Abstraction::Equity(_) => River::variation(source, target),
            Abstraction::Random(_) => self.greedy(source, target),
            Abstraction::Pocket(_) => 1., //  unreachable!("no preflop emd"),
        }
    }

    fn lookup(&self, x: &Abstraction, y: &Abstraction) -> f32 {
        if x == y {
            0.
        } else {
            *self.0.get(&Pair::from((x, y))).unwrap()
        }
    }

    fn image(&self, x: &Histogram) -> BTreeMap<Abstraction, f32> {
        x.support()
            .into_iter()
            .map(|&a| (a, 1. / x.size() as f32))
            .collect()
    }

    fn range(&self, y: &Histogram) -> BTreeMap<Abstraction, f32> {
        y.support()
            .into_iter()
            .map(|&a| (a, y.weight(&a)))
            .collect()
    }

    fn greedy(&self, x: &Histogram, y: &Histogram) -> f32 {
        let mut cost = 0.;
        let mut sources = self.image(x);
        let mut targets = self.range(y);
        'cost: while sources.values().any(|&v| v > 0.) {
            'flow: for (x, pile) in sources
                .iter()
                .cycle()
                .map(|(&x, &pile)| (x, pile))
                .collect::<Vec<_>>()
            {
                let nearest = targets
                    .iter()
                    .filter(|(_, &hole)| hole > 0.)
                    .map(|(&y, &hole)| ((y, hole), self.distance(&x, &y)))
                    .min_by(|(_, y1), (_, y2)| y1.partial_cmp(y2).unwrap());
                match nearest {
                    None => break 'cost,
                    Some(((y, hole), distance)) => {
                        if pile > hole {
                            let earth = hole;
                            cost += distance * earth;
                            *sources.get_mut(&x).unwrap() -= earth;
                            *targets.get_mut(&y).unwrap() = 0.;
                            continue 'flow;
                        } else {
                            let earth = pile;
                            cost += distance * earth;
                            *sources.get_mut(&x).unwrap() = 0.;
                            *targets.get_mut(&y).unwrap() -= earth;
                            continue 'flow;
                        }
                    }
                }
            }
        }
        cost
    }

    /// save profile to disk in a PGCOPY compatible format
    pub fn save(&self, street: Street) {
        log::info!("{:<32}{:<32}", "uploading abstraction metric", street);
        use byteorder::BigEndian;
        use byteorder::WriteBytesExt;
        use std::fs::File;
        use std::io::Write;
        let ref mut file = File::create(format!("{}.metric.pgcopy", street)).expect("new file");
        file.write_all(b"PGCOPY\n\xff\r\n\0").expect("header");
        file.write_u32::<BigEndian>(0).expect("flags");
        file.write_u32::<BigEndian>(0).expect("extension");
        for (pair, distance) in self.0.iter() {
            file.write_u16::<BigEndian>(2).expect("field count");
            file.write_u32::<BigEndian>(8).expect("8-bytes field");
            file.write_i64::<BigEndian>(i64::from(*pair)).expect("pair");
            file.write_u32::<BigEndian>(4).expect("4-bytes field");
            file.write_f32::<BigEndian>(*distance).expect("distance");
        }
        file.write_u16::<BigEndian>(0xFFFF).expect("trailer");
    }
}

trait Support {}
trait Density {
    type X: Support;
    fn density(&self, x: &Self::X) -> f32;
    fn support(&self) -> impl Iterator<Item = &Self::X>;
}
trait Measure {
    type X: Support;
    type Y: Support;
    fn distance(&self, x: &Self::X, y: &Self::Y) -> f32;
}
trait Transport {
    type X: Support;
    type Y: Support;
    type P: Density<X = Self::X>;
    type Q: Density<X = Self::Y>;
    type M: Measure<X = Self::X, Y = Self::Y>;
    fn flow(&self, x: &Self::X, y: &Self::Y) -> f32;
    fn cost(&self, p: &Self::P, q: &Self::Q, m: &Self::M) -> f32 {
        let mut cost = 0.;
        for x in p.support() {
            for y in q.support() {
                let dx = p.density(x);
                let dy = q.density(y);
                let area = m.distance(x, y);
                let flux = self.flow(x, y);
                cost += area * flux * dx * dy;
            }
        }
        cost
    }
}

impl Support for f32 {}
impl Support for Abstraction {}
impl Density for Histogram {
    type X = Abstraction;
    fn density(&self, x: &Self::X) -> f32 {
        self.weight(x)
    }
    fn support(&self) -> impl Iterator<Item = &Self::X> {
        self.support().into_iter()
    }
}
impl Measure for Metric {
    type X = Abstraction;
    type Y = Abstraction;
    fn distance(&self, x: &Self::X, y: &Self::Y) -> f32 {
        match (x, y) {
            (Self::X::Random(_), Self::Y::Random(_)) => self.lookup(x, y),
            (Self::X::Equity(a), Self::Y::Equity(b)) => (a - b).abs() as f32,
            _ => unreachable!("only equity distance for river abstractions"),
        }
    }
}

impl Transport for Metric {
    type M = Self;
    type X = Abstraction;
    type Y = Abstraction;
    type P = Histogram;
    type Q = Histogram;
    fn flow(&self, x: &Self::X, y: &Self::Y) -> f32 {
        let _ = y;
        let _ = x;
        todo!("if we want to we can explicitly use this to calculate P_ij. like if we use a LP solver to get an exact solution, this would yield matrix elements of the optimal P_ij transport plan. alternatively, we can update some internal state while we iterate through self.cost(), but that'd require another method that does some initalization so that it can mutate itself and allow cost + flow to become lookups. effectively, eagerly computing EMD.")
    }
    fn cost(&self, x: &Self::P, y: &Self::Q, _: &Self::M) -> f32 {
        self.greedy(x, y)
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
}
