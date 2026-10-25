use crate::*;

/// Local accumulators for depth-limited solving, over the blueprint lifted into depth edges.
pub type DepthProfile<'blueprint, P, const D: usize> = Overlay<DepthView<'blueprint, P, D>>;
