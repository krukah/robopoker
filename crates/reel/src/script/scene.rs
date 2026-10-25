use crate::*;

/// The assembled demonstration: the flattened layer sequence plus per-phase
/// stints, derived generically from a [`Demo`] by walking its real states —
/// phase boundaries from [`mccfr::CfrGame::depth`], reveals and badges from
/// [`Scenic`]. This replaces every hand-authored menu.
#[derive(Debug, Clone)]
pub struct Scene<G>
where
    G: Scenic,
{
    layers: Vec<Layer<G::E>>,
    stints: Vec<Stint>,
}

impl<G> From<Demo<G>> for Scene<G>
where
    G: Scenic,
{
    fn from(demo: Demo<G>) -> Self {
        let hero = demo.hero();
        let mut layers = Vec::new();
        let mut stints = Vec::<Stint>::new();
        let mut phase = None;
        for step in demo.steps() {
            if phase != Some(step.state().depth()) {
                phase = Some(step.state().depth());
                stints.push(Stint::new(layers.len()));
                layers.push(Layer::Board {
                    phase: step.state().phase(),
                    tokens: step.state().reveals(hero),
                });
            }
            stints.last_mut().expect("open stint").push(layers.len(), step.pick());
            layers.push(Layer::Choice {
                badge: G::badge(step.state().turn()),
                heroic: step.state().turn() == hero,
                menu: step.menu().to_vec(),
            });
        }
        layers.extend(demo.tally().cloned().map(Layer::Settle));
        Self { layers, stints }
    }
}

impl<G> Scene<G>
where
    G: Scenic,
{
    pub fn layers(&self) -> &[Layer<G::E>] {
        &self.layers
    }

    pub fn stints(&self) -> &[Stint] {
        &self.stints
    }

    /// The menu at a decision layer.
    pub fn menu(&self, i: usize) -> &[G::E] {
        match &self.layers()[i] {
            Layer::Choice { menu, .. } => menu,
            Layer::Board { .. } | Layer::Settle(_) => &[],
        }
    }

    /// True if the hero acts at this decision layer.
    pub fn heroic(&self, i: usize) -> bool {
        matches!(&self.layers()[i], Layer::Choice { heroic: true, .. })
    }
}
