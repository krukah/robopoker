use crate::*;

/// Tile edge length.
pub const CELL: f64 = 32.0;

/// Slot-axis pitch between tiles within a layer.
pub const PIT: f64 = 36.0;

/// Layer-axis gap between adjacent layers.
pub const GAP: f64 = 10.0;

/// Intra-layer seam: the slot-axis gap between adjacent tiles (a layer of
/// `n` tiles spans `n * PIT - SEAM`).
pub const SEAM: f64 = PIT - CELL;

/// Layer-axis canvas origin (landscape, portrait).
pub const HEAD: (f64, f64) = (40.0, 30.0);

/// Slot-axis canvas origin (landscape, portrait) — room for near labels.
pub const SIDE: (f64, f64) = (44.0, 82.0);

/// Canvas margin trailing the last layer (landscape, portrait).
pub const TAIL: (f64, f64) = (40.0, 22.0);

/// Canvas margin past the far slot edge (landscape, portrait) — room for
/// far labels.
pub const BRIM: (f64, f64) = (60.0, 66.0);

/// Label offset beyond the slot-axis extremes.
pub const MARGIN: f64 = 14.0;

/// Background-band bleed past a layer's tiles on the layer axis.
pub const SKIRT: f64 = 5.0;

/// Background-band bleed past the slot-axis extremes.
pub const FLARE: f64 = 8.0;

/// The tree's metric: one abstract geometry in (layer, slot) space,
/// projected to screen through an [`Orient`]. Pure — this type emits only
/// points and rects, never color or text, and every screen coordinate it
/// produces flows through the single projection in [`Orient`].
#[derive(Debug, Clone)]
pub struct Lay {
    orient: Orient,
    counts: Vec<usize>,
    peak: f64,
    head: f64,
    side: f64,
}

impl Lay {
    pub fn new(orient: Orient, counts: Vec<usize>) -> Self {
        let peak = counts.iter().map(|n| *n as f64 * PIT - SEAM).fold(0.0, f64::max);
        Self {
            orient,
            counts,
            peak,
            head: orient.pick(HEAD),
            side: orient.pick(SIDE),
        }
    }

    pub fn orient(&self) -> Orient {
        self.orient
    }

    fn counts(&self) -> &[usize] {
        &self.counts
    }

    fn peak(&self) -> f64 {
        self.peak
    }

    fn head(&self) -> f64 {
        self.head
    }

    fn side(&self) -> f64 {
        self.side
    }

    /// Canvas size (width, height).
    pub fn size(&self) -> (f64, f64) {
        let reach = self.head() + (self.counts().len() - 1) as f64 * (CELL + GAP) + CELL + self.orient().pick(TAIL);
        let sweep = self.side() + self.peak() + self.orient().pick(BRIM);
        match self.orient() {
            Orient::Landscape => (reach, sweep),
            Orient::Portrait => (sweep, reach),
        }
    }

    /// Layer-axis position of layer `i`.
    fn along(&self, i: usize) -> Along {
        Along::new(self.head() + i as f64 * (CELL + GAP))
    }

    /// Slot-axis extent of layer `i`'s tiles.
    fn extent(&self, i: usize) -> f64 {
        self.counts()[i] as f64 * PIT - SEAM
    }

    /// Slot-axis start of layer `i`'s centered content.
    fn start(&self, i: usize) -> f64 {
        self.side() + (self.peak() - self.extent(i)) / 2.0
    }

    /// Slot-axis center of tile `k` in layer `i`.
    pub fn slot(&self, i: usize, k: Slot) -> Cross {
        Cross::new(self.start(i) + k.raw() as f64 * PIT + CELL / 2.0)
    }

    /// Slot-axis center of the whole layer — the anchor of chance layers.
    pub fn center(&self, i: usize) -> Cross {
        Cross::new(self.start(i) + self.extent(i) / 2.0)
    }

    /// Screen rect of tile `k` in layer `i`.
    pub fn tile(&self, i: usize, k: Slot) -> Rect {
        self.orient()
            .rect(Axial::new(self.along(i), Cross::new(self.start(i) + k.raw() as f64 * PIT)), CELL, CELL)
    }

    /// The elbow connector from slot-axis position `from` at layer `a` to
    /// `to` at layer `b`: out the far edge, across the midline, in the near
    /// edge — all in abstract space, projected at the end.
    pub fn elbow(&self, a: usize, from: Cross, b: usize, to: Cross) -> Vec<Point> {
        let out = Along::new(self.along(a).raw() + CELL);
        let mid = Along::new(f64::midpoint(out.raw(), self.along(b).raw()));
        vec![
            self.orient().point(Axial::new(out, from)),
            self.orient().point(Axial::new(mid, from)),
            self.orient().point(Axial::new(mid, to)),
            self.orient().point(Axial::new(self.along(b), to)),
        ]
    }

    /// The full-slot-axis background band behind a chance layer.
    pub fn band(&self, i: usize) -> Rect {
        self.orient().rect(
            Axial::new(Along::new(self.along(i).raw() - SKIRT), Cross::new(self.side() - FLARE)),
            CELL + 2.0 * SKIRT,
            self.peak() + 2.0 * FLARE,
        )
    }

    /// A label anchor on one margin of the slot axis, centered on layer `i`.
    /// Position projects like everything else; only the text anchor is
    /// orientation-aware, because anchoring is inherently an x-axis affair.
    pub fn margin(&self, i: usize, brink: Brink) -> (Point, Anchor) {
        let cross = match brink {
            Brink::Near => Cross::new(self.side() - MARGIN),
            Brink::Far => Cross::new(self.side() + self.peak() + MARGIN),
        };
        let anchor = match (self.orient(), brink) {
            (Orient::Landscape, _) => Anchor::Middle,
            (Orient::Portrait, Brink::Near) => Anchor::End,
            (Orient::Portrait, Brink::Far) => Anchor::Start,
        };
        (
            self.orient()
                .point(Axial::new(Along::new(self.along(i).raw() + CELL / 2.0), cross)),
            anchor,
        )
    }

    /// Which brinks the (actor, street) labels occupy — they swap sides
    /// under transposition, and this is the one place that rule lives.
    fn brinks(&self) -> (Brink, Brink) {
        match self.orient() {
            Orient::Landscape => (Brink::Near, Brink::Far),
            Orient::Portrait => (Brink::Far, Brink::Near),
        }
    }

    /// Actor-label anchor for layer `i`.
    pub fn actor(&self, i: usize) -> (Point, Anchor) {
        self.margin(i, self.brinks().0)
    }

    /// Street-label anchor for a chance layer `i`.
    pub fn street(&self, i: usize) -> (Point, Anchor) {
        self.margin(i, self.brinks().1)
    }
}
