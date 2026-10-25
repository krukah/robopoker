use crate::Drift;
use crate::Prune;

/// Hamerly (2010) bounds: an upper bound to the assigned centroid and a single lower bound to
/// every other one. A point is skipped when `u(x) ≤ max(s(c(x)), l(x))`; otherwise it either
/// tightens `u` or rescans all K centroids.
#[derive(Debug, Clone, Copy, Default)]
pub struct Hamerly<const K: usize> {
    j: usize,
    upper: f32,
    lower: f32,
}

impl<const K: usize> Prune<K> for Hamerly<K> {
    fn j(&self) -> usize {
        self.j
    }

    fn settle<F>(&mut self, _: &[[f32; K]; K], midpoints: &[f32; K], distance: F)
    where
        F: Fn(usize) -> f32,
    {
        let floor = midpoints[self.j()].max(self.lower());
        if self.upper() <= floor {
            return;
        }
        *self = self.tighten(distance(self.j()));
        if self.upper() <= floor {
            return;
        }
        *self = (0..K)
            .filter(|k| *k != self.j())
            .map(|k| (k, distance(k)))
            .fold(self.tighten(self.upper()).unbound(), Self::fold);
    }

    fn update(&mut self, drift: &Drift<K>) {
        *self = Self {
            j: self.j(),
            upper: self.upper() + drift.as_array()[self.j()],
            lower: (self.lower() - drift.max_except(self.j())).max(0.0),
        };
    }
}

impl<const K: usize> Hamerly<K> {
    fn upper(&self) -> f32 {
        self.upper
    }

    fn lower(&self) -> f32 {
        self.lower
    }

    fn tighten(self, upper: f32) -> Self {
        Self { upper, ..self }
    }

    fn unbound(self) -> Self {
        Self {
            lower: f32::MAX,
            ..self
        }
    }

    /// One more centroid at distance `d`: the nearer of it and the assigned one takes the
    /// assignment, the farther bounds every other centroid from below.
    fn fold(self, (k, d): (usize, f32)) -> Self {
        if d < self.upper() {
            Self {
                j: k,
                upper: d,
                lower: self.upper(),
            }
        } else {
            Self {
                lower: self.lower().min(d),
                ..self
            }
        }
    }
}

impl<const K: usize> From<(usize, f32)> for Hamerly<K> {
    fn from((j, upper): (usize, f32)) -> Self {
        Self { j, upper, lower: 0.0 }
    }
}
