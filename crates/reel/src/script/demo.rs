use crate::*;

/// The authored demonstration hand: the hero's seat and the concrete
/// decisions along the chosen line, rigged by the game itself (cards fixed,
/// menus real). This is the materialized form of a forward-playing walk —
/// the lazy `Walk`/`Arc::Profiled` generalization is the stage-4 horizon.
#[derive(Debug, Clone)]
pub struct Demo<G>
where
    G: Scenic,
{
    hero: G::T,
    steps: Vec<Step<G>>,
    tally: Option<Tally>,
}

impl<G> Demo<G>
where
    G: Scenic,
{
    pub fn new(hero: G::T, steps: Vec<Step<G>>) -> Self {
        Self {
            hero,
            steps,
            tally: None,
        }
    }

    /// Attach the settlement. A demo whose line reaches a terminal can say
    /// how it ended; one that stops mid-hand simply doesn't, and the scene
    /// renders without a terminal column.
    pub fn settled(self, tally: Tally) -> Self {
        Self {
            tally: Some(tally),
            ..self
        }
    }

    pub fn tally(&self) -> Option<&Tally> {
        self.tally.as_ref()
    }

    pub fn hero(&self) -> G::T {
        self.hero
    }

    pub fn steps(&self) -> &[Step<G>] {
        &self.steps
    }
}
