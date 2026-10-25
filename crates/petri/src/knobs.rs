use neural::*;

/// [`DeepHyperParams`] as the page sends them, as JSON.
#[derive(serde::Deserialize)]
pub struct Knobs {
    traversals: usize,
    capacity: usize,
    width: usize,
    depth: usize,
    steps: usize,
    batch: usize,
    rate: f64,
    #[serde(default)]
    warm: bool,
}

impl From<Knobs> for DeepHyperParams {
    fn from(knobs: Knobs) -> Self {
        Self::new(knobs.traversals, knobs.capacity, knobs.width, knobs.depth, knobs.steps, knobs.batch, knobs.rate)
            .with_warm(knobs.warm)
    }
}
