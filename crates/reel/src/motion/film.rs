use super::*;
use pokerkit::Utility;

/// Time steps a phase occupies at most — equal length when probes suffice.
pub const SPAN: usize = 26;

/// Held frames on the completed hand — the only linger in the loop.
pub const LINGER: usize = 40;

/// Playback rate.
pub const FPS: usize = 20;

/// Gain painted on the chosen line as it closes a phase.
pub const CHOSEN: Utility = 0.9;

/// Opacity of dealt tokens in the frozen prefix.
pub const DEALT: f64 = 0.85;

/// Stroke width of the frozen spine and its rims.
pub const SPINE: f64 = 1.5;

/// Rim offset as a fraction of the live trace width.
pub const TRIM: f64 = 0.55;

/// Opacity of an untaken action cell — the site tree's `opacity-40`.
pub const FAINT: f64 = 0.4;

/// Offset of the chosen cell's ring, leaving a sliver of ground between ring and cell.
pub const HALO: f64 = 2.5;

/// Hairline width of token-face borders.
pub const HAIR: f64 = 0.75;

/// Rank rune size, and the fraction pair for two-rune token faces.
pub const RUNE: (f64, f64, f64) = (11.0, 0.34, 0.70);

/// Suit pip size.
pub const SUIT: f64 = 12.0;

/// Tile glyph size for single-character and wider symbols.
pub const SYM: (f64, f64) = (10.0, 8.5);

/// Actor-tag and phase-label text sizes.
pub const TAG: (f64, f64) = (10.0, 9.0);

/// Bold weight for tile glyphs and token ranks.
pub const BOLD: u32 = 700;

/// Semibold weight for actor tags.
pub const SEMI: u32 = 600;

/// Regret-to-stroke-width line: base width plus swell per unit magnitude.
pub const WIDTH: (f64, f64) = (1.2, 2.4);

/// Rim width from its offset: gain factor with a floor.
pub const RIM: (f64, f64) = (1.2, 1.5);

/// How far into a phase the panel crossfades complete.
pub const RISE: f64 = 0.35;

/// The Frame Function: an assembled [`Scene`] rendered through a [`Lay`] and a [`Palette`] into
/// a deterministic sequence of [`Frame`]s — generic up to the mccfr trait hierarchy through
/// [`Scenic`]. Each phase runs exploration probes that grow one edge per frame and reset
/// sharply; the chosen line closes the phase and freezes as the site tree draws it; later phases
/// stay hidden until reached; only the completed hand lingers.
pub struct Film<'scene, G, P>
where
    G: Scenic,
    P: Palette,
{
    scene: &'scene Scene<G>,
    lay: Lay,
    paint: &'scene P,
}

