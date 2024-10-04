use crate::clustering::histogram::Histogram;

/// `Centroid` is a wrapper around two histograms.
/// We use it to swap the current and next histograms
/// after each iteration of kmeans clustering.
pub struct Centroid {
    last: Histogram,
    next: Histogram,
}

impl Centroid {
    pub fn rotate(&mut self) {
        self.last.destroy();
        std::mem::swap(&mut self.last, &mut self.next);
    }
    pub fn absorb(&mut self, h: &Histogram) {
        self.next.absorb(h);
    }
    pub fn reveal(&self) -> &Histogram {
        &self.last
    }
}

impl From<Histogram> for Centroid {
    fn from(h: Histogram) -> Self {
        Self {
            last: h,
            next: Histogram::default(),
        }
    }
}
