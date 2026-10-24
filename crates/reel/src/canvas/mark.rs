use crate::*;

/// Default corner radius of a block.
pub const BEND: f64 = 2.0;

/// Default font weight of a label.
pub const BOOK: u32 = 400;

/// A scene primitive: pure data renderable as an SVG element now and as a
/// reactive view later. Frames are vectors of marks, so frame equality (the
/// no-jitter dedup) is structural equality of marks.
#[derive(Debug, Clone, PartialEq)]
pub enum Mark {
    Block {
        rect: Rect,
        fill: String,
        fade: f64,
        rim: Option<(String, f64)>,
        bend: f64,
    },
    Label {
        spot: Point,
        text: String,
        size: f64,
        fill: String,
        anchor: Anchor,
        face: Face,
        heft: u32,
        fade: f64,
    },
    Trace {
        spine: Vec<Point>,
        stroke: String,
        width: f64,
    },
    Slice {
        pin: Point,
        radius: f64,
        from: f64,
        upto: f64,
        fill: String,
        rim: Option<(String, f64)>,
    },
    Disc {
        pin: Point,
        radius: f64,
        fill: String,
        rim: Option<(String, f64)>,
    },
}

impl Mark {
    /// A pie wedge; chain [`Self::rim`] for the seam between slices.
    pub fn slice(pin: Point, radius: f64, from: f64, upto: f64, fill: impl Into<String>) -> Self {
        Self::Slice {
            pin,
            radius,
            from,
            upto,
            fill: fill.into(),
            rim: None,
        }
    }

    /// A filled circle; chain [`Self::rim`].
    pub fn disc(pin: Point, radius: f64, fill: impl Into<String>) -> Self {
        Self::Disc {
            pin,
            radius,
            fill: fill.into(),
            rim: None,
        }
    }

    /// Translates this mark by (dx, dy) — the composition primitive that
    /// lets several films share one canvas.
    pub fn shift(self, dx: f64, dy: f64) -> Self {
        match self {
            Self::Block {
                rect,
                fill,
                fade,
                rim,
                bend,
            } => Self::Block {
                rect: rect.shift(dx, dy),
                fill,
                fade,
                rim,
                bend,
            },
            Self::Label {
                spot,
                text,
                size,
                fill,
                anchor,
                face,
                heft,
                fade,
            } => Self::Label {
                spot: spot.shift(dx, dy),
                text,
                size,
                fill,
                anchor,
                face,
                heft,
                fade,
            },
            Self::Trace { spine, stroke, width } => Self::Trace {
                spine: spine.into_iter().map(|p| p.shift(dx, dy)).collect(),
                stroke,
                width,
            },
            Self::Slice {
                pin,
                radius,
                from,
                upto,
                fill,
                rim,
            } => Self::Slice {
                pin: pin.shift(dx, dy),
                radius,
                from,
                upto,
                fill,
                rim,
            },
            Self::Disc { pin, radius, fill, rim } => Self::Disc {
                pin: pin.shift(dx, dy),
                radius,
                fill,
                rim,
            },
        }
    }

    /// A filled rectangle; chain [`Self::fade`], [`Self::rim`], [`Self::bend`].
    pub fn block(rect: Rect, fill: impl Into<String>) -> Self {
        Self::Block {
            rect,
            fill: fill.into(),
            fade: 1.0,
            rim: None,
            bend: BEND,
        }
    }

    /// A text glyph; chain [`Self::anchor`], [`Self::face`], [`Self::heft`], [`Self::fade`].
    pub fn label(spot: Point, text: impl Into<String>, size: f64, fill: impl Into<String>) -> Self {
        Self::Label {
            spot,
            text: text.into(),
            size,
            fill: fill.into(),
            anchor: Anchor::Middle,
            face: Face::Text,
            heft: BOOK,
            fade: 1.0,
        }
    }

    /// An unfilled polyline (elbows, spines).
    pub fn trace(spine: Vec<Point>, stroke: impl Into<String>, width: f64) -> Self {
        Self::Trace {
            spine,
            stroke: stroke.into(),
            width,
        }
    }

    /// Sets opacity on blocks and labels.
    pub fn fade(mut self, level: f64) -> Self {
        match &mut self {
            Self::Block { fade, .. } | Self::Label { fade, .. } => *fade = level,
            _ => {}
        }
        self
    }