impl<'scene, G, P> Film<'scene, G, P>
where
    G: Scenic,
    P: Palette,
{
    pub fn new(scene: &'scene Scene<G>, orient: Orient, paint: &'scene P) -> Self {
        let lay = Lay::new(orient, scene.layers().iter().map(Layer::count).collect());
        Self { scene, lay, paint }
    }

    pub(crate) fn scene(&self) -> &Scene<G> {
        self.scene
    }

    fn lay(&self) -> &Lay {
        &self.lay
    }

    fn paint(&self) -> &'scene P {
        self.paint
    }

    /// Canvas size (width, height).
    pub fn size(&self) -> (f64, f64) {
        self.lay().size()
    }

    /// Every frame of the loop, lazily: each phase's [`Span`], then the
    /// linger on the completed hand.
    pub fn frames(&self) -> impl Iterator<Item = Frame> + '_ {
        (0..self.scene().stints().len())
            .flat_map(|phase| Span::new(self, phase))
            .chain(std::iter::repeat_with(|| self.close()).take(LINGER))
    }

    /// The representative still: the deepest exploration the scene's
    /// deepest phase produces, or the completed hand when none exists.
    pub fn still(&self) -> Frame {
        let (deep, depth) = self
            .scene()
            .stints()
            .iter()
            .enumerate()
            .map(|(s, stint)| (s, stint.moves().len()))
            .max_by_key(|(_, depth)| *depth)
            .unwrap_or((0, 0));
        Probe::ordered(self.scene(), deep)
            .into_iter()
            .find(|probe| probe.line().len() >= depth)
            .map_or_else(|| self.close(), |probe| self.state(deep, probe.line(), probe.gain(), depth))
    }

    /// The animated flipbook: every frame in one SVG, revealed by discrete
    /// SMIL opacity — declarative, so it plays inline on GitHub.
    pub fn svg(&self) -> String {
        let (w, h) = self.size();
        Frame::book(&Panel::frames(self), w, h, &self.paint().bg())
    }

    /// The completed hand: every phase frozen, then the settlement — the
    /// linger frame. The terminal band belongs here and only here: during
    /// the last phase the hand hasn't ended yet, so painting it early would
    /// spoil an outcome the probes are still exploring.
    fn close(&self) -> Frame {
        Frame::new(
            [
                self.bands(self.scene().stints().len()),
                self.frozen(self.scene().stints().len()),
                self.settled(),
            ]
            .concat(),
        )
    }

    /// The settlement layer's index, when the scene has one.
    fn terminus(&self) -> Option<usize> {
        self.scene()
            .layers()
            .iter()
            .position(|layer| matches!(layer, Layer::Settle(_)))
    }

    /// The terminal column: its band, the spine segment that reaches it
    /// from the hand's last committed node, and its result tiles. Empty for
    /// a scene whose demo never settled.
    fn settled(&self) -> Vec<Mark> {
        let Some(i) = self.terminus() else {
            return Vec::new();
        };
        let stint = self.scene().stints().last().expect("at least one phase");
        let (from, at) = match (stint.moves().last(), stint.picks().last()) {
            (Some(&j), Some(&slot)) => (j, self.lay().slot(j, slot)),
            _ => (stint.deal(), self.lay().center(stint.deal())),
        };
        [
            vec![
                Mark::block(self.lay().band(i), self.paint().settle()).bend(0.0),
                Mark::trace(self.lay().elbow(from, at, i, self.lay().center(i)), self.paint().fg(), SPINE),
            ],
            self.tokens(i, DEALT),
            vec![self.label(i), self.tag(i)],
        ]
        .concat()
    }

    /// One instant: frozen prefix, faded options at the probe's frontier
    /// only, and the live trajectory strode out to `depth` in regret paint.
    pub(crate) fn state(&self, phase: usize, line: &[Slot], gain: Utility, depth: usize) -> Frame {
        let hue = self.paint().regret(gain);
        let width = Easing::breadth(gain);
        let stint = &self.scene().stints()[phase];
        let head = [
            self.bands(phase + 1),
            self.frozen(phase),
            self.options(stint.moves()[depth - 1]),
        ]
        .concat();
        Frame::new(
            line.iter()
                .take(depth)
                .zip(stint.moves().iter().copied())
                .fold((head, (stint.deal(), self.lay().center(stint.deal()))), |(mut marks, prev), (&slot, i)| {
                    marks.extend(self.stride(prev, i, slot, &hue, width));
                    (marks, (i, self.lay().slot(i, slot)))
                })
                .0,
        )
    }

    /// One stride of the live trajectory: the elbow in, the lit node, its tag.
    fn stride(&self, prev: (usize, Cross), i: usize, slot: Slot, hue: &str, width: f64) -> Vec<Mark> {
        [
            vec![Mark::trace(
                self.lay().elbow(prev.0, prev.1, i, self.lay().slot(i, slot)),
                hue,
                width,
            )],
            self.node(i, slot, hue, width * TRIM, (width * TRIM * RIM.0).max(RIM.1)),
            vec![self.tag(i)],
        ]
        .concat()
    }

    /// The striped ground under every lane, then the street material behind
    /// the board layers of the `k` visible phases: pocket stock for the
    /// private first phase, felt for the rest.
    fn bands(&self, k: usize) -> Vec<Mark> {
        (0..self.scene().layers().len())
            .flat_map(|i| {
                std::iter::once(Mark::block(self.lay().band(i), self.paint().ground()).bend(0.0))
                    .chain((i % 2 == 1).then(|| Mark::block(self.lay().band(i), self.paint().stripe()).bend(0.0)))
            })
            .chain(self.scene().stints()[..k].iter().enumerate().map(|(s, stint)| {
                let color = if s == 0 { self.paint().pocket() } else { self.paint().felt() };
                Mark::block(self.lay().band(stint.deal()), color).bend(0.0)
            }))
            .collect()
    }

    /// Phases `0..upto` frozen plus every board dealt so far, the spine
    /// threading through them.
    fn frozen(&self, upto: usize) -> Vec<Mark> {
        let stints = self.scene().stints();
        let mut marks = Vec::new();
        (0..=upto.min(stints.len() - 1)).for_each(|k| {
            marks.extend(self.tokens(stints[k].deal(), DEALT));
            marks.push(self.label(stints[k].deal()));
            marks.push(self.tag(stints[k].deal()));
        });
        let mut spine = vec![(stints[0].deal(), self.lay().center(stints[0].deal()))];
        (0..upto).for_each(|k| {
            let (nodes, pins) = self.freeze(k);
            marks.extend(nodes);
            spine.extend(pins);
            if k + 1 < stints.len() {
                spine.push((stints[k + 1].deal(), self.lay().center(stints[k + 1].deal())));
            }
        });
        spine.windows(2).for_each(|pair| {
            marks.push(Mark::trace(
                self.lay().elbow(pair[0].0, pair[0].1, pair[1].0, pair[1].1),
                self.paint().fg(),
                SPINE,
            ));
        });
        marks
    }

    /// One phase's frozen contribution: each decision's untaken cells dimmed,
    /// its taken cell lit and ringed, and the spine anchors they pin.
    fn freeze(&self, phase: usize) -> (Vec<Mark>, Vec<(usize, Cross)>) {
        let stint = &self.scene().stints()[phase];
        stint.moves().iter().copied().zip(stint.picks().iter().copied()).fold(
            (Vec::new(), Vec::new()),
            |(mut marks, mut spine), (i, slot)| {
                marks.extend(
                    (0..self.scene().menu(i).len())
                        .filter(|&k| k != slot.raw())
                        .flat_map(|k| self.cell(i, Slot::new(k), FAINT)),
                );
                marks.extend(self.node(i, slot, &self.paint().fg(), HALO, SPINE));
                marks.push(self.tag(i));
                spine.push((i, self.lay().slot(i, slot)));
                (marks, spine)
            },
        )
    }

    /// The faded option tiles at one decision layer — the probe's frontier.
    fn options(&self, i: usize) -> Vec<Mark> {
        (0..self.scene().menu(i).len())
            .flat_map(|k| self.cell(i, Slot::new(k), FAINT))
            .chain(std::iter::once(self.tag(i)))
            .collect()
    }

    /// One action cell as the site draws it: action fill, hairline rule, and
    /// a knockout glyph, all at one opacity.
    fn cell(&self, i: usize, slot: Slot, fade: f64) -> Vec<Mark> {
        let edge = &self.scene().menu(i)[slot.raw()];
        let tile = self.lay().tile(i, slot);
        vec![
            Mark::block(tile, self.paint().action(G::deed(edge)))
                .rim(self.paint().border(), HAIR)
                .fade(fade),
            self.glyph(tile, &G::glyph(edge), self.paint().inverse()).fade(fade),
        ]
    }

    /// A lit decision cell ringed at `offset`: the regret hue on a live
    /// probe, the foreground on the committed line.
    fn node(&self, i: usize, slot: Slot, rim: &str, offset: f64, width: f64) -> Vec<Mark> {
        [
            self.cell(i, slot, 1.0),
            vec![Mark::block(Easing::grow(self.lay().tile(i, slot), offset), "none").rim(rim, width)],
        ]
        .concat()
    }

    /// The token faces of a major layer — board reveals or settlement tiles.
    fn tokens(&self, i: usize, fade: f64) -> Vec<Mark> {
        self.scene().layers()[i]
            .faces()
            .iter()
            .enumerate()
            .flat_map(|(k, token)| {
                let tile = self.lay().tile(i, Slot::new(k));
                std::iter::once(
                    Mark::block(tile, self.paint().white())
                        .rim(self.paint().border(), HAIR)
                        .fade(fade),
                )
                .chain(token.runes().iter().enumerate().map(move |(r, rune)| {
                    let across = match token.runes().len() {
                        1 => 0.5,
                        2 => [RUNE.1, RUNE.2][r],
                        n => (r as f64 + 0.5) / n as f64,
                    };
                    let (size, heft) = match rune.face() {
                        Face::Pip => (SUIT, 400),
                        _ => (RUNE.0, BOLD),
                    };
                    let ink = match rune.tint() {
                        Tint::Dark => self.paint().black(),
                        Tint::Red => self.paint().red(),
                        Tint::Gold => self.paint().gold(),
                    };
                    Mark::label(Point::new(tile.x() + CELL * across, tile.y() + CELL / 2.0), rune.text(), size, ink)
                        .face(rune.face())
                        .heft(heft)
                        .fade(fade)
                }))
                .collect::<Vec<Mark>>()
            })
            .collect()
    }

    /// A centered tile glyph, sized down for multi-character symbols.
    fn glyph(&self, tile: Rect, text: &str, fill: String) -> Mark {
        let size = if text.chars().count() <= 1 { SYM.0 } else { SYM.1 };
        Mark::label(Point::new(tile.x() + CELL / 2.0, tile.y() + CELL / 2.0), text, size, fill).heft(BOLD)
    }

    /// The actor label of a layer, anchored per orientation.
    fn tag(&self, i: usize) -> Mark {
        let (spot, anchor) = self.lay().actor(i);
        Mark::label(spot, self.scene().layers()[i].tag(), TAG.0, self.paint().muted())
            .anchor(anchor)
            .face(Face::Caps)
            .heft(SEMI)
    }

    /// The phase-band caption of a major layer — the street for a board,
    /// the verdict for a settlement — anchored per orientation.
    fn label(&self, i: usize) -> Mark {
        let (spot, anchor) = self.lay().street(i);
        Mark::label(spot, self.scene().layers()[i].caption(), TAG.1, self.paint().muted())
            .anchor(anchor)
            .face(Face::Caps)
    }
}

impl<G, P> Panel for Film<'_, G, P>
where
    G: Scenic,
    P: Palette,
{
    fn size(&self) -> (f64, f64) {
        self.size()
    }

    fn frames(&self) -> Vec<Frame> {
        Film::frames(self).collect()
    }
}
