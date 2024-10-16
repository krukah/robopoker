use crate::clustering::histogram::Histogram;

/// TODO this is now a full shallow wrapper around a Histogram
/// originaly i thought we shoud separate the last and next Histograms
/// but then the mutation loop changed such that it's not necessary
///
/// `Centroid` is a wrapper around two histograms.
/// We use it to swap the current and next histograms
/// after each iteration of kmeans clustering.
pub struct Centroid {
    last: Histogram,
    // next: Histogram,
}

impl Centroid {
    pub fn reset(&mut self) {
        self.last.destroy();
        // std::mem::swap(&mut self.last, &mut self.next);
    }
    pub fn absorb(&mut self, h: &Histogram) {
        self.last.absorb(h);
        // self.next.absorb(h);
    }
    pub fn histogram(&self) -> &Histogram {
        &self.last
    }
    pub fn is_empty(&self) -> bool {
        self.last.is_empty()
    }
}

impl From<Histogram> for Centroid {
    fn from(h: Histogram) -> Self {
        Self {
            last: h,
            // next: Histogram::default(),
        }
    }
}
