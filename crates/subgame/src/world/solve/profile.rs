use crate::*;

/// Local accumulators for safe subgame solving, over the blueprint lifted into world infos.
pub type WorldProfile<'blueprint, P> = Overlay<WorldView<'blueprint, P>>;
