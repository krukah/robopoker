use crate::*;

/// The single axis swap that turns one abstract (layer, slot) geometry into
/// two on-screen layouts. Landscape: layers are columns flowing left to
/// right. Portrait: layers are rows flowing top to bottom. Glyphs stay
/// upright in both; only positions are projected, never content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orient {
    Landscape,
    Portrait,
}

impl Orient {
    /// Projects an abstract coordinate into screen space — the only place
    /// (layer, slot) meets (x, y).
    pub fn point(&self, at: Axial) -> Point {
        match self {
            Self::Landscape => Point::new(at.along().raw(), at.cross().raw()),
            Self::Portrait => Point::new(at.cross().raw(), at.along().raw()),
        }
    }

    /// Projects an abstract extent into a screen rect: `along` and `cross`
    /// are the box's spans on the layer and slot axes.
    pub fn rect(&self, at: Axial, along: f64, cross: f64) -> Rect {
        match self {
            Self::Landscape => Rect::new(at.along().raw(), at.cross().raw(), along, cross),
            Self::Portrait => Rect::new(at.cross().raw(), at.along().raw(), cross, along),
        }
    }

    /// Selects from a (landscape, portrait) constant pair — the idiom for
    /// the few metrics that differ between orientations.
    pub fn pick(&self, pair: (f64, f64)) -> f64 {
        match self {
            Self::Landscape => pair.0,
            Self::Portrait => pair.1,
        }
    }
}
