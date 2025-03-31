use super::edge::Edge;
use super::turn::Turn;

/// the information bucket is fully abstracted. it must be implemented
/// by the consumer of this MCCFR API.
///
/// the implementation must be able to determine:
///  what possible Edges may emerge from this Node (Decision)
///
/// the generation of this information is the responsibility of the Encoder,
/// which has global tree context and may make probabilistic or path-dependent decisions
pub trait Info: Clone + Copy + PartialEq + Eq + std::hash::Hash {
    type E: Edge;
    type T: Turn;
    fn choices(&self) -> Vec<Self::E>;
}
impl Info for crate::mccfr::bucket::Bucket {
    type E = crate::gameplay::edge::Edge;
    type T = crate::gameplay::ply::Turn;
    fn choices(&self) -> Vec<Self::E> {
        self.2.into_iter().collect()
    }
}
