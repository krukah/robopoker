use super::*;

/// Equity buckets per series.
pub const BUCKETS: usize = 32;

/// Canvas size (width, height).
pub const CANVAS: (f64, f64) = (560.0, 360.0);

/// Plot margins: (left/right, top, bottom).
pub const INSET: (f64, f64, f64) = (18.0, 46.0, 44.0);

/// Series stroke width.
pub const STROKE: f64 = 2.0;

/// Tallest bar as a fraction of plot height.
pub const SCALE: f64 = 0.85;

/// The utility-distribution panel as a frame function: the game's
/// (observation, abstraction) bucket series morphing phase by phase on the
/// shared reel clock, with mean carets — typed by the [`Atlas`] bridge.
pub struct Curve<'paint, G, P>
where
    G: Atlas,
    P: Palette,
{
    phases: usize,
    paint: &'paint P,
    game: std::marker::PhantomData<G>,
}

impl<'paint, G, P> Curve<'paint, G, P>
where
    G: Atlas,
    P: Palette,
{
    pub fn new(phases: usize, paint: &'paint P) -> Self {
        Self {
            phases,
            paint,
            game: std::marker::PhantomData,
        }
    }

    fn phases(&self) -> usize {
        self.phases
    }

    fn paint(&self) -> &'paint P {
        self.paint
    }

    /// Canvas size (width, height).
    pub fn size(&self) -> (f64, f64) {
        CANVAS
    }

    /// The same clock as the tree: SPAN frames per phase, then the linger.
    pub fn frames(&self) -> Vec<Frame> {
        Easing::ticks(self.phases()).map(|(s, t)| self.state(s, t)).collect()
    }

    fn state(&self, s: usize, t: f64) -> Frame {
        let (w, h) = (CANVAS.0 - 2.0 * INSET.0, CANVAS.1 - INSET.1 - INSET.2);
        let (x, y) = (INSET.0, INSET.1);
        let ramp = Easing::fade(t);
        let (obs, abs) = G::curves(s);
        let (past, prior) = G::curves(s.saturating_sub(1));
        let mut marks = vec![
            Mark::trace(vec![Point::new(x, y + h), Point::new(x + w, y + h)], self.paint().border(), 1.0),
            Mark::label(Point::new(x, INSET.1 / 2.0), "EQUITY", 11.0, self.paint().fg())
                .anchor(Anchor::Start)
                .face(Face::Caps)
                .heft(700),
            Mark::label(Point::new(x, y + h + 18.0), "0.0", 9.0, self.paint().muted())
                .anchor(Anchor::Start)
                .face(Face::Data),
            Mark::label(Point::new(x + w, y + h + 18.0), "1.0", 9.0, self.paint().muted())
                .anchor(Anchor::End)
                .face(Face::Data),
        ];
        (1..4).for_each(|k| {
            let gx = x + w * k as f64 / 4.0;
            marks.push(Mark::trace(vec![Point::new(gx, y), Point::new(gx, y + h)], self.paint().border(), 0.5));
        });
        for (was, now, ink) in [(prior, abs, self.paint().gold()), (past, obs, self.paint().observed())] {
            let mix = (0..BUCKETS)
                .map(|k| was[k] + (now[k] - was[k]) * ramp)
                .collect::<Vec<f64>>();
            let peak = mix.iter().copied().fold(0.0, f64::max).max(1e-9);
            let mut spine = vec![Point::new(x, y + h)];
            mix.iter().enumerate().for_each(|(k, weight)| {
                let bar = y + h - (weight / peak) * SCALE * h;
                spine.push(Point::new(x + k as f64 * w / BUCKETS as f64, bar));
                spine.push(Point::new(x + (k + 1) as f64 * w / BUCKETS as f64, bar));
            });
            spine.push(Point::new(x + w, y + h));
            marks.push(Mark::trace(spine, ink.clone(), STROKE));
            let mean = mix
                .iter()
                .enumerate()
                .map(|(k, weight)| weight * (k as f64 + 0.5) / BUCKETS as f64)
                .sum::<f64>()
                / mix.iter().sum::<f64>().max(1e-9);
            marks.push(Mark::slice(Point::new(x + mean * w, y + h + 4.0), 6.0, -125.0, -55.0, ink));
        }
        Frame::new(marks)
    }
}

impl<G, P> Panel for Curve<'_, G, P>
where
    G: Atlas,
    P: Palette,
{
    fn size(&self) -> (f64, f64) {
        self.size()
    }

    fn frames(&self) -> Vec<Frame> {
        Curve::frames(self)
    }
}
