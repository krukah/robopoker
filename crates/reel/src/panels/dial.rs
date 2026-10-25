use super::*;

/// Wedge radius.
pub const RADIUS: f64 = 96.0;

/// Donut hole as a fraction of the radius.
pub const HOLE: f64 = 0.5;

/// Canvas padding around the dial.
pub const ORBIT: f64 = 24.0;

/// Seam width between wedges.
pub const RIFT: f64 = 1.5;

/// Center-glyph text size.
pub const FACE: f64 = 13.0;

/// Chosen-slot policy concentration: base share plus growth per phase.
pub const LEAD: (f64, f64) = (0.5, 0.12);

/// The policy pie as a frame function: at each phase, the hero's decision
/// distribution over the REAL menu — wedges typed by edge, colored by
/// [`crate::Deed`], resolving toward the chosen slot as the hand progresses.
/// Weights are demo-synthetic (concentration on the pick) until Profiled
/// walks land; the types are the point.
pub struct Dial<'scene, G, P>
where
    G: Scenic,
    P: Palette,
{
    scene: &'scene Scene<G>,
    paint: &'scene P,
}

impl<'scene, G, P> Dial<'scene, G, P>
where
    G: Scenic,
    P: Palette,
{
    pub fn new(scene: &'scene Scene<G>, paint: &'scene P) -> Self {
        Self { scene, paint }
    }

    fn scene(&self) -> &'scene Scene<G> {
        self.scene
    }

    fn paint(&self) -> &'scene P {
        self.paint
    }

    /// Canvas size (width, height).
    pub fn size(&self) -> (f64, f64) {
        (2.0 * (RADIUS + ORBIT), 2.0 * (RADIUS + ORBIT))
    }

    /// The same clock as the tree: SPAN frames per phase, then the linger.
    pub fn frames(&self) -> Vec<Frame> {
        Easing::ticks(self.scene().stints().len())
            .map(|(s, t)| self.state(s, t))
            .collect()
    }

    /// The hero's decision layer of a phase: the wedge source.
    fn helm(&self, s: usize) -> (usize, usize) {
        let stint = &self.scene().stints()[s];
        stint
            .moves()
            .iter()
            .copied()
            .zip(stint.picks().iter().copied())
            .find(|(i, _)| self.scene().heroic(*i))
            .map_or((stint.moves()[0], stint.picks()[0].raw()), |(i, pick)| (i, pick.raw()))
    }

    /// Policy weights over a phase's menu: the pick leads, growing by phase.
    fn policy(&self, s: usize) -> Vec<f64> {
        let (layer, pick) = self.helm(s);
        let n = self.scene().menu(layer).len();
        let lead = (LEAD.0 + LEAD.1 * s as f64).min(0.92);
        (0..n)
            .map(|k| if k == pick { lead } else { (1.0 - lead) / (n - 1).max(1) as f64 })
            .collect()
    }

    fn state(&self, s: usize, t: f64) -> Frame {
        let pin = Point::new(RADIUS + ORBIT, RADIUS + ORBIT);
        let ramp = Easing::fade(t);
        let last = if s == 0 { Vec::new() } else { self.policy(s - 1) };
        let (layer, pick) = self.helm(s);
        let menu = self.scene().menu(layer);
        let next = self.policy(s);
        let mut marks = Vec::new();
        let mut angle = -90.0;
        for (k, edge) in menu.iter().enumerate() {
            let was = last.get(k).copied().unwrap_or(1.0 / menu.len() as f64);
            let sweep = 360.0 * (was + (next[k] - was) * ramp);
            marks.push(
                Mark::slice(pin, RADIUS, angle, (angle + sweep).min(angle + 359.9), self.paint().action(G::deed(edge)))
                    .rim(self.paint().bg(), RIFT),
            );
            angle += sweep;
        }
        marks.push(Mark::disc(pin, RADIUS * HOLE, self.paint().bg()).rim(self.paint().border(), 1.0));
        marks.push(Mark::label(pin, G::glyph(&menu[pick]), FACE, self.paint().fg()).heft(700));
        Frame::new(marks)
    }
}

impl<G, P> Panel for Dial<'_, G, P>
where
    G: Scenic,
    P: Palette,
{
    fn size(&self) -> (f64, f64) {
        self.size()
    }

    fn frames(&self) -> Vec<Frame> {
        Dial::frames(self)
    }
}
