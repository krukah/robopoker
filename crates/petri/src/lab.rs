use crate::*;

/// A training run the page can drive without knowing its game or its networks.
pub trait Lab {
    fn step(&mut self, epochs: usize) -> Frame;

    /// Fits the strategy net on everything seen so far. Training never reads it, so this only observes.
    fn checkpoint(&mut self) -> Frame;
}
