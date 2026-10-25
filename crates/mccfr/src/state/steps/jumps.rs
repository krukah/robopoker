use super::*;

/// Direction-agnostic extensions on any stream of [`Jump`] pairs.
///
/// Only edge-level projections belong here; anything depending on the structural
/// meaning of sequence order (e.g. splitting at chance boundaries) must be keyed
/// to a specific direction.
pub trait Jumps: IntoIterator + Sized
where
    Self::Item: Jump,
{
    /// Discard the anchor turns; yield only edges in original order.
    fn edges(self) -> impl Iterator<Item = <Self::Item as Jump>::E> {
        self.into_iter().map(Jump::edge)
    }
}
