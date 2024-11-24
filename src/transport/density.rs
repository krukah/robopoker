use super::support::Support;
use crate::Probability;

/// generalization of any probability distribution over
/// arbitrary Support.
pub trait Density {
    type S: Support;

    fn density(&self, x: &Self::S) -> Probability;
    fn support(&self) -> impl Iterator<Item = &Self::S>;
}