    /// Sets a stroke outline on a block, slice, or disc.
    pub fn rim(mut self, stroke: impl Into<String>, width: f64) -> Self {
        match &mut self {
            Self::Block { rim, .. } | Self::Slice { rim, .. } | Self::Disc { rim, .. } => {
                *rim = Some((stroke.into(), width));
            }
            _ => {}
        }
        self
    }

    /// Sets corner radius on a block.
    pub fn bend(mut self, radius: f64) -> Self {
        if let Self::Block { bend, .. } = &mut self {
            *bend = radius;
        }
        self
    }

    /// Sets text anchoring on a label.
    pub fn anchor(mut self, at: Anchor) -> Self {
        if let Self::Label { anchor, .. } = &mut self {
            *anchor = at;
        }
        self
    }

    /// Sets the font role on a label.
    pub fn face(mut self, role: Face) -> Self {
        if let Self::Label { face, .. } = &mut self {
            *face = role;
        }
        self
    }

    /// Sets font weight on a label.
    pub fn heft(mut self, weight: u32) -> Self {
        if let Self::Label { heft, .. } = &mut self {
            *heft = weight;
        }
        self
    }

    /// Renders this mark as one SVG element.
    pub fn svg(&self) -> String {
        match self {
            Self::Block {
                rect,
                fill,
                fade,
                rim,
                bend,
            } => format!(
                r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="{bend}" fill="{fill}" opacity="{fade}"{}/>"#,
                rect.x(),
                rect.y(),
                rect.w(),
                rect.h(),
                rim.as_ref()
                    .map(|(stroke, width)| format!(r#" stroke="{stroke}" stroke-width="{width:.1}""#))
                    .unwrap_or_default(),
            ),
            Self::Label {
                spot,
                text,
                size,
                fill,
                anchor,
                face,
                heft,
                fade,
            } => format!(
                r#"<text x="{:.1}" y="{:.1}" {} font-size="{size}" font-weight="{heft}" fill="{fill}" text-anchor="{}" dominant-baseline="middle" opacity="{fade}">{}</text>"#,
                spot.x(),
                spot.y(),
                face.svg(),
                anchor.svg(),
                text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;"),
            ),
            Self::Trace { spine, stroke, width } => format!(
                r#"<path d="M{}" fill="none" stroke="{stroke}" stroke-width="{width:.1}"/>"#,
                spine
                    .iter()
                    .map(|p| format!("{:.1},{:.1}", p.x(), p.y()))
                    .collect::<Vec<String>>()
                    .join(" L"),
            ),
            Self::Slice {
                pin,
                radius,
                from,
                upto,
                fill,
                rim,
            } => format!(
                r#"<path d="{}" fill="{fill}"{}/>"#,
                Mark::arc(*pin, *radius, *from, *upto),
                Mark::seam(rim.as_ref())
            ),
            Self::Disc { pin, radius, fill, rim } => format!(
                r#"<circle cx="{:.1}" cy="{:.1}" r="{radius:.1}" fill="{fill}"{}/>"#,
                pin.x(),
                pin.y(),
                Mark::seam(rim.as_ref()),
            ),
        }
    }
}

impl Mark {
    /// The SVG path of a pie wedge: center, out along `from`, arc to `upto`,
    /// close. Angles in degrees, clockwise from three o'clock.
    pub fn arc(pin: Point, radius: f64, from: f64, upto: f64) -> String {
        let x0 = pin.x() + radius * from.to_radians().cos();
        let y0 = pin.y() + radius * from.to_radians().sin();
        let x1 = pin.x() + radius * upto.to_radians().cos();
        let y1 = pin.y() + radius * upto.to_radians().sin();
        format!(
            "M{:.1},{:.1} L{x0:.1},{y0:.1} A{radius:.1},{radius:.1} 0 {},1 {x1:.1},{y1:.1} Z",
            pin.x(),
            pin.y(),
            usize::from(upto - from > 180.0),
        )
    }

    fn seam(rim: Option<&(String, f64)>) -> String {
        rim.map(|(stroke, width)| format!(r#" stroke="{stroke}" stroke-width="{width:.1}""#))
            .unwrap_or_default()
    }
}
