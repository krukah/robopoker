use super::*;

/// Cell edge length.
pub const GRAIN: f64 = 28.0;

/// Gap between cells.
pub const GUTTER: f64 = 2.0;

/// Canvas margin around the grid (room for the title).
pub const APRON: f64 = 26.0;

/// Cell-label text size.
pub const TEXT: f64 = 9.0;

/// Title text size.
pub const TITLE: f64 = 11.0;

/// The two sheet roles: hero representing, villain defending.
pub const ROLES: (&str, &str) = ("HERO · REPRESENTING", "VILLAIN · DEFENDING");

/// Opacity curve of a cell weight: floor plus gamma-lifted range.
pub const GLOW: (f64, f64, f64) = (0.05, 0.92, 0.62);

/// Cells dimmer than this are skipped entirely.
pub const DUSK: f64 = 0.09;

/// The secret-space heatmap as a frame function: the game's private
/// information charted on the [`Atlas`] grid, posterior weights tightening
/// phase by phase. The 169-hand range grid is this type instantiated at
/// NLHE; Kuhn charts three ranks.
pub struct Sheet<'paint, G, P>
where
    G: Atlas,
    P: Palette,
{
    phases: usize,
    hero: bool,
    title: String,
    paint: &'paint P,
    game: std::marker::PhantomData<G>,
}

impl<'paint, G, P> Sheet<'paint, G, P>
where
    G: Atlas,
    P: Palette,
{
    pub fn new(phases: usize, hero: bool, title: impl Into<String>, paint: &'paint P) -> Self {
        Self {
            phases,
            hero,
            title: title.into(),
            paint,
            game: std::marker::PhantomData,
        }
    }

    fn phases(&self) -> usize {
        self.phases
    }

    fn hero(&self) -> bool {
        self.hero
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn paint(&self) -> &'paint P {
        self.paint
    }

    /// Canvas size (width, height).
    pub fn size(&self) -> (f64, f64) {
        let (cols, rows) = G::grid();
        (
            cols as f64 * (GRAIN + GUTTER) - GUTTER + 2.0 * APRON,
            rows as f64 * (GRAIN + GUTTER) - GUTTER + 2.0 * APRON,
        )
    }

    /// The same clock as the tree: SPAN frames per phase, then the linger.
    pub fn frames(&self) -> Vec<Frame> {
        Easing::ticks(self.phases()).map(|(s, t)| self.state(s, t)).collect()
    }

    fn state(&self, s: usize, t: f64) -> Frame {
        let (cols, rows) = G::grid();
        let ramp = Easing::fade(t);
        let mut marks = vec![
            Mark::label(Point::new(APRON, APRON / 2.0), self.title().to_string(), TITLE, self.paint().muted())
                .anchor(Anchor::Start)
                .heft(600),
        ];
        for row in 0..rows {
            for col in 0..cols {
                let was = if s == 0 { 0.0 } else { f64::from(G::weight(col, row, s - 1, self.hero())) };
                let now = f64::from(G::weight(col, row, s, self.hero()));
                let weight = was + (now - was) * ramp;
                let glow = GLOW.0 + GLOW.1 * weight.powf(GLOW.2);
                if glow < DUSK {
                    continue;
                }
                let spot = G::spot() == (col, row) && self.hero();
                let tile = Rect::new(
                    APRON + col as f64 * (GRAIN + GUTTER),
                    APRON + row as f64 * (GRAIN + GUTTER),
                    GRAIN,
                    GRAIN,
                );
                let fill = if spot { self.paint().action(Deed::Open) } else { self.paint().fg() };
                marks.push(Mark::block(tile, fill).fade(glow));
                marks.push(
                    Mark::label(
                        Point::new(tile.x() + GRAIN / 2.0, tile.y() + GRAIN / 2.0),
                        G::cell(col, row),
                        TEXT,
                        self.paint().bg(),
                    )
                    .heft(600)
                    .fade(glow),
                );
                if spot {
                    marks.push(
                        Mark::block(Rect::new(tile.x() - 1.0, tile.y() - 1.0, GRAIN + 2.0, GRAIN + 2.0), "none")
                            .rim(self.paint().action(Deed::Open), 1.5),
                    );
                }
            }
        }
        Frame::new(marks)
    }
}

impl<G, P> Panel for Sheet<'_, G, P>
where
    G: Atlas,
    P: Palette,
{
    fn size(&self) -> (f64, f64) {
        self.size()
    }

    fn frames(&self) -> Vec<Frame> {
        Sheet::frames(self)
    }
}
