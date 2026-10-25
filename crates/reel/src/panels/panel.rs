use crate::*;

/// One reel panel: anything that renders a frame sequence on the shared
/// clock with a fixed canvas. The composition currency of [`crate::Trio`]
/// and the mounting currency of the client — object-safe, game-erased.
pub trait Panel {
    /// Canvas size (width, height).
    fn size(&self) -> (f64, f64);

    /// The panel's frames, materialized.
    fn frames(&self) -> Vec<Frame>;
}
