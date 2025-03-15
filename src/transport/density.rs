use super::support::Support;
use crate::Probability;

/// generalization of any probability distribution over
/// arbitrary Support.
pub trait Density {
    type Support: Support;

    fn density(&self, x: &Self::Support) -> Probability;
    fn support(&self) -> impl Iterator<Item = &Self::Support>;
}
