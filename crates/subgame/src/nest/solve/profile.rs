use crate::*;

/// Local accumulators for off-tree nesting, over the blueprint lifted into nest edges.
pub type NestProfile<'blueprint, P> = Overlay<NestView<'blueprint, P>>;
