use super::*;

/// The whole dashboard-of-reels for one game: tree films in both
/// orientations, the hero|dial|villain triplet, and the equity curve — all
/// on one clock, all from one [`Atlas`] game and one [`Palette`]. The SVG
/// bin and the Leptos page consume this identically.
pub struct Ensemble<'paint, G, P>
where
    G: Atlas,
    P: Palette,
{
    scene: Scene<G>,
    paint: &'paint P,
}

impl<'paint, G, P> Ensemble<'paint, G, P>
where
    G: Atlas,
    P: Palette,
{
    /// The game's authored demonstration, assembled and ready to render.
    pub fn new(paint: &'paint P) -> Self {
        Self {
            scene: Scene::from(G::demo()),
            paint,
        }
    }

    pub fn scene(&self) -> &Scene<G> {
        &self.scene
    }

    fn paint(&self) -> &'paint P {
        self.paint
    }

    /// Number of phases on the shared clock.
    pub fn phases(&self) -> usize {
        self.scene().stints().len()
    }

    /// The tree film in either orientation.
    pub fn film(&self, orient: Orient) -> Film<'_, G, P> {
        Film::new(self.scene(), orient, self.paint())
    }

    /// The hero-sheet | policy-dial | villain-sheet composition.
    pub fn trio(&self) -> Trio {
        let dial = Dial::new(self.scene(), self.paint());
        let hero = Sheet::<G, P>::new(self.phases(), true, ROLES.0, self.paint());
        let vill = Sheet::<G, P>::new(self.phases(), false, ROLES.1, self.paint());
        Trio::new(&[&hero as &dyn Panel, &dial, &vill])
    }

    /// The utility-distribution curve.
    pub fn curve(&self) -> Curve<'_, G, P> {
        Curve::new(self.phases(), self.paint())
    }
}
